use std::net::SocketAddr;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
};

use crate::egress::{EgressGroup, EgressRule};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

pub async fn check_connectivity(
    egress_groups: &Vec<EgressGroup>,
) -> Result<Vec<EgressGroupResult>> {
    let mut res: Vec<EgressGroupResult> = Vec::new();

    // TODO: Make this spawn new threads so we can parallelize these tests
    for group in egress_groups {
        audit_group(&group, &mut res).await;
    }

    Ok(res)
}

async fn audit_group(group: &EgressGroup, res: &mut Vec<EgressGroupResult>) {
    let mut rule_res_vec: Vec<EgressRuleResult> = Vec::new();

    for rule in group.rules.clone() {
        if rule.rule_enabled == false {
            continue;
        } else {
            let dest = format!("{}:{}", rule.dst, rule.port);
            match rule.protocol.as_str() {
                "udp" => {
                    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                    let stream = sock.connect(dest.parse::<SocketAddr>().unwrap()).await;

                    match stream {
                        Err(e) => {
                            let rule_res = EgressRuleResult {
                                name: rule.name,
                                result: ConnCheckResult::Fail,
                                err_msg: Some(e.to_string()),
                            };
                            &rule_res_vec.push(rule_res);
                        }
                        Ok(mut c) => {
                            let rule_res = EgressRuleResult {
                                name: rule.name,
                                result: ConnCheckResult::Pass,
                                err_msg: None,
                            };
                            &rule_res_vec.push(rule_res);

                            //need to close out this socket and connection to reclaim it
                        }
                    }
                }
                "tcp" => {
                    let stream = TcpStream::connect(dest.parse::<SocketAddr>().unwrap()).await;

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

                            c.shutdown();
                        }
                    }
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
