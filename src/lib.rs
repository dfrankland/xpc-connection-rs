#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod message;
pub mod xpc_sys;

use std::{
    ffi::CString,
    os::raw::c_void,
    ptr,
};

use crossbeam::deque::{fifo, Stealer};

use self::{
    message::{Message, xpc_object_to_message, message_to_xpc_object},
    xpc_sys::{
        dispatch_queue_create, xpc_connection_create_mach_service,
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

    pub fn setup(self: &mut Self) -> Stealer<Message> {
        // Setup FIFO async deque
        let (w, s) = fifo();

        // Start a connection
        let service_name_cstring = CString::new(self.service_name.clone()).unwrap();
        let label_name = service_name_cstring.as_ptr();
        let connection = unsafe {
            xpc_connection_create_mach_service(
                label_name,
                dispatch_queue_create(label_name, ptr::null_mut() as *mut _),
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

        s
    }

    pub fn send_message(self: &Self, message: Message) {
        let xpc_object = message_to_xpc_object(message);
        unsafe {
            xpc_connection_send_message(self.connection.unwrap(), xpc_object);
            xpc_release(xpc_object);
        }
    }
}
