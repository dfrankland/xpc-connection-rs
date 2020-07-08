use futures::executor::block_on;
use futures::StreamExt;
use std::collections::HashMap;
use xpc_connection::{Message, XpcConnection};

/// FIXME: Outgoing strings, both as dictionary keys and as string values, are
/// expected to be explicitly NULL terminated, but incoming strings do not
/// contain the NULL. This prevents us from being able to compare outgoing and
/// incoming container dictionaries directly.
/// This will also make writing an XPC listener a little more confusing as
/// messages have two different behaviours depending on the context.

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_int64() {
    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let mut output = HashMap::new();
    output.insert("Value\0".to_string(), Message::Int64(1));
    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(d) = block_on(receiver.next()).unwrap() {
        let input = d.get("Value").unwrap();
        if let Message::Int64(1) = input {
            return;
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary");
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_string() {
    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let mut output = HashMap::new();
    output.insert(
        "Value\0".to_string(),
        Message::String(String::from("Hello\0")),
    );

    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(d) = block_on(receiver.next()).unwrap() {
        let input = d.get("Value").unwrap();
        if let Message::String(s) = input {
            assert_eq!(*s, "Hello");
            return;
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary");
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_dictionary() {
    // Dictionary is mostly covered by the other tests, but testing nested
    // dictionaries is useful.

    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let mut output = HashMap::new();
    output.insert(
        "Outer\0".to_string(),
        Message::Dictionary({
            let mut inner = HashMap::new();
            inner.insert("Value\0".to_string(), Message::Int64(1));
            inner
        }),
    );

    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(outer_hashmap) = block_on(receiver.next()).unwrap() {
        let inner_dictionary = outer_hashmap.get("Outer").unwrap();
        if let Message::Dictionary(inner_hashmap) = inner_dictionary {
            if let Message::Int64(1) = inner_hashmap.get("Value").unwrap() {
                return;
            }

            panic!("Received unexpected value: {:?}", inner_hashmap);
        }

        panic!("Received unexpected value: {:?}", inner_dictionary);
    }

    panic!("Didn't receive the container dictionary");
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_array() {
    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let mut output = HashMap::new();
    output.insert(
        "Value\0".to_string(),
        Message::Array(vec![Message::Int64(1)]),
    );

    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(d) = block_on(receiver.next()).unwrap() {
        let input = d.get("Value").unwrap();
        if let Message::Array(a) = input {
            if let Message::Int64(1) = a[0] {
                return;
            }

            panic!("Received unexpected value: {:?}", a);
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary");
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_data() {
    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let value = vec![0, 1];
    let mut output = HashMap::new();
    output.insert("Value\0".to_string(), Message::Data(value.clone()));

    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(d) = block_on(receiver.next()).unwrap() {
        let input = d.get("Value").unwrap();
        if let Message::Data(v) = input {
            assert_eq!(*v, value);
            return;
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary");
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_uuid() {
    let mut con = XpcConnection::new("com.example.echo\0");
    let mut receiver = con.connect();

    let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let mut output = HashMap::new();
    output.insert("Value\0".to_string(), Message::Uuid(value.clone()));

    con.send_message(Message::Dictionary(output.clone()));

    if let Message::Dictionary(d) = block_on(receiver.next()).unwrap() {
        let input = d.get("Value").unwrap();
        if let Message::Uuid(v) = input {
            assert_eq!(*v, value);
            return;
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary");
}
