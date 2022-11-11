use anyhow::{anyhow, Result};
use serde_json::Value;

/// Queries Azure IMDS for the region of the deployed VMs.
/// Short and sweet, bailing quick if something weird happens.
pub async fn get_region() -> Result<&'static str> {
    let client = reqwest::Client::new();

    let res = client
        .get("http://169.254.169.254/metadata/instance?api-version=2021-02-01")
        .header("Metadata", "true")
        .send()
        .await?;

    if res.status().is_client_error() || res.status().is_server_error() {
        // handle an issue with the client making the request
    }

    let res_payload = res.text().await?;
    let val: Value = serde_json::from_str(res_payload.as_str())?;
    let region: &str = val["compute"]["location"].as_str().unwrap();

    if !region.is_empty() {
        return Ok(region);
    } else {
        unimplemented!("Need to implement error handling for an empty region returned from IMDS")
    }
}
