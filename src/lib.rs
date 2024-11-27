use std::io::{Read, Write};
use std::net::TcpStream;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::{Value, json};

pub fn send_message(stream: &mut TcpStream, message: &str) -> std::io::Result<()> {
    let message_bytes = message.as_bytes();
    let size = message_bytes.len() as u32;

    stream.write_u32::<LittleEndian>(size)?;
    stream.write_all(message_bytes)?;
    Ok(())
}

pub fn receive_message(stream: &mut TcpStream) -> std::io::Result<String> {
    let size = stream.read_u32::<LittleEndian>()?;

    let mut buffer = vec![0; size as usize];
    stream.read_exact(&mut buffer)?;

    let message = String::from_utf8(buffer).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
    })?;
    Ok(message)
}

pub fn connect_to_server(address: &str) -> std::io::Result<TcpStream> {
    TcpStream::connect(address)
}

pub fn build_register_team_message(name: &str) -> Value {
    json!({
        "RegisterTeam": {
            "name": name
        }
    })
}
