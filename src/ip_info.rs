use anyhow::Context;
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_rpc_client_api::response::RpcContactInfo;
use std::collections::HashMap;
use tokio::task::JoinSet;
use tracing::info;

pub const BATCH_SIZE: u32 = 256;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpData {
    pub pubkey: Option<String>,
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
        validators: &[RpcContactInfo],
    ) -> anyhow::Result<Vec<IpData>> {
        let contact_info_map: HashMap<String, RpcContactInfo> = validators
            .iter()
            .filter_map(|v| v.tpu.map(|tpu| (tpu.ip().to_string(), v.clone())))
            .collect();

        let mut progress_bar = progress::Bar::new();
        progress_bar.set_job_title("Getting IP info");
        let percent_per_batch = (BATCH_SIZE * 100) as i32 / (validators.len() as i32);

        //split the validators into batches of size BATCH_SIZE and send requests in parallel
        let mut result = Vec::new();
        let mut join_set = JoinSet::new();

        // let mut join_set = JoinSet::new();
        for batch in validators.chunks(BATCH_SIZE as usize) {
            let ips = batch
                .iter()
                .filter_map(|v| v.tpu.map(|tpu| tpu.ip().to_string()))
                .collect::<Vec<_>>();
            join_set.spawn(Self::get_ip_info_batch(
                self.client.clone(),
                self.api_key.clone(),
                ips,
            ));
        }

        while let Some(res) = join_set.join_next().await {
            let ip_data = res??;
            //show progress
            progress_bar.add_percent(percent_per_batch);
            for mut data in ip_data {
                let Some(ip) = data.ip_address.clone() else {
                    warn!("No IP address found for data: {:?}", data);
                    continue;
                };
                let Some(contact_info) = contact_info_map.get(&ip) else {
                    warn!("No contact info found for IP: {:?}", ip);
                    continue;
                };
                data.pubkey = Some(contact_info.pubkey.to_string());
                result.push(data);
            }
        }
        progress_bar.jobs_done();
        Ok(result)
    }

    async fn get_ip_info_batch(
        client: Client,
        api_key: String,
        ips: Vec<String>,
    ) -> anyhow::Result<Vec<IpData>> {
        let comma_separated_ips = ips.join(",");
        let url = format!(
            "http://api.db-ip.com/v2/{:}/{:}",
            api_key, comma_separated_ips
        );
        let resp = client.get(&url).send().await?;
        let status = resp.status();
        let text = resp.text().await.context("cannot decode text")?;
        if status.is_success() {
            let data: HashMap<String, IpData> = serde_json::from_str(&text)?;
            Ok(data
                .into_iter()
                .map(|(ip, mut v)| {
                    v.ip_address = Some(ip);
                    v
                })
                .collect())
        } else {
            anyhow::bail!("Failed to get IP info: {:?}", text);
        }
    }
}
