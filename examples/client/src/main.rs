use futures::stream::StreamExt;
use std::{collections::HashMap, error::Error, ffi::CString};
use xpc_connection::{Message, XpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;

    println!("Attempting to connect to {:?}", mach_port_name);
    let mut client = XpcClient::connect(&mach_port_name);

    let mut dictionary = HashMap::new();
    dictionary.insert(CString::new("hello")?, Message::Int64(2));

    println!("Sending a message");
    client.send_message(Message::Dictionary(dictionary));

    if let Some(message) = client.next().await {
        println!("Client received message {:?}", message);
    }

    Ok(())
}
