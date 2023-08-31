use chrono::Utc;
use ethers::{
    prelude::{abigen, SignerMiddleware},
    providers::{Http, Provider},
    types::Address,
};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use switchboard_evm::sdk::EVMFunctionRunner;
pub use switchboard_utils::reqwest;

abigen!(Receiver, r#"[ function callback(int256, uint256) ]"#,);
static UNUSED_URL: &str = "https://goerli-rollup.arbitrum.io/rpc";

#[derive(Debug, Deserialize)]
pub struct DeribitRespnseInner {
    pub mark_iv: f64,
    pub timestamp: u64,
}
#[derive(Debug, Deserialize)]
pub struct DeribitResponse {
    pub result: DeribitRespnseInner,
}

#[tokio::main(worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Initialize clients ---
    let function_runner = EVMFunctionRunner::new().unwrap();
    let receiver: Address = env!("EXAMPLE_PROGRAM").parse().unwrap();
    let client = SignerMiddleware::new_with_provider_chain(
        Provider::<Http>::try_from(UNUSED_URL).unwrap(),
        function_runner.enclave_wallet.clone(),
    )
    .await?;
    let receiver_contract = Receiver::new(receiver, client.into());

    // --- Logic Below ---
    let url = "https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C";
    let derebit_response: DeribitResponse = reqwest::get(url).await?.json().await?;

    let timestamp = derebit_response.result.timestamp.into();
    let mut mark_iv = Decimal::from_f64(derebit_response.result.mark_iv).unwrap();
    mark_iv.rescale(8);

    // --- Send the callback to the contract with Switchboard verification ---
    let callback = receiver_contract.callback(mark_iv.mantissa().into(), timestamp);
    let expiration = (Utc::now().timestamp() + 120).into();
    let gas_limit = 5_500_000.into();
    function_runner.emit(receiver, expiration, gas_limit, vec![callback])?;
    Ok(())
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
