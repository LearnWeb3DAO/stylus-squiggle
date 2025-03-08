#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

mod svg;

use alloy_primitives::FixedBytes;
use alloy_sol_types::sol;
use openzeppelin_stylus::token::erc721::{Erc721, Error as Erc721Error};
use stylus_sdk::abi;
use stylus_sdk::{alloy_primitives::utils::parse_ether, alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    struct Squiggle {
        #[borrow]
        Erc721 erc721;

        uint256 total_supply;

        // Token ID to seeds map
        mapping(uint256 => bytes32) seeds;
    }
}

sol! {
    error InsufficientPayment();
}

#[derive(SolidityError)]
pub enum SquiggleError {
    Erc721(Erc721Error),
    InsufficientPayment(InsufficientPayment),
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
#[inherit(Erc721)]
impl Squiggle {
    #[payable]
    pub fn mint(&mut self) -> Result<(), SquiggleError> {
        let mint_price = parse_ether("0.01").unwrap();
        let value = self.vm().msg_value();
        if value < mint_price {
            return Err(SquiggleError::InsufficientPayment(InsufficientPayment {}));
        }

        let seed = self.generate_seed();
        let token_id = self.total_supply.get();
        let mut setter = self.seeds.setter(token_id);
        setter.set(seed);

        self.total_supply.set(token_id + U256::from(1u8));

        let minter = self.vm().msg_sender();
        self.erc721._mint(minter, token_id)?;
        Ok(())
    }

    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, token_id: U256) -> String {
        let seed = self.seeds.get(token_id);
        let generator = svg::SquiggleGenerator::new(seed);
        let metadata = generator.generate_metadata();

        metadata
    }

    fn generate_seed(&self) -> FixedBytes<32> {
        let hash_data = (
            self.vm().block_coinbase(),
            self.vm().block_number(),
            self.vm().block_timestamp(),
            self.vm().msg_sender(),
            self.vm().chain_id(),
        );

        let encoded_data = abi::encode_params(&hash_data);
        let hash = self.vm().native_keccak256(&encoded_data);

        hash
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Squiggle::from(&vm);

        let _ = contract.mint();
        let token_uri = contract.token_uri(U256::from(0));
        println!("{}", token_uri);

        let seed = contract.seeds.get(U256::from(0));
        println!("{}", seed);
    }
}
