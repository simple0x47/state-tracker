use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StateTrackingConfig {
    pub state_output_sender_path: String,
    pub state_output_receiver_path: String,

    pub state_sender_interval_in_seconds: u64,
}
