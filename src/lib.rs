#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod message;
pub mod xpc_sys;

use std::{
    boxed::Box,
    ffi::CString,
    os::raw::c_void,
};

use crossbeam::{
    deque::fifo,
    thread,
};

use self::{
    message::{Message, xpc_object_to_message, message_to_xpc_object},
    xpc_sys::{
        dispatch_queue_attr_s, dispatch_queue_create, xpc_connection_create_mach_service,
        xpc_connection_resume, xpc_connection_send_message, xpc_connection_set_event_handler,
        xpc_connection_t, xpc_object_t, xpc_release,
        XPC_CONNECTION_MACH_SERVICE_PRIVILEGED,
    },
};

pub struct XpcConnection {
    pub service_name: String,
    connection: Option<xpc_connection_t>,
}

impl XpcConnection {
    pub fn new(service_name: &str) -> XpcConnection {
        XpcConnection {
            service_name: service_name.to_owned(),
            connection: None,
        }
    }

    pub fn setup<T: FnMut(Message) + Send + Sync>(self: &mut Self, callback: &mut T) {
        // Setup a worker to asynchronously check for messages
        let (w, s) = fifo();
        thread::scope(|scope| {
            scope.spawn(|| {
                loop {
                    if let Some(message) = s.steal() {
                        callback(message);
                    }
                }
            })
        });

        // Start a connection
        let label_name = CString::new(self.service_name.clone()).unwrap();
        let connection = unsafe {
            xpc_connection_create_mach_service(
                label_name.as_ptr(),
                dispatch_queue_create(
                    label_name.as_ptr(),
                    Box::into_raw(Box::new(dispatch_queue_attr_s { _address: 0 }))
                ),
                u64::from(XPC_CONNECTION_MACH_SERVICE_PRIVILEGED),
            )
        };
        self.connection = Some(connection);

        // Handle messages received
        let mut event_handler: &mut FnMut(xpc_object_t) = &mut move |event| {
            w.push(xpc_object_to_message(event));
            unsafe { xpc_release(event); }
        };
        let event_handler = &mut event_handler;
        unsafe {
            xpc_connection_set_event_handler(connection, event_handler as *mut _ as *mut c_void);
            xpc_connection_resume(connection);
        }
    }

    pub fn send_message(self: &Self, message: Message) {
        let xpc_object = message_to_xpc_object(message);
        unsafe {
            xpc_connection_send_message(self.connection.unwrap(), xpc_object);
            xpc_release(xpc_object);
        }
    }
}
