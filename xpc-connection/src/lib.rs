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

use std::{ffi::CStr, ops::Deref, ptr};

use block::ConcreteBlock;

use futures::channel::mpsc::{unbounded as unbounded_channel, UnboundedReceiver, UnboundedSender};

pub use self::message::*;
use xpc_connection_sys::{
    dispatch_queue_create, xpc_connection_create_mach_service, xpc_connection_resume,
    xpc_connection_send_message, xpc_connection_set_event_handler, xpc_connection_t, xpc_release,
    XPC_CONNECTION_MACH_SERVICE_PRIVILEGED,
};

#[derive(Debug)]
pub struct XpcConnection {
    pub service_name: String,
    connection: Option<xpc_connection_t>,
    unbounded_sender: Option<UnboundedSender<Message>>,
}

impl XpcConnection {
    pub fn new(service_name: &str) -> XpcConnection {
        XpcConnection {
            service_name: service_name.to_owned(),
            connection: None,
            unbounded_sender: None,
        }
    }

    pub fn connect(self: &mut Self) -> UnboundedReceiver<Message> {
        // Start a connection
        let connection = {
            let service_name_cstring =
                CStr::from_bytes_with_nul(self.service_name.as_bytes()).unwrap();
            let label_name = service_name_cstring.as_ptr();
            unsafe {
                xpc_connection_create_mach_service(
                    label_name,
                    dispatch_queue_create(label_name, ptr::null_mut() as *mut _),
                    u64::from(XPC_CONNECTION_MACH_SERVICE_PRIVILEGED),
                )
            }
        };
        self.connection = Some(connection);

        // Create channel to send messages from bindings
        let (unbounded_sender, unbounded_receiver) = unbounded_channel();
        let unbounded_sender_clone = unbounded_sender.clone();

        // Keep the sender so that the channel remains open
        self.unbounded_sender = Some(unbounded_sender);

        // Handle messages received
        let block = ConcreteBlock::new(move |event| {
            unbounded_sender_clone
                .unbounded_send(xpc_object_to_message(event))
                .ok();
        });

        // We must move it from the stack to the heap so that when the libxpc
        // reference count is released we don't double free. This limitation is
        // explained in the blocks crate.
        let block = block.copy();

        unsafe {
            xpc_connection_set_event_handler(connection, block.deref() as *const _ as *mut _);
            xpc_connection_resume(connection);
        }

        // Give back a stream of messages sent
        unbounded_receiver
    }

    pub fn send_message(self: &Self, message: Message) {
        let xpc_object = message_to_xpc_object(message);
        unsafe {
            xpc_connection_send_message(self.connection.unwrap(), xpc_object);
            xpc_release(xpc_object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on_stream;
    use std::collections::HashMap;
    use xpc_connection_sys::xpc_connection_cancel;

    // This also tests that the event handler block is only freed once, as a
    // double free is possible if the block isn't copied on to the heap.
    #[test]
    fn event_handler_receives_error_on_close() {
        let mut con = XpcConnection::new("com.apple.blued\0");
        let mut blocking_stream = block_on_stream(con.connect());

        // Cancelling the connection will cause the event handler to be called
        // with an error message. This will happen under normal circumstances,
        // for example if the service invalidates the connection.
        unsafe {
            xpc_connection_cancel(con.connection.unwrap());
        }

        match blocking_stream.next().unwrap() {
            Message::Error(_) => {}
            _ => panic!("Expected a Message::Error"),
        }
    }

    #[test]
    fn stream_closed_on_drop() {
        let mut con = XpcConnection::new("com.apple.blued\0");
        let mut blocking_stream = block_on_stream(con.connect());

        let message = Message::Dictionary({
            let mut dictionary = HashMap::new();
            dictionary.insert("kCBMsgId\0".to_string(), Message::Int64(1));
            dictionary.insert(
                "kCBMsgArgs\0".to_string(),
                Message::Dictionary({
                    let mut temp = HashMap::new();
                    temp.insert("kCBMsgArgAlert\0".to_string(), Message::Int64(1));
                    temp.insert(
                        "kCBMsgArgName\0".to_string(),
                        Message::String("rust\0".to_string()),
                    );
                    temp
                }),
            );
            dictionary
        });

        // Can get data while the channel is open
        con.send_message(message);

        let mut count = 0;

        loop {
            match blocking_stream.next() {
                Some(Message::Error(error)) => {
                    println!("{:?}", error);
                    break;
                }
                Some(message) => {
                    println!("Received error: {:?}", message);
                    count += 1;

                    // Explained in `event_handler_receives_error_on_close`.
                    unsafe {
                        xpc_connection_cancel(con.connection.unwrap());
                    }
                }
                None => panic!("We should have received a message."),
            }

            // We can't be sure how many buffered messages we'll receive from
            // blued before the connection is cancelled, but it's safe to say
            // it should be less than 5.
            assert!(count < 5);
        }

        drop(con);
        assert!(blocking_stream.next().is_none());
    }
}
