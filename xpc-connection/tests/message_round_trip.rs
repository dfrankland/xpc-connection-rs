use futures::{executor::block_on, StreamExt};
use std::{
    collections::HashMap,
    error::Error,
    ffi::CString,
    fs::File,
    os::unix::prelude::{FromRawFd, IntoRawFd, MetadataExt},
    time::{Duration, SystemTime},
};
use xpc_connection::{Message, XpcClient};

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_int64() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let mut output = HashMap::new();
    let key = CString::new("K")?;
    output.insert(key.clone(), Message::Int64(1));
    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Int64(1)) = input {
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[tokio::test]
#[ignore = "This test requires the echo server to be running"]
async fn send_and_receive_string() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let mut output = HashMap::new();
    let key = CString::new("K")?;
    let value = CString::new("V")?;
    output.insert(key.clone(), Message::String(value.clone()));

    con.send_message(Message::Dictionary(output));

    match con.next().await {
        Some(Message::Dictionary(d)) => {
            let input = d.get(&key);
            if let Some(Message::String(s)) = input {
                assert_eq!(s, &value);
                return Ok(());
            }
            panic!("Received unexpected value: {:?}", input);
        }
        Some(message) => panic!("Didn't receive the container dictionary: {:?}", message),
        None => panic!("Didn't receive a response"),
    }
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_dictionary() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let mut output = HashMap::new();
    let outer_key = CString::new("O")?;
    let inner_key = CString::new("I")?;
    output.insert(
        outer_key.clone(),
        Message::Dictionary({
            let mut inner = HashMap::new();
            inner.insert(inner_key.clone(), Message::Int64(1));
            inner
        }),
    );

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(outer_hashmap)) = message {
        let inner_dictionary = outer_hashmap.get(&outer_key);
        if let Some(Message::Dictionary(inner_hashmap)) = inner_dictionary {
            if let Some(Message::Int64(1)) = inner_hashmap.get(&inner_key) {
                return Ok(());
            }

            panic!("Received unexpected value: {:?}", inner_hashmap);
        }

        panic!("Received unexpected value: {:?}", inner_dictionary);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_array() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let mut output = HashMap::new();
    let key = CString::new("K")?;
    output.insert(key.clone(), Message::Array(vec![Message::Int64(1)]));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Array(a)) = input {
            if let Message::Int64(1) = a[0] {
                return Ok(());
            }

            panic!("Received unexpected value: {:?}", a);
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_data() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = vec![0, 1];
    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Data(value.clone()));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Data(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_uint64() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = 0x2d13772f7f30cc5d_u64;

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Uint64(value));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Uint64(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_uuid() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Uuid(value.clone()));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Uuid(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_fd() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let original = File::create("/tmp/a")?;
    let original_inode = original.metadata()?.ino();

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Fd(original.into_raw_fd()));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Fd(v)) = input {
            let new = unsafe { File::from_raw_fd(*v) };
            assert_eq!(original_inode, new.metadata()?.ino());
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_double() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = 1.23456789_f64;

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Double(value));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Double(v)) = input {
            assert!((*v - value).abs() < std::f64::EPSILON);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_bool() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = true;

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Bool(value));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Bool(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_date() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = SystemTime::now();

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Date(value));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Date(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_negative_date() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = SystemTime::UNIX_EPOCH - Duration::from_secs(90);

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Date(value));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Date(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn send_and_receive_null() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Null);

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if matches!(input, Some(Message::Null)) {
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}

#[test]
#[ignore = "This test requires the echo server to be running"]
fn connect_and_disconnect() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;
    let mut con = XpcClient::connect(mach_port_name);

    let key = CString::new("K")?;
    let value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let mut output = HashMap::new();
    output.insert(key.clone(), Message::Uuid(value.clone()));

    con.send_message(Message::Dictionary(output));

    let message = block_on(con.next());
    if let Some(Message::Dictionary(d)) = message {
        let input = d.get(&key);
        if let Some(Message::Uuid(v)) = input {
            assert_eq!(*v, value);
            return Ok(());
        }

        panic!("Received unexpected value: {:?}", input);
    }

    panic!("Didn't receive the container dictionary: {:?}", message);
}
