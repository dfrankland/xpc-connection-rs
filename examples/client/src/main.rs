use futures::stream::StreamExt;
use std::{collections::HashMap, ffi::CStr, time::Duration};
use xpc_connection::{Message, XpcClient};

#[tokio::main]
async fn main() {
    send_with_reply(c"echo-agent", false).await;
    send_with_reply(c"echo-daemon", true).await;
}

async fn send_with_reply(mach_port_name: &CStr, privileged: bool) {
    println!("Attempting to connect to {mach_port_name:?}");

    let mut client = if privileged {
        XpcClient::connect_privileged(mach_port_name)
    } else {
        XpcClient::connect_unprivileged(mach_port_name)
    };

    let mut dictionary = HashMap::new();
    dictionary.insert(c"hello".to_owned(), Message::Int64(2));

    println!("Sending a message");
    client.send_message(Message::Dictionary(dictionary));

    let message = tokio::time::timeout(Duration::from_secs(1), client.next())
        .await
        .expect("No reply received");

    println!("Client received message {:?}", message);
}
