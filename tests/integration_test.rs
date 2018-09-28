use std::collections::HashMap;

use futures::{prelude::*, channel::mpsc::unbounded as unbounded_channel, executor::block_on_stream};

use xpc_connection::{Message, XpcConnection};

#[test]
fn it_connects_to_bleud() {
    let (unbounded_sender, unbounded_receiver) = unbounded_channel();

    let unbounded_sender_ref = unbounded_sender.clone();

    let mut xpc_connection = XpcConnection::new("com.apple.blued\0");

    xpc_connection.connect(move |message| {
        unbounded_sender_ref.unbounded_send(message).unwrap();
    });

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

    xpc_connection.send_message(message);

    println!("Got data! {:?}", block_on_stream(unbounded_receiver.take(1)).next().unwrap());
}
