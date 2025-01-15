use std::io;
use serde_json::Value;
use crate::network;

pub struct ChallengeHandler {
    last_secret: Option<u64>, // Dernier secret reçu
}

impl ChallengeHandler {
    /// Crée un nouveau gestionnaire de challenges
    pub fn new() -> Self {
        ChallengeHandler {
            last_secret: None,
        }
    }

    /// Met à jour le dernier secret reçu
    pub fn update_secret(&mut self, secret: u64) {
        self.last_secret = Some(secret);
        println!("Dernier secret mis à jour : {}", secret);
    }

    /// Gère le challenge SecretSumModulo
    pub fn handle_secret_sum_modulo(
        &self,
        modulo: u64,
        stream: &mut std::net::TcpStream,
        challenge_count: u64,
    ) -> io::Result<()> {
        // Utiliser le dernier secret ou 0 si aucun secret n'a été reçu
        let secret = self.last_secret.unwrap_or(0);

        // Calculer le résultat du modulo
        let result = secret % modulo;
        println!(
            "Challenge #{} - Dernier secret={} % {} = {}",
            challenge_count, secret, modulo, result
        );

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
        println!("Réponse envoyée pour le challenge #{} : {}", challenge_count, response);

        Ok(())
    }

    /// Gère les messages reçus liés aux challenges ou aux secrets
    pub fn process_message(
        &mut self,
        msg: &Value,
        stream: &mut std::net::TcpStream,
        challenge_count: &mut u64,
    ) -> io::Result<()> {
        if let Some(challenge) = msg.get("Challenge") {
            if let Some(secret_sum_modulo) = challenge.get("SecretSumModulo") {
                let modulo = secret_sum_modulo.as_u64().unwrap_or(1);
                *challenge_count += 1; // Incrémenter le compteur de challenges
                self.handle_secret_sum_modulo(modulo, stream, *challenge_count)?;
            }
        } else if let Some(hint) = msg.get("Hint") {
            if let Some(secret) = hint.get("Secret") {
                let secret_value = secret.as_u64().unwrap_or(0);
                self.update_secret(secret_value);
            }
        }
        Ok(())
    }
}
