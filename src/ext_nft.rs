use super::*;
use near_sdk::serde_json::json;
use near_sdk::{Gas, ext_contract, PromiseResult};

const DEFAULT_GAS_FEE: Gas = 20_000_000_000_000;

#[ext_contract(ext_self)]
pub trait ExtContract {
    fn on_minted_chest(&mut self, chest_id: ChestId) -> bool;
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn on_minted_chest(&mut self, chest_id: ChestId) -> bool {
        env::log(format!("promise_result_count = {}", env::promise_results_count()).as_bytes());
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut chest = self.chests.get(&chest_id)
                    .expect("Chest not found when call back.");
                chest.minted = true;
                self.chests.insert(&chest_id, &chest);
                
                // let place_id = chest.location.label;
                // let mut chests_at_place = self.chests_per_place.get(&place_id)
                //     .unwrap_or_else(|| panic!("List chest at {} is not found.", place_id));
                // chests_at_place.remove(&chest.id);
                // self.chests_per_place.insert(&place_id, &chests_at_place);
                
                true
            }
            _ => false
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn set_ft_contract (&mut self, ft_contract: AccountId) {
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "You can't set new ft contract id");
        self.ft_contract = ft_contract
    }

/* TEST MINT CHEST 
    #[payable]
    pub fn mint_chest_test(&mut self) -> Promise {
        let chest_id = "abcdefgh".to_string();
        {
            let chest = Chest {
                id: chest_id.clone(),
                sender_account_id: "nttin.testnet".to_string(),
                sender_name: "test tin".to_string(),
                code: "abc".to_string(),
                message: "xyz".to_string(),
                location: Location { label: "aa_bb_cc".to_string(), lat: 1.0, lng: 2.0 },
            };
            
            let place_id = chest.location.clone().label;
            
            self.chests.insert(&chest_id, &chest);
            
            let mut chests_at_place = UnorderedSet::<ChestId>::new(StorageKey::ChestsAtPlace { id: place_id.clone() });
            chests_at_place.insert(&chest_id);
            self.chests_per_place.insert(&place_id, &chests_at_place);
        }

        // --------

        let chest = self.chests.get(&chest_id).expect("Chest not found");
        
        let receiver_id = env::predecessor_account_id(); 

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let token_id = block_timestamp.to_string() + "_" + &chest.sender_account_id + "_" + &receiver_id;

        env::log(b"This function is calling...");
        
        return Promise::new(self.ft_contract.parse().unwrap())
            .function_call(
                b"nft_mint".to_vec(),
                json!({
                    "token_id": token_id,
                    "receiver_id": receiver_id,
                    "token_metadata": {
                        "title": "Rep.run invitation",
                        "description": "You was invited to Rep.run by ".to_string() + &chest.sender_account_id,
                        "media": "https://png.pngtree.com/png-clipart/20190706/original/pngtree-yellow-cartoon-key-material-png-image_4380760.jpg",
                        "copies": 1
                    }
                })
                .to_string()
                .as_bytes()
                .to_vec(),
                10_000_000_000_000_000_000_000, 
                DEFAULT_GAS_FEE 
            )
            .then(ext_self::on_minted_chest(
                    chest_id,
                    &env::current_account_id(),
                    0,
                    DEFAULT_GAS_FEE
            ));
    }
*/
    
    #[payable]
    pub fn mint_chest(&mut self, chest_id: ChestId, account_id: Option<AccountId>) -> Promise {
        let chest = self.chests.get(&chest_id)
            .expect("Chest not found.");

        let receiver_id = account_id.unwrap_or(env::predecessor_account_id()); 

        assert_ne!(
            receiver_id,
            chest.sender_account_id,
            "Can't mint your own chest."
        );

        assert!(
            chest.time + chest.expired_time >= env::block_timestamp(),
            "Can't mint, chest was expired."
        );

        assert!(!chest.minted, "Can't mint the minted chest!");

        let block_timestamp = env::block_timestamp() / 1_000_000_000;
        let token_id = block_timestamp.to_string() + "_" + &chest.sender_account_id + "_invite_" + &receiver_id;
        
        return Promise::new(self.ft_contract.parse().unwrap())
            .function_call(
                b"nft_mint".to_vec(),
                json!({
                    "token_id": token_id,
                    "receiver_id": receiver_id,
                    "token_metadata": {
                        "title": "Rep.run invitation",
                        "description": "You was invited to Rep.run by ".to_string() + &chest.sender_account_id,
                        "media": "https://png.pngtree.com/png-clipart/20190706/original/pngtree-yellow-cartoon-key-material-png-image_4380760.jpg",
                        "copies": 1
                    }
                })
                .to_string()
                .as_bytes()
                .to_vec(),
                100_000_000_000_000_000_000_000, 
                DEFAULT_GAS_FEE 
            )
            .then(ext_self::on_minted_chest(
                    chest_id,
                    &env::current_account_id(),
                    0,
                    DEFAULT_GAS_FEE
            ));
    }
}
