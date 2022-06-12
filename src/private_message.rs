use super::*;

pub type MessageId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PrivateMessage {
    pub message_id: Option<MessageId>,
    pub sender_id: AccountId,
    pub receiver_id: AccountId,
    pub sender_body: String,
    pub receiver_body: String,
    pub time: u64,
    pub block_height: BlockHeight,
    pub last_message_height: BlockHeight,
}

#[near_bindgen]
impl Contract {
    pub fn new_message(
        &mut self,
        // message_id: Option<MessageId>,
        receiver_id: AccountId,
        sender_body: String,
        receiver_body: String,
    ) {
        let id: String = if env::predecessor_account_id() > receiver_id.clone() {
            env::predecessor_account_id() + "_" + &receiver_id.clone()
        } else {
            receiver_id.clone() + "_" + &env::predecessor_account_id()
        };
        env::log(format!("ID: {}", id).as_bytes());

        match self.get_message(id.clone()) {
            Some(last_message) => {
                let message = PrivateMessage {
                    message_id: Some(id.clone()),
                    sender_id: env::predecessor_account_id(),
                    receiver_id: receiver_id.clone(),
                    sender_body,
                    receiver_body,
                    time: env::block_timestamp(),
                    block_height: env::block_index(),
                    last_message_height: last_message.block_height,
                };

                if last_message.block_height == env::block_index() {
                    env::panic(b"Can't post twice per block");
                }

                self.messages.remove(&id);
                self.messages.insert(&id, &message);
            }

            None => {
                let message = PrivateMessage {
                    message_id: Some(id.clone()),
                    sender_id: env::predecessor_account_id(),
                    receiver_id: receiver_id.clone(),
                    sender_body,
                    receiver_body,
                    time: env::block_timestamp(),
                    block_height: env::block_index(),
                    last_message_height: 0,
                };
                self.messages.insert(&id.clone(), &message);

                let mut sender = self.internal_get_account(&env::predecessor_account_id());
                sender.related_conversations.insert(&id);
                self.internal_set_account(&env::predecessor_account_id(), sender);

                let mut receiver = self.internal_get_account(&receiver_id);
                receiver.related_conversations.insert(&id);
                self.internal_set_account(&receiver_id, receiver);
            }
        }
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<PrivateMessage> {
        self.messages.get(&message_id)
    }
}
