use crate::ValidatorInfo;
use anyhow::Context;
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_rpc_client_api::response::RpcContactInfo;
use std::collections::HashMap;
use tokio::task::JoinSet;
use tracing::info;

pub const BATCH_SIZE: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorIpData {
    pub pubkey: Option<String>,
    pub stake: Option<u64>,
    pub total_slots: Option<u64>,
    pub ip_address: Option<String>,
    pub continent_code: Option<String>,
    pub continent_name: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    #[serde(default)]
    pub is_eu_member: bool,
    pub currency_code: Option<String>,
    pub currency_name: Option<String>,
    pub phone_prefix: Option<String>,
    pub state_prov_code: Option<String>,
    pub state_prov: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    #[serde(default)]
    pub geoname_id: u64,
    #[serde(default)]
    pub gmt_offset: i64,
    pub time_zone: Option<String>,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    pub weather_code: Option<String>,
    #[serde(default)]
    pub as_number: u64,
    pub as_name: Option<String>,
    pub isp: Option<String>,
    pub usage_type: Option<String>,
    pub organization: Option<String>,
}

#[derive(Clone)]
pub struct IpInfo {
    client: Client,
    api_key: String,
}
impl IpInfo {
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        Self { client, api_key }
    }

    pub async fn get_validators_info(
        &self,
        validators: &[ValidatorInfo],
    ) -> anyhow::Result<Vec<ValidatorIpData>> {
        info!("Getting IP info for {} validators", validators.len());

        let mut progress_bar = progress::Bar::new();
        progress_bar.set_job_title("Getting IP info");
        let percent_per_batch = (BATCH_SIZE * 100) as i32 / (validators.len() as i32);

        //split the validators into batches of size BATCH_SIZE and send requests in parallel
        let mut result = Vec::new();
        let mut join_set = JoinSet::new();

        for batch in validators.chunks(BATCH_SIZE as usize) {
            join_set.spawn(Self::get_ip_info_batch(
                self.client.clone(),
                self.api_key.clone(),
                batch.to_vec(),
            ));
        }

        while let Some(res) = join_set.join_next().await {
            let ip_data = res??;
            //show progress
            progress_bar.add_percent(percent_per_batch);
            result.extend_from_slice(ip_data.as_slice())
        }
        progress_bar.jobs_done();
        Ok(result)
    }

    async fn get_ip_info_batch(
        client: Client,
        api_key: String,
        validators: Vec<ValidatorInfo>,
    ) -> anyhow::Result<Vec<ValidatorIpData>> {
        let comma_separated_ips = validators
            .iter()
            .filter_map(|i| i.ip.clone())
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "http://api.db-ip.com/v2/{:}/{:}",
            api_key, comma_separated_ips
        );
        let resp = client.get(&url).send().await?;
        let status = resp.status();
        let text = resp.text().await.context("cannot decode text")?;
        if status.is_success() {
            let mut ip_data_map: HashMap<String, ValidatorIpData> = serde_json::from_str(&text)?;
            let mut result = Vec::new();
            for validator in validators {
                let Some(ref ip) = validator.ip else {
                    warn!("cannot find ip for validator {:?}", validator.ip);
                    continue;
                };
                let Some(mut ip_data) = ip_data_map.get(ip).cloned() else {
                    warn!("cannot find ip_data for validator {:?}", validator.ip);
                    continue;
                };
                ip_data.ip_address = validator.ip;
                ip_data.pubkey = Some(validator.contact.pubkey.to_string());
                ip_data.stake = validator.stake;
                ip_data.total_slots = validator.total_slots;
                result.push(ip_data)
            }
            Ok(result)
        } else {
            anyhow::bail!("Failed to get IP info: {:?}", text);
        }
    }
}
