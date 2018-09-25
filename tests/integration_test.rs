use std::{
    collections::HashMap,
    thread,
    time,
};

use crossbeam_deque::{fifo, Steal};

use xpc_connection::{XpcConnection, Message};

#[test]
fn it_connects_to_bleud() {
    let (w, s) = fifo();

    let mut xpc_connection = XpcConnection::new("com.apple.blued\0");

    xpc_connection.connect(move |message| {
        w.push(message);
    });

    let message = Message::Dictionary({
        let mut dictionary = HashMap::new();
        dictionary.insert("kCBMsgId\0".to_string(), Message::Int64(1));
        dictionary.insert(
            "kCBMsgArgs\0".to_string(),
            Message::Dictionary({
                let mut temp = HashMap::new();
                temp.insert("kCBMsgArgAlert\0".to_string(), Message::Int64(1));
                temp.insert("kCBMsgArgName\0".to_string(), Message::String("rust\0".to_string()));
                temp
            })
        );
        dictionary
    });

    xpc_connection.send_message(message);

    thread::sleep(time::Duration::from_secs(5));

    if let Steal::Data(data) = s.steal() {
        println!("Got data! {:?}", data);
    } else {
        panic!("Data wasn't found!");
    }
}
