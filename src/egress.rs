use anyhow::{anyhow, Result};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::conncheck::EgressGroupResult;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EgressData {
    #[serde(rename = "egressVersion")]
    pub egress_version: String,
    pub name: String,
    pub groups: Vec<EgressGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EgressGroup {
    pub enabled: bool,
    pub name: String,
    pub rules: Vec<EgressRule>,
    pub required_group: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EgressRule {
    pub name: String,
    pub dst: String,
    pub protocol: String,
    pub port: String,
    pub description: String,
    #[serde(rename = "requiredPrivate")]
    pub required_private: bool,
    #[serde(rename = "enabled")]
    pub rule_enabled: bool,
}

impl EgressData {
    pub fn filter_groups(&mut self, selected: &Vec<&String>) {
        if selected.len() == 1 {
            match selected.get(0) {
                Some(s) => {
                    if s.eq_ignore_ascii_case("required-egress-only") {
                        let data: Vec<EgressGroup> = self
                            .groups
                            .clone()
                            .into_iter()
                            .filter(|grp| grp.required_group)
                            .collect();

                        self.groups = data;
                    } else if s.eq_ignore_ascii_case("optional-egress-only") {
                        let data: Vec<EgressGroup> = self
                            .groups
                            .clone()
                            .into_iter()
                            .filter(|grp| !grp.required_group)
                            .collect();

                        self.groups = data;
                    } else {
                        ()
                    }
                }
                None => (),
            }
        } else {
            let data: Vec<EgressGroup> = self
                .groups
                .clone()
                .into_iter()
                .filter(|grp| selected.contains(&&grp.name))
                .collect();

            self.groups = data;
        }
    }
}

pub async fn load_egress_data() -> Result<EgressData> {
    let in_file: std::fs::File;

    if std::env::var("LOCAL_TEST")? == String::from("true") {
        in_file = std::fs::File::open("./egress-data/consolidated-egress.json")?;
    } else {
        in_file = std::fs::File::open("/etc/egress-data/consolidated-egress.json")?;
    }

    let buf = std::io::BufReader::new(in_file);
    let data: EgressData = serde_json::from_reader(buf)?;

    Ok(data)
}
