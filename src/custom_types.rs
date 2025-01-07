use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Kicked,
    Timeout(u64),
    ConnectionDropped,
    ClientMessage(u64),
    ServerMessage(u64),
    ClientDisconnected(u64),
    NewClient(u64),
    IdAssign(u64),
    Whisper(u64, u64),
    MessageDeserializeError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub body: Option<Vec<u8>>,
}

impl Message {
    pub fn new(message_type: MessageType, body: Option<Vec<u8>>) -> Self {
        Self { message_type, body }
    }
}
