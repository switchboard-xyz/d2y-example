pub mod coinbase;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use switchboard_common;
use switchboard_evm::sdk::EVMFunctionRunner;
pub use switchboard_utils::reqwest;

use ethers::{
    prelude::{abigen, SignerMiddleware},
    providers::{Http, Provider},
    types::I256,
};
use rand;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::Value;
use std::hash::Hasher;

use std::sync::Arc;
use std::time::{Duration, SystemTime};


#[tokio::main(worker_threads = 12)]
async fn main() {
    // define the abi for the callback
    // -- here it's just a function named "callback", expecting the feed names, values, and timestamps
    // -- we also include a view function for getting all feeds
    // running `npx hardhat typechain` will create artifacts for the contract
    // this in particular is found at
    // SwitchboardPushReceiver/artifacts/contracts/src/SwitchboardPushReceiver/Receiver/Receiver.sol/Receiver.json
    abigen!(Receiver, "./src/abi/Receiver.json",);

    // Generates a new enclave wallet, pulls in relevant environment variables
    let function_runner = EVMFunctionRunner::new().unwrap();

    // set the gas limit and expiration date
    // -- this is the maximum amount of gas that can be used for the transaction (and it's a lot)
    let gas_limit = 5_500_000;
    let est_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    // setup the provider + signer
    let provider = Provider::<Http>::try_from("").unwrap();
    let client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, function_runner.enclave_wallet.clone())
            .await
            .unwrap(),
    );


    let derebit_response = reqwest::get("https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let contract_address = env!("EXAMPLE_PROGRAM")
        .parse::<ethers::types::Address>()
        .unwrap();

    let receiver_contract = Receiver::new(contract_address, client);

    // send the callback to the contract
    let callback = receiver_contract.callback(0, 0);

    // get the calls from the output results
    let mut callbacks = vec![callback];


    // Emit the result
    function_runner
        .emit(
            contract_address,
            expiration_time_seconds.try_into().unwrap(),
            gas_limit.into(),
            callbacks,
        )
        .unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
    }
}
