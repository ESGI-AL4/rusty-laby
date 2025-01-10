use serde_json::Value;

pub fn parse_json(msg: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(msg)
}

pub fn extract_registration_token(json: &Value) -> Option<&str> {
    json.get("RegisterTeamResult")?
        .get("Ok")?
        .get("registration_token")?
        .as_str()
}

fn main() {}