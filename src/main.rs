mod csv;
mod ip_info;

use crate::ip_info::{IpInfo, ValidatorIpData};
use figment::providers::Env;
use figment::Figment;
use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::response::RpcContactInfo;
use std::collections::{HashMap, HashSet};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
struct Config {
    rpc_url: String,
    api_key: String,
}

#[derive(Clone)]
struct ValidatorInfo {
    pub contact: RpcContactInfo,
    pub stake: Option<u64>,
    pub total_slots: Option<u64>,
    pub ip: Option<String>,
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

    info!("starting.. getting info from rpc");

    let rpc = RpcClient::new(config.rpc_url);
    let cluster_info = rpc.get_cluster_nodes().await?;
    let leaders = rpc
        .get_leader_schedule(None)
        .await?
        .expect("No leader schedule available");

    info!("leader schedule fetched");
    let vote_accounts = rpc.get_vote_accounts().await?;
    let epoch = rpc.get_epoch_info().await?.epoch;
    let vote_accounts_map = vote_accounts
        .current
        .iter()
        .map(|vote_account| (vote_account.node_pubkey.clone(), vote_account))
        .collect::<HashMap<_, _>>();

    info!("fetched vote accounts");

    let leader_slot_count = leaders
        .iter()
        .fold(HashMap::new(), |mut store, (pubkey, slots)| {
            store.insert(pubkey.clone(), slots.iter().count());
            store
        });

    //get contact info for validators active in the current epoch
    let mut validators = Vec::new();
    let mut unknown_validators = Vec::new();

    for leader in leaders.iter() {
        let contact_info = cluster_info
            .iter()
            .find(|contact_info| contact_info.pubkey.eq(leader.0));
        if let Some(contact_info) = contact_info {
            validators.push(ValidatorInfo {
                contact: contact_info.clone(),
                stake: vote_accounts_map
                    .get(&contact_info.pubkey)
                    .map(|v| v.activated_stake),
                total_slots: leader_slot_count
                    .get(&contact_info.pubkey)
                    .map(|v| *v as u64),
                ip: contact_info.tpu_quic.map(|tpu| tpu.ip().to_string()),
            });
        } else {
            unknown_validators.push(leader.0);
        }
    }

    info!(
        "Validators active in the current epoch: {:?}",
        validators.len()
    );

    info!(
        "validators with no tpu quic ports {:?}",
        validators
            .iter()
            .filter(|v| v.contact.tpu_quic.is_none())
            .count()
    );

    //check count of validators with same ip
    let mut ip_count = HashSet::new();
    for validator in validators.iter() {
        if let Some(ip) = &validator.ip {
            ip_count.insert(ip);
        }
    }
    info!(
        "Validators with same ip: {:?}",
        validators.len() - ip_count.len()
    );

    info!(
        "Validators contacts missing in the current epoch: {:?}",
        unknown_validators.len()
    );

    // get ip info for validators
    let ip_info = IpInfo::new(config.api_key);
    let validators_info = ip_info.get_validators_info(&validators).await?;
    //save the data to a file
    csv::save_to_file(
        validators_info,
        &format!("VALIDATORS_FOR_EPOCH_{}.csv", epoch),
    )?;
    Ok(())
}
