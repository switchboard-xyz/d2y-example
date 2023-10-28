use ethers::prelude::*;
use ethers::{
    abi::AbiDecode,
    prelude::{abigen, SignerMiddleware},
    providers::{Http, Provider},
    types::{Address, U256},
};
use futures::TryFutureExt;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use switchboard_evm::sdk::EVMFunctionRunner;
use switchboard_evm::*;
pub use switchboard_utils::reqwest;

abigen!(Receiver, r#"[ function callback(int256, uint256) ]"#,);
static CLIENT_URL: &str = "https://goerli-rollup.arbitrum.io/rpc";
static RECEIVER: &str = env!("CALLBACK_ADDRESS");

#[derive(Debug, Deserialize)]
pub struct DeribitRespnseInner {
    pub mark_iv: f64,
    pub timestamp: u64,
}
#[derive(Debug, Deserialize)]
pub struct DeribitResponse {
    pub result: DeribitRespnseInner,
}

#[derive(Default, Debug, Clone, EthAbiType, EthAbiCodec)]
struct Params {
    order_id: Address,
    timestamp: U256,
}

impl SbFunctionParameters for Params {
    fn parse(data: &[u8]) -> Self {
        Params::decode(data).unwrap_or_default()
    }
}

#[sb_function(expiration_seconds = 120, gas_limit = 5_500_000)]
async fn sb_function<M: Middleware, S: Signer>(
    client: SignerMiddleware<M, S>,
    _: NoParams,
) -> Result<Vec<FnCall<M, S>>, Error> {
    let receiver: Address = RECEIVER.parse().map_err(|_| Error::ParseError)?;
    let receiver_contract = Receiver::new(receiver, client.into());

    // --- Logic Below ---
    let url =
        "https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C";
    let derebit_response: DeribitResponse = reqwest::get(url)
        .and_then(|r| r.json())
        .await
        .map_err(|_| Error::FetchError)?;

    let timestamp = derebit_response.result.timestamp.into();
    let mut mark_iv =
        Decimal::from_f64(derebit_response.result.mark_iv).ok_or(Error::ParseError)?;
    mark_iv.rescale(8);
    let callback = receiver_contract.callback(mark_iv.mantissa().into(), timestamp);
    // --- Send the callback to the contract with Switchboard verification ---
    Ok(vec![callback])
}

#[sb_error]
enum Error {
    ParseError = 1,
    FetchError,
}

/// Run `cargo test -- --nocapture`
#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test() {
        let derebit_response: DeribitResponse = reqwest::get(
            "https://www.deribit.com/api/v2/public/get_order_book?instrument_name=ETH-29SEP23-2000-C",
        ).and_then(|r| r.json()).await.unwrap();
        println!("{:#?}", derebit_response);
    }
}
