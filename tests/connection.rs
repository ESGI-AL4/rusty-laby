#[cfg(test)]
mod tests {
    use rusty_laby::{build_register_team_message, connect_to_server, receive_message, send_message}

    const ADDRESS: String = String::from("localhost:8778");
    /// Test for `send_message`.
    #[test]
    fn test_send_message() -> std::io::Result<()> {
        todo!()
    }
}
