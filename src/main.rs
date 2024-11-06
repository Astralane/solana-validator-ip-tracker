mod csv;
mod ip_info;

use crate::ip_info::{IpData, IpInfo};
use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
struct Config {
    rpc_url: String,
    api_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_env("RUST_LOG"))
            .finish(),
    )
    .expect("Failed to set up tracing");
    dotenv::dotenv().ok();

    let config: Config = Figment::new().merge(Env::raw()).extract().unwrap();

    let rpc = RpcClient::new(config.rpc_url);
    let cluster_info = rpc.get_cluster_nodes().await?;
    let leaders = rpc
        .get_leader_schedule(None)
        .await?
        .expect("No leader schedule available");

    //get contact info for validators active in the current epoch
    let mut validators = Vec::new();
    let mut unknown_validators = Vec::new();

    for leader in leaders.iter() {
        let contact_info = cluster_info
            .iter()
            .find(|contact_info| contact_info.pubkey.eq(leader.0));
        if let Some(contact_info) = contact_info {
            validators.push(contact_info.clone());
        } else {
            unknown_validators.push(leader.0);
        }
    }

    info!(
        "Validators active in the current epoch: {:?}",
        validators.len()
    );

    info!(
        "Validators contacts missing in the current epoch: {:?}",
        unknown_validators.len()
    );

    // get ip info for validators
    let ip_info = IpInfo::new(config.api_key);
    let validators_info = ip_info.get_validators_info(&validators).await?;
    //save the data to a file
    csv::save_to_file(validators_info, "validators_info.csv")?;
    Ok(())
}
