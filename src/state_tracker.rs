use crate::error::{Error, ErrorKind};
use crate::tracked_data::TrackedData;

use tokio::net::UnixDatagram;
use tokio::sync::mpsc::Receiver;

/// Receives state updates from functioning parts of any program
/// and proceeds to output them through an UnixDatagram socket.
///
/// The purpose which it full-fills is to allow microservices to communicate
/// the current state of all their functionalities easily through a standardized way.
pub struct StateTracker {
    receiver: Receiver<TrackedData>,
    output_sender: UnixDatagram,
    output_receiver_path: String,
}

impl StateTracker {
    /// Tries to create an instance of StateTracker.
    ///
    /// # Arguments
    /// * `output_sender_path` - Path to the UnixDatagram socket that will send the outputs.
    /// * `output_receiver_path` - Path to the UnixDatagram socket that will receive the outputs.
    /// * `receiver` - Receiver of TrackedData objects.
    pub fn try_new(
        output_sender_path: &str,
        output_receiver_path: &str,
        receiver: Receiver<TrackedData>,
    ) -> Result<Self, Error> {
        let output_sender = match UnixDatagram::bind(output_sender_path) {
            Ok(output) => output,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to bind to output path: {}", error),
                ))
            }
        };

        Ok(Self {
            receiver,
            output_sender,
            output_receiver_path: output_receiver_path.to_string(),
        })
    }

    pub async fn run(mut self) {
        loop {
            match self.receiver.recv().await {
                Some(tracked_data) => {
                    match serde_json::to_vec(&tracked_data) {
                        Ok(serialized_data) => {
                            match self
                                .output_sender
                                .send_to(serialized_data.as_slice(), &self.output_receiver_path)
                                .await
                            {
                                Ok(_) => {
                                    log::info!("sent data to output socket");
                                }
                                Err(error) => {
                                    log::error!("failed to write to output socket: {}", error)
                                }
                            }
                        }
                        Err(error) => log::error!("failed to serialize tracked data: {}", error),
                    };
                }
                None => (),
            }
        }
    }
}

#[cfg(test)]
use crate::state::State;
use std::time::{Duration, SystemTime};
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;

#[tokio::test]
async fn correct_output_retrieved() {
    const SENDER_PATH: &str = "/tmp/cooplan_state_tracker_test_sender.sock";
    const RECEIVER_PATH: &str = "/tmp/cooplan_state_tracker_test_receiver.sock";
    const TEST_ID: &str = "test_id";

    tokio::fs::remove_file(SENDER_PATH).await;
    tokio::fs::remove_file(RECEIVER_PATH).await;

    let (sender, receiver) = tokio::sync::mpsc::channel(1024);

    let output_receiver = tokio::net::UnixDatagram::bind(RECEIVER_PATH).unwrap();

    let state_tracker = StateTracker::try_new(SENDER_PATH, RECEIVER_PATH, receiver).unwrap();

    tokio::spawn(state_tracker.run());

    sender
        .send(TrackedData::new(
            TEST_ID.to_string(),
            State::Idle,
            SystemTime::now(),
        ))
        .await
        .expect("failed to send data");

    let mut buffer = [0; 1024];

    let length = timeout(Duration::from_secs(3), output_receiver.recv(&mut buffer))
        .await
        .unwrap()
        .unwrap();

    let data = &buffer[..length];
    let tracker_data = serde_json::from_slice::<TrackedData>(data).unwrap();

    assert_eq!(tracker_data.id, TEST_ID);
    assert_eq!(tracker_data.state, State::Idle);
}
