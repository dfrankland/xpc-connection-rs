use std::{
    collections::HashMap,
    ffi::CStr,
    mem,
    os::raw::{c_char, c_void},
    ptr,
    sync::mpsc::channel,
};

use block::{Block, ConcreteBlock};

use xpc_connection_sys::{
    _xpc_type_activity, _xpc_type_array, _xpc_type_bool, _xpc_type_connection, _xpc_type_data,
    _xpc_type_date, _xpc_type_dictionary, _xpc_type_double, _xpc_type_endpoint, _xpc_type_error,
    _xpc_type_fd, _xpc_type_int64, _xpc_type_null, _xpc_type_shmem, _xpc_type_string,
    _xpc_type_uint64, _xpc_type_uuid, uuid_t, xpc_array_append_value, xpc_array_apply,
    xpc_array_create, xpc_array_get_count, xpc_data_create, xpc_data_get_bytes_ptr,
    xpc_data_get_length, xpc_dictionary_apply, xpc_dictionary_create, xpc_dictionary_get_count,
    xpc_dictionary_set_value, xpc_get_type, xpc_int64_create, xpc_int64_get_value, xpc_object_t,
    xpc_release, xpc_string_create, xpc_string_get_string_ptr, xpc_uuid_create, xpc_uuid_get_bytes,
};

#[derive(Debug, Clone)]
pub enum XpcType {
    Activity,
    Array,
    Bool,
    Connection,
    Data,
    Date,
    Dictionary,
    Double,
    Endpoint,
    Error,
    Fd,
    Int64,
    Null,
    Shmem,
    String,
    Uint64,
    Uuid,
}

macro_rules! check_xpctype {
    ($xpc_object:ident, $xpc_object_type:ident, [ $(($type:ident, $enum:ident),)+ ]) => {
        $(
            if $xpc_object_type == unsafe { &$type as *const _ } {
                return (XpcType::$enum, $xpc_object);
            }
        )+
    }
}

pub fn xpc_object_to_xpctype(xpc_object: xpc_object_t) -> (XpcType, xpc_object_t) {
    let xpc_object_type = unsafe { xpc_get_type(xpc_object) };
    check_xpctype!(
        xpc_object,
        xpc_object_type,
        [
            (_xpc_type_activity, Activity),
            (_xpc_type_array, Array),
            (_xpc_type_bool, Bool),
            (_xpc_type_connection, Connection),
            (_xpc_type_data, Data),
            (_xpc_type_date, Date),
            (_xpc_type_dictionary, Dictionary),
            (_xpc_type_double, Double),
            (_xpc_type_endpoint, Endpoint),
            (_xpc_type_error, Error),
            (_xpc_type_fd, Fd),
            (_xpc_type_int64, Int64),
            (_xpc_type_null, Null),
            (_xpc_type_shmem, Shmem),
            (_xpc_type_string, String),
            (_xpc_type_uint64, Uint64),
            (_xpc_type_uuid, Uuid),
        ]
    );
    panic!("Unknown `xpc` object type!")
}

unsafe fn copy_raw_to_vec(ptr: *const u8, length: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(length);
    vec.set_len(length);
    std::ptr::copy_nonoverlapping(ptr, vec.as_mut_ptr(), length);
    vec
}

#[derive(Debug, Clone)]
pub enum Message {
    Int64(i64),
    String(String),
    Dictionary(HashMap<String, Message>),
    Array(Vec<Message>),
    Data(Vec<u8>),
    Uuid(Vec<u8>),
    Error(MessageError),
}

#[derive(Debug, Clone)]
pub enum MessageError {
    ConnectionInterrupted,
    ConnectionInvalid,
    Unknown,
}

pub fn xpc_object_to_message(xpc_object: xpc_object_t) -> Message {
    match xpc_object_to_xpctype(xpc_object).0 {
        XpcType::Int64 => Message::Int64(unsafe { xpc_int64_get_value(xpc_object) }),
        XpcType::String => Message::String(cstring_to_string(unsafe {
            xpc_string_get_string_ptr(xpc_object)
        })),
        XpcType::Dictionary => {
            let (sender, receiver) = channel();
            let mut rc_block = ConcreteBlock::new(move |key, value| {
                sender
                    .send((cstring_to_string(key), xpc_object_to_message(value)))
                    .unwrap();
                1
            });
            let block = &mut *rc_block;
            unsafe { xpc_dictionary_apply(xpc_object, block as *mut Block<_, _> as *mut c_void) };

            let mut dictionary = HashMap::new();
            for _ in 0..unsafe { xpc_dictionary_get_count(xpc_object) } {
                let (key, value) = receiver.recv().unwrap();
                dictionary.insert(key, value);
            }

            Message::Dictionary(dictionary)
        }
        XpcType::Array => {
            let (sender, receiver) = channel();
            let mut rc_block = ConcreteBlock::new(move |index: usize, value| {
                sender.send((index, xpc_object_to_message(value))).unwrap();
                1
            });
            let block = &mut *rc_block;
            unsafe { xpc_array_apply(xpc_object, block as *mut Block<_, _> as *mut c_void) };

            let mut array = vec![];
            for _ in 0..unsafe { xpc_array_get_count(xpc_object) } {
                let (index, value) = receiver.recv().unwrap();
                array[index] = value;
            }

            Message::Array(array)
        }
        XpcType::Data => unsafe {
            let ptr = xpc_data_get_bytes_ptr(xpc_object) as *mut u8;
            let length = xpc_data_get_length(xpc_object) as usize;
            Message::Data(copy_raw_to_vec(ptr, length))
        },
        XpcType::Uuid => unsafe {
            let ptr = xpc_uuid_get_bytes(xpc_object) as *mut u8;
            let length = mem::size_of::<uuid_t>();
            Message::Uuid(copy_raw_to_vec(ptr, length))
        },
        XpcType::Error => {
            // TODO: Figure out how to return more specific error messages...
            //
            // if xpc_object == _xpc_error_connection_interrupted {
            //     return Message::Error(MessageError::ConnectionInterrupted);
            // }
            //
            // if xpc_object == _xpc_error_connection_invalid {
            //     return Message::Error(MessageError::ConnectionInvalid);
            // }

            Message::Error(MessageError::Unknown)
        }
        _ => panic!("Unmapped `xpc` object type!"),
    }
}

pub fn message_to_xpc_object(message: Message) -> xpc_object_t {
    match message {
        Message::Int64(value) => unsafe { xpc_int64_create(value) },
        Message::String(value) => unsafe {
            let cstr = CStr::from_bytes_with_nul(value.as_bytes()).unwrap();
            let cstr_ptr = cstr.as_ptr();
            xpc_string_create(cstr_ptr)
        },
        Message::Dictionary(values) => {
            let dictionary = unsafe {
                xpc_dictionary_create(ptr::null(), ptr::null_mut() as *mut *mut c_void, 0)
            };
            for (key, value) in values {
                unsafe {
                    let cstr = CStr::from_bytes_with_nul(key.as_bytes()).unwrap();
                    let cstr_ptr = cstr.as_ptr();
                    let xpc_value = message_to_xpc_object(value);
                    xpc_dictionary_set_value(dictionary, cstr_ptr, xpc_value);
                    xpc_release(xpc_value);
                }
            }
            dictionary
        }
        Message::Array(values) => {
            let array = unsafe { xpc_array_create(ptr::null_mut() as *mut *mut _, 0) };
            for value in values {
                unsafe {
                    let xpc_value = message_to_xpc_object(value);
                    xpc_array_append_value(array, xpc_value);
                    xpc_release(xpc_value);
                }
            }
            array
        }
        Message::Data(value) => unsafe { xpc_data_create(value.as_ptr() as *const _, value.len() as u64) },
        Message::Uuid(value) => unsafe {
            let cstr = CStr::from_bytes_with_nul(&value).unwrap();
            let cstr_ptr = cstr.as_ptr();
            xpc_uuid_create(cstr_ptr as *const _)
        },
        Message::Error(_) => panic!("Cannot convert error to `xpc` object!"),
    }
}

fn cstring_to_string(cstring: *const c_char) -> String {
    unsafe { CStr::from_ptr(cstring) }
        .to_str()
        .unwrap()
        .to_owned()
}
