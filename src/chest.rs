use super::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Location {
    pub label: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum ChestType {
    Standard,
    Image { url: String },
    Video { url: String },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Chest {
    pub id: ChestId,
    pub sender_account_id: AccountId,
    pub sender_name: String,
    pub code: String,
    pub message: String,
    pub location: Location,
    pub time: u64,
    pub expired_time: u64,
    pub minted: bool,
    pub chest_type: ChestType,
}

#[near_bindgen]
impl Contract {
    /*
     * view method
     */
    pub fn get_all_chests_by_key(&self, key: String) -> Vec<Chest> {
        match self.chests_per_place.get(&key) {
            Some(chests) => chests
                .iter()
                .map(|id| self.chests.get(&id).expect("Chest not found"))
                .collect(),
            None => vec![],
        }
    }

    pub fn get_active_chests_by_key(&self, key: String) -> Vec<Chest> {
        let now: u64 = env::block_timestamp();
        match self.chests_per_place.get(&key) {
            Some(chests) => chests
                .iter()
                .map(|id| self.chests.get(&id).expect("Chest not found"))
                .filter(|chest| now - chest.time < chest.expired_time)
                .collect(),
            None => vec![],
        }
    }

    pub fn get_chests_by_account_id(&self, account_id: ValidAccountId) -> Vec<Chest> {
        if let Some(account) = self.internal_get_account_optional(&account_id.into()) {
            return account
                .chests
                .iter()
                .map(|id| self.chests.get(id).expect("Chest ID not found"))
                .collect();
        }
        vec![]
    }

    pub fn get_all_chests(&self) -> Vec<Chest> {
        self.place_ids
            .iter()
            .flat_map(|place_id| match self.chests_per_place.get(&place_id) {
                Some(chests) => chests
                    .iter()
                    .map(|id| self.chests.get(&id).expect("Chest not found"))
                    .collect(),
                None => vec![],
            })
            .collect()
    }

    pub fn get_all_place_id(&self) -> Vec<PlaceId> {
        self.place_ids.iter().collect()
    }

    /*
     * Modify method
     */
    #[payable]
    pub fn place_message_chest(
        &mut self,
        name: String,
        code: String,
        message: String,
        chest_type: Option<ChestType>,
        location: Location,
        expired_time: Option<u64>,
    ) -> Chest {
        assert!(
            message.len() <= MAX_MESSAGE_LENGTH,
            "Can not make a chest with message have more than {} characters",
            MAX_MESSAGE_LENGTH
        );

        assert!(
            env::attached_deposit() == 1_000_000_000_000_000_000_000_000,
            "Must attach exact 1 near"
        );

        let account_id = env::predecessor_account_id();
        let _storage_update = self.new_storage_update(account_id.clone());
        let _ = self.internal_get_account(&account_id);

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let chest_id = block_timestamp.to_string() + "_" + &env::predecessor_account_id();

        let place_id = location.clone().label;

        let chest = Chest {
            id: chest_id.clone(),
            sender_account_id: account_id.clone(),
            sender_name: name,
            code,
            message,
            chest_type: match chest_type {
                Some(chest_type_unwrap) => chest_type_unwrap,
                None => ChestType::Standard,
            },
            location,
            time: env::block_timestamp(),
            expired_time: DEFAULT_EXPIRE_TIME,
            minted: false,
        };

        // Insert to chests
        self.chests.insert(&chest_id, &chest);

        // Insert to chests per place
        let mut chests_at_place = match self.chests_per_place.get(&place_id.clone()) {
            Some(place_ids) => place_ids,
            None => {
                self.place_ids.insert(&place_id);
                UnorderedSet::<ChestId>::new(StorageKey::ChestsAtPlace {
                    id: place_id.clone(),
                })
            }
        };
        chests_at_place.insert(&chest_id);
        self.chests_per_place.insert(&place_id, &chests_at_place);

        chest
    }

    #[payable]
    pub fn place_chest(
        &mut self,
        name: String,
        code: String,
        message: String,
        chest_type: Option<ChestType>,
        location: Location,
        expired_time: Option<u64>,
    ) -> Chest {
        assert!(
            message.len() <= MAX_MESSAGE_LENGTH,
            "Can not make a chest with message have more than {} characters",
            MAX_MESSAGE_LENGTH
        );

        let account_id = env::predecessor_account_id();
        let _storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);

        assert!(
            account.chests.len() < 4,
            "Max num of chest you can place is 4, You can not place more"
        );

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let chest_id = block_timestamp.to_string() + "_" + &env::predecessor_account_id();

        let place_id = location.clone().label;

        let chest = Chest {
            id: chest_id.clone(),
            sender_account_id: account_id.clone(),
            sender_name: name,
            code,
            message,
            chest_type: match chest_type {
                Some(chest_type_unwrap) => chest_type_unwrap,
                None => ChestType::Standard,
            },
            location,
            time: env::block_timestamp(),
            expired_time: DEFAULT_EXPIRE_TIME,
            minted: false,
        };

        // Add chest_id to account info
        account.chests.push(chest_id.clone());
        self.internal_set_account(&account_id, account);

        // Insert to chests
        self.chests.insert(&chest_id, &chest);

        // Insert to chests per place
        let mut chests_at_place = match self.chests_per_place.get(&place_id.clone()) {
            Some(place_ids) => place_ids,
            None => {
                self.place_ids.insert(&place_id);
                UnorderedSet::<ChestId>::new(StorageKey::ChestsAtPlace {
                    id: place_id.clone(),
                })
            }
        };
        chests_at_place.insert(&chest_id);
        self.chests_per_place.insert(&place_id, &chests_at_place);

        chest
    }

    #[payable]
    pub fn replace_chest_by_chest_id(
        &mut self,
        chest_id: ChestId,
        name: String,
        code: String,
        message: String,
        chest_type: Option<ChestType>,
        location: Location,
        expired_time: Option<u64>,
    ) -> Chest {
        let old_chest = self.chests.get(&chest_id).expect("Chest not exists!");

        assert!(
            old_chest.time + old_chest.expired_time < env::block_timestamp(),
            "Chest still active, you just can replace when it expired."
        );

        assert!(
            message.len() <= MAX_MESSAGE_LENGTH,
            "Can not make a chest with message have more than {} characters",
            MAX_MESSAGE_LENGTH
        );

        let account_id = env::predecessor_account_id();
        let _storage_update = self.new_storage_update(account_id.clone());
        let mut account = self.internal_get_account(&account_id);

        let index = account
            .chests
            .iter()
            .position(|v| chest_id == v.clone())
            .expect("You doesn't own this chest.");

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let new_chest_id = block_timestamp.to_string() + "_" + &env::predecessor_account_id();

        let place_id = location.clone().label;

        let chest = Chest {
            id: new_chest_id.clone(),
            sender_account_id: account_id.clone(),
            sender_name: name,
            code,
            message,
            chest_type: match chest_type {
                Some(chest_type_unwrap) => chest_type_unwrap,
                None => ChestType::Standard,
            },
            location,
            time: env::block_timestamp(),
            expired_time: DEFAULT_EXPIRE_TIME,
            minted: false,
        };

        // Add chest_id to account info
        std::mem::replace(&mut account.chests[index], new_chest_id.clone());
        self.internal_set_account(&account_id, account);

        // Insert to chests
        self.chests.insert(&new_chest_id, &chest);

        // Insert to chests per place
        let mut chests_at_place = match self.chests_per_place.get(&place_id.clone()) {
            Some(place_ids) => place_ids,
            None => {
                self.place_ids.insert(&place_id);
                UnorderedSet::<ChestId>::new(StorageKey::ChestsAtPlace {
                    id: place_id.clone(),
                })
            }
        };
        chests_at_place.insert(&new_chest_id);
        self.chests_per_place.insert(&place_id, &chests_at_place);

        chest
    }

    #[payable]
    pub fn edit_chest(&mut self, chest_id: ChestId, new_location: Location) -> Chest {
        let account_id = env::predecessor_account_id();
        self.internal_get_account(&account_id);
        let mut chest = self.chests.get(&chest_id).expect("Chest not found");

        assert!(
            account_id == chest.sender_account_id || self.is_admin(account_id),
            "Just owner or admin can edit chest information"
        );

        // Change place_id
        if chest.location.label != new_location.label {
            // Remove old place
            let old_place_id = chest.location.label;
            let mut list_chests_at_old_place = self
                .chests_per_place
                .get(&old_place_id)
                .expect("Place_id not found");
            list_chests_at_old_place.remove(&chest_id.clone());
            if list_chests_at_old_place.len() == 0 {
                self.place_ids.remove(&old_place_id);
                self.chests_per_place.remove(&old_place_id);
            } else {
                self.chests_per_place
                    .insert(&old_place_id, &list_chests_at_old_place);
            }

            // Add new place
            let new_place_id = new_location.label.clone();
            let mut list_chests_at_new_place =
                match self.chests_per_place.get(&new_place_id.clone()) {
                    Some(chest_ids) => chest_ids,
                    None => {
                        self.place_ids.insert(&new_place_id.clone());
                        UnorderedSet::<ChestId>::new(StorageKey::ChestsAtPlace {
                            id: new_place_id.clone(),
                        })
                    }
                };
            list_chests_at_new_place.insert(&chest_id);
            self.chests_per_place
                .insert(&new_place_id, &list_chests_at_new_place);
        }

        // Replace new chest
        chest.location = new_location;
        self.chests.insert(&chest_id, &chest);

        chest
    }

    #[payable]
    pub fn delete_chest(&mut self, chest_id: ChestId) -> Chest {
        let chest = self.chests.get(&chest_id).expect("Chest id not found");
        let account_id = env::predecessor_account_id();

        assert!(
            account_id == chest.sender_account_id || self.is_admin(account_id.clone()),
            "Just owner or admin can edit chest information"
        );

        // Remove in chests
        self.chests.remove(&chest_id);

        // Remove chest in account info
        let mut account = self.internal_get_account(&account_id.clone());
        let index = account.chests.iter().position(|x| *x == chest_id).unwrap();
        account.chests.remove(index);
        self.internal_set_account(&account_id, account);

        // Remove chest in chest per place
        let mut list_chests_at_place = self
            .chests_per_place
            .get(&chest.location.label)
            .expect("Place id not found");
        list_chests_at_place.remove(&chest_id);

        // Remove place if need
        if list_chests_at_place.len() == 0 {
            self.place_ids.remove(&chest_id);
            self.chests_per_place.remove(&chest.location.label);
        } else {
            self.chests_per_place
                .insert(&chest.location.label, &list_chests_at_place);
        }

        chest
    }
}
