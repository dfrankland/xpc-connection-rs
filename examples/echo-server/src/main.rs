use core_foundation::{base::TCFType, data::CFData};
use futures::stream::StreamExt;
use security_framework::os::macos::code_signing::{Flags, GuestAttributes, SecCode};
use std::{error::Error, ffi::CString};
use xpc_connection::{Message, MessageError, XpcClient, XpcListener};

fn get_code_object_for_client(client: &XpcClient) -> SecCode {
    let token_data = CFData::from_buffer(&client.audit_token());
    let mut attrs = GuestAttributes::new();
    attrs.set_audit_token(token_data.as_concrete_TypeRef());
    SecCode::copy_guest_with_attribues(None, &attrs, Flags::NONE).unwrap()
}

#[allow(dead_code)]
/// This isn't used because we don't sign our builds, but it's a useful example.
fn validate_client_by_code_signing_requirement(client: &XpcClient) -> bool {
    let requirement = "anchor apple".parse().unwrap();

    if get_code_object_for_client(client)
        .check_validity(Flags::NONE, &requirement)
        .is_ok()
    {
        println!("The client's code signature matches");
        return true;
    }

    println!("The client's code signature doesn't match");
    false
}

fn validate_client_by_path(client: &XpcClient) -> bool {
    if get_code_object_for_client(client)
        .path(Flags::NONE)
        .unwrap()
        // It'd be better to use to_path
        .get_string()
        .to_string()
        // This is insecure, it's just so the tests can be run from anywhere
        .contains("message_round_trip")
    {
        println!("The client was validated using its path");
        return true;
    }

    println!("The client's path doesn't contain 'message_round_trip'");
    false
}

async fn handle_client(mut client: XpcClient) {
    println!("New connection");

    if !validate_client_by_path(&client) {
        return;
    }

    loop {
        match client.next().await {
            None => {
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

    println!("The connection was invalidated.");
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
