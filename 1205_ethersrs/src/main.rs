use ethers::{prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Provider::<Http>::try_from("http://localhost:8545/")?;

    let address = provider.resolve_name("vitalik.eth").await?;
    println!("{:?}", address);

    let ens = provider.lookup_address(address).await?;
    println!("{:?}", ens);

    Ok(())
}
