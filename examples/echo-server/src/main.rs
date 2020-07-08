use futures::stream::StreamExt;
use std::{error::Error, ffi::CString};
use xpc_connection::{Message, MessageError, XpcClient, XpcListener};

async fn handle_client(mut client: XpcClient) {
    println!("New connection");

    loop {
        match client.next().await {
            None => {
                println!("The connection was invalidated.");
                break;
            }
            Some(Message::Error(MessageError::ConnectionInterrupted)) => {
                println!("The connection was interrupted.");
            }
            Some(m) => {
                println!("Server received {:?}", m);
                client.send_message(m);
            }
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mach_port_name = CString::new("com.example.echo")?;

    println!(
        "Waiting for new connections on {:?}",
        mach_port_name.to_string_lossy()
    );

    let mut listener = XpcListener::listen(&mach_port_name);

    while let Some(client) = listener.next().await {
        tokio::spawn(handle_client(client));
    }

    println!("Server is shutting down");

    Ok(())
}
