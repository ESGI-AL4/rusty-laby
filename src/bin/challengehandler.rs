use std::io;
use serde_json::Value;
use crate::bin::network;

pub struct ChallengeHandler {
    last_secrets: Vec<u128>, // Derniers secrets reçus
}

impl ChallengeHandler {
    /// Crée un nouveau gestionnaire de challenges
    pub fn new(users_number: usize) -> Self {
        ChallengeHandler {
            last_secrets: vec![0; users_number],
        }
    }

    pub fn update_secrets(&mut self, secret: u128, user_id: usize) {
        self.last_secrets[user_id as usize] = secret;
    }

    /// Gère le challenge SecretSumModulo
    pub fn handle_secret_sum_modulo(
        & mut self,
        modulo: u128,
        stream: &mut std::net::TcpStream,
        challenge_count: u64,
    ) -> io::Result<()> {
        // Utiliser le dernier secret ou 0 si aucun secret n'a été reçu
        let secrets_sum: u128 = self.last_secrets.iter().sum();

        // Calculer le résultat du modulo
        let result:u128 = secrets_sum % modulo;

        // Construire la réponse
        let response = serde_json::json!({
            "Action": {
                "SolveChallenge": {
                    "answer": result.to_string()
                }
            }
        });

        // Envoyer la réponse
        network::send_message(stream, &response.to_string())?;
        Ok(())
    }

    /// Gère les messages reçus liés aux challenges ou aux secrets
    pub fn process_message(
        &mut self,
        msg: &Value,
        stream: &mut std::net::TcpStream,
        challenge_count: &mut u64,
        user_id: usize
    ) -> io::Result<()> {
        if let Some(challenge) = msg.get("Challenge") {
            if let Some(secret_sum_modulo) = challenge.get("SecretSumModulo") {
                let modulo = secret_sum_modulo.as_u64().unwrap_or(1);
                self.handle_secret_sum_modulo(modulo as u128, stream, *challenge_count)?;
            }
        } else if let Some(hint) = msg.get("Hint") {
            if let Some(secret) = hint.get("Secret") {
                let secret_value = secret.as_u64().unwrap_or(0);
                self.update_secrets(secret_value as u128, user_id);
            }
        }
        Ok(())
    }
}

fn main() {}
