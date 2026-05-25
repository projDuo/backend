pub mod input;
pub mod output;

use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum AuthEventIn {
    Identify(input::Identify),
    Heartbeat,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum AuthEventOut {
    Authorized(output::Authorized),
    HeartbeatAck,
}