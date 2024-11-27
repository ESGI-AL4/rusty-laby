use rusty_laby::{build_register_team_message, connect_to_server, receive_message, send_message};
// Import from your lib.rs.

fn main() -> std::io::Result<()> {
    let mut stream = connect_to_server("localhost:8778")?;
    println!("Connected to server...");

    let message = build_register_team_message("rusty-octopus");
    send_message(&mut stream, &message.to_string())?;
    println!("Message sent!");

    loop {
        match receive_message(&mut stream) {
            Ok(msg) => println!("Server: {}", msg),
            Err(e) => {
                println!("Failed to receive message: {}", e);
                break;
            }
        }
    }

    Ok(())
}

