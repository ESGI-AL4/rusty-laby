use std::collections::HashMap;
use serde_json::Value;
use crate::network;
use std::io;

pub struct ChallengeHandler {
    player_secrets: HashMap<String, u64>, // Secrets reçus pour chaque joueur (par nom ou ID)
}

impl ChallengeHandler {
    /// Crée un nouveau gestionnaire de challenges
    pub fn new() -> Self {
        ChallengeHandler {
            player_secrets: HashMap::new(),
        }
    }

    /// Met à jour le secret pour un joueur donné
    pub fn update_secret(&mut self, player_name: &str, secret: u64) {
        self.player_secrets.insert(player_name.to_string(), secret);
        println!("Secret mis à jour pour {}: {}", player_name, secret);
    }

    /// Gère le challenge SecretSumModulo
    pub fn handle_secret_sum_modulo(
        &self,
        modulo: u64,
        player_name: &str,
        stream: &mut std::net::TcpStream,
    ) -> io::Result<()> {
        // Si aucun secret n'a été reçu pour ce joueur, utiliser 0 comme valeur par défaut
        let secret_sum = self
            .player_secrets
            .get(player_name)
            .cloned()
            .unwrap_or(0);

        let result = secret_sum % modulo;
        println!(
            "Calcul SecretSumModulo : somme={} % {} = {}",
            secret_sum, modulo, result
        );

        let response = serde_json::json!({
            "Challenge": {
                "SecretSumModulo": result
            }
        });

        network::send_message(stream, &response.to_string())?;
        println!("Réponse envoyée : {}", response);
        Ok(())
    }

    /// Gère les messages reçus liés aux challenges ou aux secrets
    pub fn process_message(
        &mut self,
        msg: &Value,
        player_name: &str,
        stream: &mut std::net::TcpStream,
    ) -> io::Result<()> {
        if let Some(challenge) = msg.get("Challenge") {
            if let Some(secret_sum_modulo) = challenge.get("SecretSumModulo") {
                let modulo = secret_sum_modulo.as_u64().unwrap_or(1);
                // Répondre au challenge
                self.handle_secret_sum_modulo(modulo, player_name, stream)?;
            }
        } else if let Some(secret) = msg.get("Secret") {
            let secret_value = secret.as_u64().unwrap_or(0);
            self.update_secret(player_name, secret_value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;

    #[test]
    fn test_secret_sum_modulo_single_player_first_challenge() {
        let mut handler = ChallengeHandler::new();

        // Aucun secret reçu, somme par défaut = 0
        let modulo = 1000;
        let player_name = "Player1";
        let secret_sum: u64 = handler.player_secrets.get(player_name).cloned().unwrap_or(0);
        let result = secret_sum % modulo;
        assert_eq!(result, 0 % 1000);
    }

    #[test]
    fn test_secret_sum_modulo_after_secret_received() {
        let mut handler = ChallengeHandler::new();

        // Simuler la réception d'un secret pour un joueur
        handler.update_secret("Player1", 4518741838142591602);

        // Simuler le calcul du modulo
        let modulo = 1000;
        let secret_sum: u64 = handler.player_secrets.get("Player1").cloned().unwrap_or(0);
        let result = secret_sum % modulo;
        assert_eq!(result, 4518741838142591602 % 1000);
    }
}
