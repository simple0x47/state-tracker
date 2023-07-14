use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum State {
    Idle,
    Valid,
    Error(String),
}

impl State {
    pub fn is_error(&self) -> bool {
        matches!(self, State::Error(_))
    }
}
