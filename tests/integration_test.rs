use std::{
    collections::HashMap,
    thread,
    time,
};
use xpc_connection::{XpcConnection, message::Message};

#[test]
fn it_connects_to_bleud() {
    let mut xpc_connection = XpcConnection::new("com.apple.blued");

    let s = xpc_connection.setup();

    let message = Message::Dictionary({
        let mut dictionary = HashMap::new();
        dictionary.insert("kCBMsgId".to_string(), Message::Int64(1));
        dictionary.insert(
            "kCBMsgArgs".to_string(),
            Message::Dictionary({
                let mut temp = HashMap::new();
                temp.insert("kCBMsgArgAlert".to_string(), Message::Int64(1));
                temp.insert("kCBMsgArgName".to_string(), Message::String("rust".to_string()));
                temp
            })
        );
        dictionary
    });

    xpc_connection.send_message(message);

    thread::sleep(time::Duration::from_secs(5));

    println!("{:?}", s.steal());
}
