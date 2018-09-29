#![feature(tool_lints)]

#[allow(
    dead_code,
    safe_packed_borrows,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    clippy::all
)]
mod xpc_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
mod message;

use std::{ffi::CStr, os::raw::c_void, ptr, mem, boxed::Box};

use block::{Block, ConcreteBlock};

use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver, unbounded as unbounded_channel};

pub use self::message::*;
use self::xpc_sys::{
    dispatch_queue_create, xpc_connection_create_mach_service, xpc_connection_resume,
    xpc_connection_send_message, xpc_connection_set_event_handler, xpc_connection_t, xpc_release,
    XPC_CONNECTION_MACH_SERVICE_PRIVILEGED,
};

#[derive(Debug)]
pub struct XpcConnection {
    pub service_name: String,
    connection: Option<xpc_connection_t>,
    unbounded_sender: Option<*mut UnboundedSender<Message>>,
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

        // Forget the sender so that the channel remains open
        let raw_unbounded_sender = Box::into_raw(Box::new(unbounded_sender));
        self.unbounded_sender = Some(raw_unbounded_sender);
        mem::forget(raw_unbounded_sender);

        // Handle messages received
        let mut rc_block = ConcreteBlock::new(move |event| {
            unbounded_sender_clone.unbounded_send(xpc_object_to_message(event)).unwrap();
        });
        let block = &mut *rc_block;
        unsafe {
            xpc_connection_set_event_handler(connection, block as *mut Block<_, _> as *mut c_void);
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

impl Drop for XpcConnection {
    fn drop(&mut self) {
        if let Some(unbounded_sender) = self.unbounded_sender {
            unsafe {
                drop(Box::from_raw(unbounded_sender))
            }
        }
    }
}
