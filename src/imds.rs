use anyhow::{anyhow, Result};
use log::{error, info, debug, warn};
use serde_json::Value;

const MAX_RETRY_COUNT: u8 = 5;

/// Queries Azure IMDS for the region of the deployed VMs.
/// Short and sweet, bailing quick if something weird happens.
#[tracing::instrument()]
pub async fn get_region(imds_host: &str) -> Result<String> {
    info!("Querying IMDS for the Azure region the node is deployed in. This is used to build the URLs for the connectivity checks.");
    let client = reqwest::Client::new();

    let mut is_retriable = true;
    let mut retry_count: u8 = 0;
    let mut region: String = String::from("");

    let imds_url = format!("http://{}/metadata/instance?api-version=2021-12-13", imds_host);

    while is_retriable {
        debug!("Attempting to get IMDS response for VM from {}", imds_url);
        let res = client
            .get(&imds_url)
            .header("Metadata", "true")
            .send()
            .await?;

        if res.status().is_client_error() || res.status().is_server_error() {
            match res.status().as_u16() {
                400 => {
                    error!("IMDS query is missing the Metadata header or ")
                },
                404|405 => {
                    // Resource not found or method not allowed. No point in attempting a retry
                    // so return an error and exit.
                    error!("VM instance metadata was not found or the GET method was not accepted by IMDS.");
                    return Err(anyhow!(
                        "IMDS query failed with status code {}, retry is not applicable. Response body: {:?}",
                        res.status().as_u16(),
                        res.text().await?
                    ));
                },
                410|429|500 => {
                    // These three error codes represent:
                    //  - 410 - Retry after some time with a max backoff of 70 seconds
                    //  - 429 API rate limits have been exceeded (5 requests per second as documented at https://learn.microsoft.com/en-us/azure/virtual-machines/instance-metadata-service?tabs=linux#rate-limiting)
                    //  - 500 - Internal server error, retry after some time but no max backoff specified
                    //
                    // Log the response status we got with appropriate messaging and then try again.
                    warn!(
                        "IMDS returned a {} response, retrying after some time.",
                        res.status().as_u16()
                    );

                    if retry_count > MAX_RETRY_COUNT {
                        error!("Retry count exceeded. Returning error and exiting.");
                        return Err(anyhow!(
                            "Retry count exceeded."
                        ));
                    }

                    // Increment the retry count and calculate the sleep duration (10 * retry count, up to max retry count of 5)
                    retry_count += 1;
                    let sleep_dur = 10 * retry_count;

                    tokio::time::sleep(std::time::Duration::from_secs(sleep_dur.into())).await;
                    continue;
                }
                _ => {
                    // Unexpected HTTP status code from IMDS, log and error out.
                    error!("Unexpected HTTP status code from IMDS, logging error details and exiting.");
                    return Err(anyhow!(
                        "Unexpected HTTP status code from IMDS, logging error details and exiting. Status: {}, response body: {:?}",
                        res.status().as_u16(),
                        res.text().await?
                    ));
                }
            }
            // handle an issue with the client making the request
            if res.status().is_client_error() {
                error!("Bad client request made to IMDS, logging error details and exiting.");
                return Err(anyhow!(
                    "Request to IMDS for instance metadata returned a client error response. Status: {}, response body: {:?}",
                    res.status().as_u16(),
                    res.text().await?
                ));
            }

            if res.status().is_server_error() {
                error!("Bad server response from IMDS, logging error details and exiting.");
                return Err(anyhow!(
                    "Request to IMDS for instance metadata returned a server error response. Status: {}, response body: {:?}",
                    res.status().as_u16(),
                    res.text().await?
                ));
            }
        }

        let res_payload = res.text().await?;
        is_retriable = false;

        debug!("IMDS response: {:?}", res_payload);
        let val: Value = serde_json::from_str(res_payload.as_str())?;
        let rval = val["compute"]["location"].as_str().unwrap();
        region = String::from(rval);

        if region.is_empty() {
            error!("Azure region was not returned in the IMDS response - response received: {:?}", res_payload);
            return Err(anyhow!(
            "Azure region was not returned in the IMDS response - response received: {:?}",
            res_payload
        ));
        }
    }

    info!("IMDS query successful, region: {}", region);

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

        let region = get_region(&mock_server.address().to_string().as_str()).await.unwrap();

        assert_eq!("eastus2", region.as_str());
    }

    #[tokio::test]
    async fn client_should_retry_on_retriable_error() {
        let imds_resp = fs::read_to_string(std::path::Path::new("test/imds_resp.json")).unwrap();
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("GET"))
            .and(path("/metadata/instance"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let region = get_region(&mock_server.address().to_string().as_str()).await;


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