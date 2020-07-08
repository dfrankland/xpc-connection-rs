#[allow(
    dead_code,
    safe_packed_borrows,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    clippy::all
)]
extern crate xpc_connection_sys;

mod message;

use std::ffi::CStr;
use std::{ops::Deref, ptr};
use std::{pin::Pin, task::Poll};

use block::ConcreteBlock;

use futures::{
    channel::mpsc::{unbounded as unbounded_channel, UnboundedReceiver, UnboundedSender},
    Stream,
};

pub use self::message::*;
use xpc_connection_sys::{
    dispatch_queue_create, xpc_connection_cancel, xpc_connection_create_mach_service,
    xpc_connection_resume, xpc_connection_send_message, xpc_connection_set_event_handler,
    xpc_connection_t, xpc_object_t, xpc_release, XPC_CONNECTION_MACH_SERVICE_LISTENER,
    XPC_CONNECTION_MACH_SERVICE_PRIVILEGED,
};

#[derive(Debug)]
struct Connection(xpc_connection_t);

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            xpc_connection_cancel(self.0);
            xpc_release(self.0 as xpc_object_t);
        }
    }
}

unsafe impl Send for Connection {}

#[derive(Debug)]
pub struct XpcListener {
    connection: Connection,
    receiver: UnboundedReceiver<XpcClient>,
    sender: UnboundedSender<XpcClient>,
}

impl Stream for XpcListener {
    type Item = XpcClient;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Stream::poll_next(Pin::new(&mut self.receiver), cx)
    }
}

impl XpcListener {
    /// The connection must be a listener.
    // TODO: Is there a way to verify that the xpc_connection_t is a listener?
    fn from_raw(connection: xpc_connection_t) -> XpcListener {
        let (sender, receiver) = unbounded_channel();
        let sender_clone = sender.clone();

        let block = ConcreteBlock::new(move |event| match xpc_object_to_message(event) {
            Message::Client(client) => sender_clone.unbounded_send(client).ok(),
            _ => None,
        });

        // We must move it from the stack to the heap so that when the libxpc
        // reference count is released we don't double free. This limitation is
        // explained in the blocks crate.
        let block = block.copy();

        unsafe {
            xpc_connection_set_event_handler(connection, block.deref() as *const _ as *mut _);
            xpc_connection_resume(connection);
        }

        XpcListener {
            connection: Connection(connection),
            receiver,
            sender,
        }
    }

    pub fn listen(name: impl AsRef<CStr>) -> Self {
        let name = name.as_ref();
        let queue = unsafe { dispatch_queue_create(name.as_ptr(), ptr::null_mut()) };
        let flags = XPC_CONNECTION_MACH_SERVICE_LISTENER as u64;
        let connection =
            unsafe { xpc_connection_create_mach_service(name.as_ref().as_ptr(), queue, flags) };
        Self::from_raw(connection)
    }
}

#[derive(Debug)]
pub struct XpcClient {
    connection: Connection,
    receiver: UnboundedReceiver<Message>,
    sender: UnboundedSender<Message>,
}

impl Stream for XpcClient {
    type Item = Message;

    /// `Poll::Ready(None)` returned in place of `MessageError::ConnectionInvalid`
    /// as it's not recoverable. `MessageError::ConnectionInterrupted` should
    /// be treated as recoverable.
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match Stream::poll_next(Pin::new(&mut self.receiver), cx) {
            Poll::Ready(Some(Message::Error(MessageError::ConnectionInvalid))) => Poll::Ready(None),
            v => v,
        }
    }
}

impl XpcClient {
    /// This sets up a client connection's event handler so that its `Stream`
    /// implementation can be used.
    fn from_raw(connection: xpc_connection_t) -> Self {
        let (sender, receiver) = unbounded_channel();
        let sender_clone = sender.clone();

        // Handle messages received
        let block = ConcreteBlock::new(move |event| {
            let message = xpc_object_to_message(event);
            sender_clone.unbounded_send(message).ok()
        });

        // We must move it from the stack to the heap so that when the libxpc
        // reference count is released we don't double free. This limitation is
        // explained in the blocks crate.
        let block = block.copy();

        unsafe {
            xpc_connection_set_event_handler(connection, block.deref() as *const _ as *mut _);
            xpc_connection_resume(connection);
        }

        XpcClient {
            connection: Connection(connection),
            receiver,
            sender,
        }
    }

    /// The connection isn't established until the first call to `send_message`.
    pub fn connect(name: impl AsRef<CStr>) -> Self {
        let name = name.as_ref();
        let queue = unsafe { dispatch_queue_create(name.as_ptr(), ptr::null_mut()) };
        let flags = XPC_CONNECTION_MACH_SERVICE_PRIVILEGED as u64;
        let connection = unsafe { xpc_connection_create_mach_service(name.as_ptr(), queue, flags) };
        Self::from_raw(connection)
    }

    /// The connection is established on the first call to `send_message`. You
    /// may receive an error relating to an invalid mach port name or a variety
    /// of other issues.
    pub fn send_message(&self, message: Message) {
        let xpc_object = message_to_xpc_object(message);
        unsafe {
            xpc_connection_send_message(self.connection.0, xpc_object);
            xpc_release(xpc_object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{executor::block_on, StreamExt};
    use std::{collections::HashMap, ffi::CString};
    use xpc_connection_sys::xpc_connection_cancel;

    // This also tests that the event handler block is only freed once, as a
    // double free is possible if the block isn't copied on to the heap.
    #[test]
    fn event_handler_receives_error_on_close() {
        let mach_port_name = CString::new("com.apple.blued").unwrap();
        let mut client = XpcClient::connect(&mach_port_name);
        let connection = client.connection.0;

        // Cancelling the connection will cause the event handler to be called
        // with an error message. This will happen under normal circumstances,
        // for example if the service invalidates the connection.
        unsafe {
            xpc_connection_cancel(connection);
        }

        if let Some(message) = block_on(client.next()) {
            panic!("Expected `None`, but received {:?}", message);
        }
    }

    #[test]
    fn stream_closed_on_drop() -> Result<(), Box<dyn std::error::Error>> {
        let mach_port_name = CString::new("com.apple.blued")?;
        let mut client = XpcClient::connect(&mach_port_name);
        let connection = client.connection.0;

        let message = Message::Dictionary({
            let mut dictionary = HashMap::new();
            dictionary.insert(CString::new("kCBMsgId")?, Message::Int64(1));
            dictionary.insert(
                CString::new("kCBMsgArgs")?,
                Message::Dictionary({
                    let mut temp = HashMap::new();
                    temp.insert(CString::new("kCBMsgArgAlert")?, Message::Int64(1));
                    temp.insert(
                        CString::new("kCBMsgArgName")?,
                        Message::String(CString::new("rust")?),
                    );
                    temp
                }),
            );
            dictionary
        });

        // Can get data while the channel is open
        client.send_message(message);

        let mut count = 0;

        loop {
            match block_on(client.next()) {
                Some(Message::Error(error)) => {
                    panic!("Error: {:?}", error);
                }
                Some(message) => {
                    println!("Received message: {:?}", message);
                    count += 1;

                    // Explained in `event_handler_receives_error_on_close`.
                    unsafe {
                        xpc_connection_cancel(connection);
                    }
                }
                None => {
                    // We can't be sure how many buffered messages we'll receive
                    // from blued before the connection is cancelled, but it's
                    // safe to say it should be less than 5.
                    assert!(count < 5);
                    return Ok(());
                }
            }
        }
    }
}
