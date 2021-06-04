use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, AccountId, CryptoHash};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NftContract {
    pub metadata: NFTMetadata,

    pub tokens_by_id: LookupMap<TokenId, Token>,

    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
}

pub type TokenId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<U64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

impl Default for NftContract {
    fn default() -> Self {
        Self {
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            metadata: NFTMetadata {
                spec: "z-nft-1.0.0".to_string(),
                name: "Blockchain Z-days Demo".to_string(),
                symbol: "ZNFT".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        }
    }
}

#[near_bindgen]
impl NftContract {
    pub fn nft_metadata(&self) -> NFTMetadata {
        self.metadata.clone()
    }

    pub fn nft_mint(&mut self, token_id: TokenId, metadata: TokenMetadata) {
        let token = Token {
            token_id,
            owner_id: env::predecessor_account_id(),
            metadata,
        };
        assert!(
            self.tokens_by_id.insert(&token.token_id, &token).is_none(),
            "Token already exists"
        );
        let mut tokens_set = self
            .tokens_per_owner
            .get(&token.owner_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::TokenPerOwnerInner {
                        account_id_hash: hash_account_id(&token.owner_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        tokens_set.insert(&token.token_id);
        self.tokens_per_owner.insert(&token.owner_id, &tokens_set);
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens_by_id.get(&token_id)
    }

    pub fn nft_transfer(&mut self, receiver_id: ValidAccountId, token_id: TokenId) {
        let sender_id = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&token_id).expect("Token not found");

        if sender_id != token.owner_id {
            env::panic(b"Unauthorized");
        }

        assert_ne!(
            &token.owner_id,
            receiver_id.as_ref(),
            "Token owner and receiver should be different"
        );

        log!(
            "Transfer {} from @{} to @{}",
            token_id,
            &token.owner_id,
            receiver_id
        );

        let mut tokens_set = self
            .tokens_per_owner
            .get(&token.owner_id)
            .expect("Token should be owned by the sender");
        tokens_set.remove(&token_id);
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(&token.owner_id);
        } else {
            self.tokens_per_owner.insert(&token.owner_id, &tokens_set);
        }

        let mut tokens_set = self
            .tokens_per_owner
            .get(receiver_id.as_ref())
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::TokenPerOwnerInner {
                        account_id_hash: hash_account_id(receiver_id.as_ref()),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        tokens_set.insert(&token_id);
        self.tokens_per_owner
            .insert(receiver_id.as_ref(), &tokens_set);

        let new_token = Token {
            token_id: token_id.clone(),
            owner_id: receiver_id.as_ref().clone(),
            metadata: token.metadata,
        };
        self.tokens_by_id.insert(&token_id, &new_token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::serde::export::TryFrom;
    use near_sdk::Balance;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn bob() -> AccountId {
        String::from("bob.near")
    }

    fn nft() -> AccountId {
        String::from("nft.near")
    }

    fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit,
            prepaid_gas: 2 * 10u64.pow(14),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    fn helper_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Mochi Rising".to_string()),
            description: Some("Limited edition canvas".to_string()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    fn helper_mint() -> (NftContract, VMContext) {
        let context = get_context(nft(), 10u128.pow(24));
        testing_env!(context.clone());
        let mut contract = NftContract::default();
        contract.nft_mint("0".to_string(), helper_token_metadata());
        (contract, context)
    }

    #[test]
    fn basic_mint_from_owner() {
        helper_mint();
    }

    #[test]
    fn simple_transfer() {
        let (mut contract, context) = helper_mint();
        let token_info = contract.nft_token("0".to_string());
        assert!(token_info.is_some(), "Newly minted token not found");
        testing_env!(context.clone());
        contract.nft_transfer(ValidAccountId::try_from(bob()).unwrap(), "0".to_string());
        assert_eq!(contract.nft_token("0".to_string()).unwrap().owner_id, bob());
    }
}

fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

#[derive(BorshSerialize)]
enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
}
