mod test_target;

use std::net::SocketAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
};

use crate::{
    egress::{EgressGroup, EgressRule},
    imds,
};

pub(crate) const IMDS_HOST: &str = "169.254.169.254";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ConnCheckResult {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EgressGroupResult {
    pub pass_pct: i8,
    pub failed_checks: Option<Vec<EgressRuleResult>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EgressRuleResult {
    pub name: String,
    pub result: ConnCheckResult,
    pub err_msg: Option<String>,
}

#[tracing::instrument(skip(egress_groups))]
pub async fn check_connectivity(
    egress_groups: &Vec<EgressGroup>,
    ccp_fqdn: &str,
) -> Result<Vec<EgressGroupResult>> {
    tracing::debug!("Beginning connectivity checks...");
    let vm_region = imds::get_region(IMDS_HOST).await?; // grab region for use in URLs
    let mut res: Vec<EgressGroupResult> = Vec::new();

    // TODO: Make this spawn new threads so we can parallelize these tests
    for group in egress_groups {
        audit_group(&group, &mut res, ccp_fqdn, vm_region.as_str()).await;
    }

    Ok(res)
}

#[tracing::instrument(skip(group, res))]
async fn audit_group(
    group: &EgressGroup,
    res: &mut Vec<EgressGroupResult>,
    ccp: &str,
    vm_region: &str,
) {
    let mut rule_res_vec: Vec<EgressRuleResult> = Vec::new();

    for rule in group.rules.clone() {
        if rule.rule_enabled == false {
            continue;
        } else {
            let dest = test_target::build_conn_string(&rule, ccp, vm_region).await.unwrap();
            let addr = dest.parse::<SocketAddr>();

            if addr.is_err() {
                let addr_err = addr.unwrap_err();
                log::warn!("Failed to parse address: {}",addr_err.to_string());
                let rule_res = EgressRuleResult {
                    name: rule.name,
                    result: ConnCheckResult::Fail,
                    err_msg: Some(addr_err.to_string()),
                };
                rule_res_vec.push(rule_res);
                continue;
            }

            match rule.protocol.as_str() {
                "udp" => {
                    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                    let stream = sock.connect(addr.unwrap()).await;

                    if stream.is_err() {
                        // handle connectivity error/fail
                        let e = stream.err().unwrap();
                        let rule_res = EgressRuleResult {
                            name: rule.name,
                            result: ConnCheckResult::Fail,
                            err_msg: Some(e.to_string()),
                        };
                        let _ = &rule_res_vec.push(rule_res);
                    } else {
                        // test passed
                        let rule_res = EgressRuleResult {
                            name: rule.name,
                            result: ConnCheckResult::Pass,
                            err_msg: None,
                        };
                        let _ = &rule_res_vec.push(rule_res);
                    }
                }
                "tcp" => {
                    let stream = TcpStream::connect(addr.unwrap()).await;

                    match stream {
                        Err(e) => {
                            let rule_res = EgressRuleResult {
                                name: rule.name,
                                result: ConnCheckResult::Fail,
                                err_msg: Some(e.to_string()),
                            };
                            rule_res_vec.push(rule_res);
                        }
                        Ok(mut c) => {
                            // TODO: Should we even care about successful results here or just assume any rules not marked as a fail
                            // are a pass?
                            let rule_res = EgressRuleResult {
                                name: rule.name,
                                result: ConnCheckResult::Pass,
                                err_msg: None,
                            };
                            rule_res_vec.push(rule_res);

                            c.shutdown().await.unwrap();
                        }
                    }
                }

                "https" | "http" => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            }
        }
    }

    let fail_count = rule_res_vec
        .clone()
        .into_iter()
        .filter(|r| r.result == ConnCheckResult::Fail)
        .collect::<Vec<EgressRuleResult>>()
        .len();

    let passed_percent = (fail_count / group.rules.len()) as i8;

    res.push(EgressGroupResult {
        pass_pct: passed_percent,
        failed_checks: Some(
            rule_res_vec
                .into_iter()
                .filter(|r_res| r_res.result == ConnCheckResult::Fail)
                .collect::<Vec<EgressRuleResult>>(),
        ),
    })
}