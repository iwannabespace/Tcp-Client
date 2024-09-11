use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    YouAreKicked,
    YouAreTimedout,
    ConnectionDropped,
    ClientSentMessage(usize),
    ClientDisconnected(usize),
    MessageDeserializeError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub body: Option<Vec<u8>>
}
