use std::error::Error;
use std::sync::Arc;

use ethers::core::types::Address;
use sha3::Digest;
use crate::api::GetRandomValueResponse;

use crate::config::GenerateOptions;
use crate::ethereum::PythContract;

/// Run the entire random number generation protocol to produce a random number.
pub async fn generate(opts: &GenerateOptions) -> Result<(), Box<dyn Error>> {
    let contract = Arc::new(PythContract::from_opts(&opts.ethereum).await?);

    let user_randomness = rand::random::<[u8; 32]>();
    let provider = opts.provider.parse::<Address>()?;

    // Request a random number on the contract
    let sequence_number = contract.request_wrapper(&provider, &user_randomness, opts.blockhash).await?;
    println!("Requested the random number with sequence number {:#?}", sequence_number);

    // Get the committed value from the provider
    let client = reqwest::Client::new();
    let request_url = client.get(format!("{}/v1/revelation", &opts.url)).query(&[("sequence", sequence_number)]).build()?;
    let resp = client.execute(request_url)
        .await?
        .json::<GetRandomValueResponse>()
        .await?;

    println!("Retrieved the provider's random value. Server response: {:#?}", resp);
    let provider_randomness = resp.value;

    // Submit the provider's and our values to the contract to reveal the random number.
    let random_value = contract.reveal_wrapper(&provider, sequence_number, &user_randomness, &provider_randomness).await?;

    println!("Generated random number: {:#?}", random_value);

    Ok(())
}