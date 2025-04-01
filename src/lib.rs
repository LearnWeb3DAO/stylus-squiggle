#![no_std]
extern crate alloc;

#[cfg(test)]
extern crate std;

use alloc::{string::String, vec, vec::Vec};

mod svg;

use openzeppelin_stylus::token::erc721::{Erc721, Error as Erc721Error};
use stylus_sdk::{
    alloy_primitives::FixedBytes, alloy_primitives::U256, alloy_sol_types::sol, prelude::*,
};

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

/// Declare that `Squiggle` is a contract with the following external methods.
#[public]
#[inherit(Erc721)]
impl Squiggle {
    #[payable]
    pub fn mint(&mut self) -> Result<(), SquiggleError> {
        let mint_price = U256::from(1e17 as u64);
        let value = self.vm().msg_value();
        if value < mint_price {
            return Err(SquiggleError::InsufficientPayment(InsufficientPayment {}));
        }

        let seed = self.generate_seed();
        let token_id = self.total_supply.get();

        self.seeds.setter(token_id).set(seed);
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

    pub fn seed(&self, token_id: U256) -> FixedBytes<32> {
        self.seeds.get(token_id)
    }

    fn generate_seed(&self) -> FixedBytes<32> {
        let mut hash_data = [0u8; 32 + 32 + 32 + 32];
        hash_data[24..32].copy_from_slice(&self.vm().block_number().to_be_bytes());
        hash_data[32 + 24..32 * 2].copy_from_slice(&self.vm().block_timestamp().to_be_bytes());
        hash_data[64 + 12..32 * 3].copy_from_slice(&self.vm().msg_sender().into_array());
        hash_data[96 + 24..32 * 4].copy_from_slice(&self.vm().chain_id().to_be_bytes());
        self.vm().native_keccak256(&hash_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::println;

    #[no_mangle]
    pub unsafe extern "C" fn emit_log(_pointer: *const u8, _len: usize, _: usize) {}

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = Squiggle::from(&vm);

        vm.set_value(U256::from(1e17 as u64));
        let _ = contract.mint();

        let token_uri = contract.token_uri(U256::from(0));
        println!("token uri {}: {}", 0, token_uri);

        let seed = contract.seed(U256::from(0));
        println!("seed {}: {}", 0, seed);
    }
}
