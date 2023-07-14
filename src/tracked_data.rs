use crate::state::State;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Deserialize, Serialize, Debug)]
pub struct TrackedData {
    pub id: String,
    pub state: State,
    pub timestamp: SystemTime,
}

impl TrackedData {
    pub fn new(id: String, state: State, timestamp: SystemTime) -> Self {
        Self {
            id,
            state,
            timestamp,
        }
    }
}

pub fn generate_state_tracking_data(id: &str, state: State) -> TrackedData {
    TrackedData::new(id.to_string(), state, SystemTime::now())
}
