#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod xpc_sys;
pub mod message;

use std::{
    ffi::CString,
    boxed::Box,
    os::raw::c_void,
};

use self::{
    xpc_sys::{
        dispatch_queue_attr_s, dispatch_queue_create,
        xpc_connection_t, xpc_connection_create_mach_service, XPC_CONNECTION_MACH_SERVICE_PRIVILEGED,
        xpc_connection_set_event_handler, xpc_connection_resume,
        xpc_object_t, xpc_retain,
        xpc_connection_send_message, xpc_release,
    },
    message::Message,
};

pub struct XpcConnection<T: Fn(Message) -> () + Send> {
    service_name: String,
    callback: Option<T>,
    connection: Option<xpc_connection_t>,
}

impl<T: Fn(Message) -> () + Send> XpcConnection<T> {
    pub fn new(service_name: &str) -> XpcConnection<T> {
        XpcConnection {
            service_name: service_name.to_owned(),
            callback: None,
            connection: None,
        }
    }

    pub fn set_callback(self: &mut Self, callback: T) {
        self.callback = Some(callback);
    }

    pub fn setup(self: &mut Self) {
        let label_name = CString::new(self.service_name.clone()).unwrap();
        let attr = Box::into_raw(Box::new(dispatch_queue_attr_s { _address: 0 }));
        let dispatch_queue = unsafe { dispatch_queue_create(label_name.as_ptr(), attr) };

        let connection = unsafe {
            xpc_connection_create_mach_service(
                label_name.as_ptr(),
                dispatch_queue,
                u64::from(XPC_CONNECTION_MACH_SERVICE_PRIVILEGED),
            )
        };

        self.connection = Some(connection);

        let mut cb: &mut FnMut(xpc_object_t) = &mut move |event| {
            unsafe {
                xpc_retain(event);
                // TODO: Call one of the callbacks
                xpc_release(event);
            }
        };
        let cb = &mut cb;

        unsafe {
            xpc_connection_set_event_handler(connection, cb as *mut _ as *mut c_void);
            xpc_connection_resume(connection);
        }
    }

    pub fn send_message(self: &Self, message: xpc_object_t) {
        unsafe {
            xpc_connection_send_message(self.connection.unwrap(), message);
            xpc_release(message);
        }
    }
}
