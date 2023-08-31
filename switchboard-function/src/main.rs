use ethers::{
    prelude::{abigen, SignerMiddleware},
    providers::{Http, Provider},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use switchboard_evm::sdk::EVMFunctionRunner;
pub use switchboard_utils::reqwest;
use rust_decimal::prelude::FromPrimitive;

static UNUSED_URL: &str = "https://goerli-rollup.arbitrum.io/rpc";

#[derive(Debug, Deserialize)]
pub struct DeribitRespnseInner {
    pub mark_iv: f64,
    pub timestamp: u64
}
#[derive(Debug, Deserialize)]
pub struct DeribitResponse {
    pub result: DeribitRespnseInner,
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    // define the abi for the callback
    // -- here it's just a function named "callback", expecting the feed names, values, and timestamps
    // -- we also include a view function for getting all feeds
    // running `npx hardhat typechain` will create artifacts for the contract
    // this in particular is found at
    // SwitchboardPushReceiver/artifacts/contracts/src/SwitchboardPushReceiver/Receiver/Receiver.sol/Receiver.json
    abigen!(Receiver, "./src/abi/Receiver.json",);


    // set the gas limit and expiration date
    // -- this is the maximum amount of gas that can be used for the transaction (and it's a lot)
    let gas_limit = 5_500_000;
    let expiration_time_seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        + 120;

    // setup the provider + signer
    let provider = Provider::<Http>::try_from(UNUSED_URL).unwrap();

    let derebit_response: DeribitResponse = reqwest::get(
        "https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C",
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    println!("{:#?}", derebit_response);

    let contract_address = env!("EXAMPLE_PROGRAM")
        .parse::<ethers::types::Address>()
        .unwrap();


    let function_runner = EVMFunctionRunner::new().unwrap();
    let client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, function_runner.enclave_wallet.clone())
            .await
            .unwrap(),
    );

    let receiver_contract = Receiver::new(contract_address, client);
    let mut mark_iv = Decimal::from_f64(derebit_response.result.mark_iv).unwrap();
    mark_iv.rescale(8);
    // send the callback to the contract
    let callback = receiver_contract.callback(mark_iv.mantissa().into(), derebit_response.result.timestamp.into());

    // get the calls from the output results
    let callbacks = vec![callback];

    // Emit the result
    function_runner.emit(
            contract_address,
            expiration_time_seconds.try_into().unwrap(),
            gas_limit.into(),
            callbacks,
        )
        .unwrap();
}

/// Run `cargo test -- --nocapture`
#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test() {
        let derebit_response: DeribitResponse = reqwest::get(
            "https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C",
        )
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        println!("{:#?}", derebit_response);
    }
}
