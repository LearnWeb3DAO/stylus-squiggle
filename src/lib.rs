#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

mod erc721;
mod svg;

use alloy_primitives::FixedBytes;
use erc721::InsufficientPayment;
use erc721::{Erc721, Erc721Error, Erc721Params};
use stylus_sdk::abi;
use stylus_sdk::crypto::keccak;
use stylus_sdk::{alloy_primitives::utils::parse_ether, alloy_primitives::U256, prelude::*};

struct SquiggleParams;
impl Erc721Params for SquiggleParams {
    const NAME: &'static str = "StylusSquiggle";
    const SYMBOL: &'static str = "STYLUS-SQUIGGLE";
}

sol_storage! {
    #[entrypoint]
    struct Squiggle {
        #[borrow]
        Erc721<SquiggleParams> erc721;

        // Token ID to seeds map
        mapping(uint256 => bytes32) seeds;
    }
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
#[inherit(Erc721<SquiggleParams>)]
impl Squiggle {
    #[payable]
    pub fn mint(&mut self) -> Result<(), Erc721Error> {
        let mint_price = parse_ether("0.01").unwrap();
        let value = self.vm().msg_value();
        if value < mint_price {
            return Err(Erc721Error::InsufficientPayment(InsufficientPayment {}));
        }

        let seed = self.generate_seed();
        let token_id = self.erc721.total_supply.get();
        let mut setter = self.seeds.setter(token_id);
        setter.set(seed);

        let minter = self.vm().msg_sender();
        self.erc721.mint(minter)?;
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
        let hash = keccak(encoded_data);

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
