use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::event;

/// Queries Azure IMDS for the region of the deployed VMs.
/// Short and sweet, bailing quick if something weird happens.
#[tracing::instrument()]
pub async fn get_region() -> Result<String> {
    event!(tracing::Level::INFO, "Querying IMDS for the Azure region the node is deployed in. This is used to build the URLs for the connectivity checks.");
    let client = reqwest::Client::new();

    let res = client
        .get("http://169.254.169.254/metadata/instance?api-version=2021-02-01")
        .header("Metadata", "true")
        .send()
        .await?;

    if res.status().is_client_error() || res.status().is_server_error() {
        // handle an issue with the client making the request
        todo!(
            "Handle an issue with the client making the request or a server error in the response"
        );
    }

        let res_payload = res.text().await?;

        debug!("IMDS response: {:?}", res_payload);
        let val: Value = serde_json::from_str(res_payload.as_str())?;
        let rval = val["compute"]["location"].as_str().unwrap();
        let region = String::from(rval);

        if region.is_empty() {
            error!("Azure region was not returned in the IMDS response - response received: {:?}", res_payload);
            return Err(anyhow!(
            "Azure region was not returned in the IMDS response - response received: {:?}",
            res_payload
        ));
        }
    }

    Ok(region)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use wiremock::{
        MockServer, Mock, ResponseTemplate,
        matchers::{method, path, header},
    };

    use std::fs;
    use std::path;

    #[tokio::test]
    async fn client_should_extract_region_from_successful_response() {
        let imds_resp = fs::read_to_string(std::path::Path::new("test/imds_resp.json")).unwrap();
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("GET"))
            .and(path("/metadata/instance"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_raw(imds_resp.as_str(), "application/json; charset=utf-8")
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/metadata/instance?api-version=2021-02-01", &mock_server.uri()))
            .header("Metadata", "true")
            .send()
            .await.unwrap();

        let res_payload = res.text().await.unwrap();
        let val: Value = serde_json::from_str(res_payload.as_str()).unwrap();
        let region = val["compute"]["location"].as_str().unwrap();

        assert_eq!("eastus2", region);
    }

    #[tokio::test]
    async fn client_should_retry_on_retriable_error() {
        let imds_resp = fs::read_to_string(std::path::Path::new("test/imds_resp.json")).unwrap();
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("GET"))
            .and(path("/metadata/instance"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(503)
                .set_body_raw(imds_resp.as_str(), "application/json; charset=utf-8")
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/metadata/instance?api-version=2021-02-01", &mock_server.uri()))
            .header("Metadata", "true")
            .send()
            .await.unwrap();
    }

    #[tokio::test]
    async fn client_should_exit_on_client_error_response() {
        let imds_resp = fs::read_to_string(std::path::Path::new("test/imds_resp.json")).unwrap();
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("GET"))
            .and(path("/metadata/instance"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(400)
                .set_body_raw(imds_resp.as_str(), "application/json; charset=utf-8")
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/metadata/instance?api-version=2021-02-01", &mock_server.uri()))
            .header("Metadata", "true")
            .send()
            .await.unwrap();
    }
}