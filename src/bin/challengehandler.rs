use std::io;
use serde_json::Value;
use crate::bin::network;

pub struct ChallengeHandler {
    last_secrets: Vec<u128>, // Derniers secrets reçus
    modulo: i128, // Dernier secret reçu
}

impl ChallengeHandler {
    /// Crée un nouveau gestionnaire de challenges
    pub fn new(users_number: usize) -> Self {
        ChallengeHandler {
            last_secrets: vec![0; users_number],
            modulo: -1
        }
    }

    // Met à jour le dernier modulo reçu
    // pub fn update_modulo(&mut self, modulo: i64) {
    //     self.modulo = modulo;
        // println!("Dernier modulo mis à jour : {}", modulo as u64);
    // }

    pub fn update_secrets(&mut self, secret: u128, user_id: usize) {
        self.last_secrets[user_id as usize] = secret;
        // println!("Dernier secret mis à jour : {}", secret);
    }

    /// Gère le challenge SecretSumModulo
    pub fn handle_secret_sum_modulo(
        & mut self,
        modulo: u128,
        stream: &mut std::net::TcpStream,
        challenge_count: u64,
    ) -> io::Result<()> {
        // Utiliser le dernier secret ou 0 si aucun secret n'a été reçu
        // let secret = self.last_secret.unwrap_or(0);
        // println!("Secrets: {:?}", self.last_secrets);
        let secrets_sum: u128 = self.last_secrets.iter().sum();

        // Calculer le résultat du modulo
        let result:u128 = secrets_sum % modulo;
        // println!("Solve challenge with ({} + {} + {}) % {} = {}", self.last_secrets[0], self.last_secrets[1], self.last_secrets[2], modulo, result);

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
        // println!("Réponse envoyée pour le challenge #{} : {}", challenge_count, response);
        Ok(())
    }

    /// Gère les messages reçus liés aux challenges ou aux secrets
    // pub fn process_message(
    //     &mut self,
    //     msg: &Value,
    //     stream: &mut std::net::TcpStream,
    //     challenge_count: &mut u64,
    // ) -> io::Result<()> {
    //     if let Some(challenge) = msg.get("Challenge") {
    //         if let Some(secret_sum_modulo) = challenge.get("SecretSumModulo") {
    //             let modulo = secret_sum_modulo.as_u64().unwrap_or(1);
    //             *challenge_count += 1; // Incrémenter le compteur de challenges
    //             self.handle_secret_sum_modulo(modulo, stream, *challenge_count)?;
    //         }
    //     } else if let Some(hint) = msg.get("Hint") {
    //         if let Some(secret) = hint.get("Secret") {
    //             let secret_value = secret.as_u64().unwrap_or(0);
    //             self.update_secret(secret_value);
    //         }
    //     }
    //     Ok(())
    // }

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
                // println!("player {} received challenge SecretSumModulo with modulo {}", user_id, modulo);
                self.handle_secret_sum_modulo(modulo as u128, stream, *challenge_count)?;
            }
        } else if let Some(hint) = msg.get("Hint") {
            if let Some(secret) = hint.get("Secret") {
                let secret_value = secret.as_u64().unwrap_or(0);
                // println!("player {} received hint Secret with value {}", user_id, secret_value);
                self.update_secrets(secret_value as u128, user_id);
            }
        }
        Ok(())
    }
}

fn main() {}
