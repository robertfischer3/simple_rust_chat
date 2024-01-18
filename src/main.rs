// Import necessary libraries for networking, threading, etc.
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Set constant values
const LOCAL_ADDRESS: &str = "127.0.0.1:6000";
const MESSAGE_SIZE: usize = 64;

fn main() {

    // Create TCP listener to receive incoming connections
    let server = TcpListener::bind(LOCAL_ADDRESS).expect("Listener failed to bind");

    // Set listener to nonblocking mode
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    // Vector to store connected clients
    let mut connected_clients = vec![];

    // Create channel for message passing between threads
    let (sender, receiver) = mpsc::channel::<String>();

    loop {

        // Accept new incoming client connection
        if let Ok((mut socket, client_address)) = server.accept() {

            println!("Client {} connected", client_address);

            // Clone sender for newly connected client
            let sender_copy = sender.clone();

            // Store back reference to connected socket
            connected_clients.push(socket.try_clone().expect("Failed to clone client"));

            // Spawn thread to handle this client
            thread::spawn(move || {

                loop {
                    println!("New thread spawned...");
                    // Buffer to receive message
                    let mut buffer = vec![0; MESSAGE_SIZE];

                    // Read message from socket
                    match socket.read_exact(&mut buffer) {

                        Ok(_) => {

                            // Extract message bytes from buffer
                            let msg_bytes = buffer.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

                            // Decode message
                            let message = String::from_utf8(msg_bytes).expect("Invalid utf8 message");

                            println!("{}: {:?}", client_address, message);

                            // Send message to receiver
                            sender_copy.send(message).expect("Failed to send message to rx");
                        },

                        // Would block error means no messages available
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),

                        // Other errors mean close connection
                        Err(_) => {
                            println!("Closing connection with {}", client_address);
                            break;
                        }
                    }

                    // Short delay between checks
                    sleep();

                }

            });

        }

        // Check for received messages
        if let Ok(message) = receiver.try_recv() {

            // Filter and keep alive clients
            connected_clients = connected_clients.into_iter().filter_map(|mut client| {

                // Make copy of message for client
                let mut buffer = message.clone().into_bytes();

                // Pad buffer to fixed size
                buffer.resize(MESSAGE_SIZE, 0);

                // Send message
                client.write_all(&buffer).map(|_| client).ok()

            }).collect::<Vec<_>>();

        }

        // Short delay between loops
        sleep();

    }

}

// Helper function to sleep thread
fn sleep() {
    thread::sleep(Duration::from_millis(100));
}