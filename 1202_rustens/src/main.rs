use web3::contract::ens::Ens;
use web3::api::Namespace;

#[tokio::main]
async fn main() -> web3::Result<()> {
    let transport = web3::transports::Http::new(
        "http://localhost:8545",
    ).unwrap();

    let ens_name = "vitalik.eth";

    let ens = Ens::new(transport);
    let addr = ens.eth_address(ens_name).await;
    println!("{:?}", addr);

    Ok(())
}
