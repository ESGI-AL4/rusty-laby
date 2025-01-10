use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::net::TcpStream;

/// Envoie un message au serveur
pub fn send_message(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let message_bytes = message.as_bytes();
    let size = message_bytes.len() as u32;
    stream.write_u32::<LittleEndian>(size)?;
    stream.write_all(message_bytes)?;
    Ok(())
}

/// Reçoit un message du serveur
pub fn receive_message(stream: &mut TcpStream) -> io::Result<String> {
    let size = stream.read_u32::<LittleEndian>()?;
    let mut buffer = vec![0; size as usize];
    stream.read_exact(&mut buffer)?;
    String::from_utf8(buffer).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid data: {}", e))
    })
}

/// Connecte le client au serveur
pub fn connect_to_server(address: &str) -> io::Result<TcpStream> {
    TcpStream::connect(address)
}

fn main() {}