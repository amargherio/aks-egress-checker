use anyhow::{anyhow, Result};
use serde_json::Value;

/// Queries Azure IMDS for the region of the deployed VMs.
/// Short and sweet, bailing quick if something weird happens.
pub async fn get_region() -> Result<String> {
    let client = reqwest::Client::new();

    let res = client
        .get("http://169.254.169.254/metadata/instance?api-version=2021-02-01")
        .header("Metadata", "true")
        .send()
        .await?;

    if res.status().is_client_error() || res.status().is_server_error() {
        // handle an issue with the client making the request
        todo!("Handle an issue with the client making the request or a server error in the response");
    }

    let res_payload = res.text().await?;
    let val: Value = serde_json::from_str(res_payload.as_str())?;
    let region = val["compute"]["location"].to_string();

    if region.is_empty() {
        return Err(anyhow!("Azure region was not returned in the IMDS response - response received: {:?}", res_payload));
    }

    Ok(region)
}
