use std::env;
use std::env::VarError;
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
    //#[serde(rename = "required")]
    //pub required_group: bool,
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
                            // .filter(|grp| grp.required_group)
                            .collect();

                        self.groups = data;
                    } else if s.eq_ignore_ascii_case("optional-egress-only") {
                        let data: Vec<EgressGroup> = self
                            .groups
                            .clone()
                            .into_iter()
                            // .filter(|grp| !grp.required_group)
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

#[tracing::instrument()]
pub async fn load_egress_data() -> Result<EgressData> {
    let mut egress_data = EgressData {
        name: String::from("aks-egress"),
        egress_version: String::from(""),
        groups: Vec::new(),
    };
    let mut path = "";

    let test_val = env::var("LOCAL_TEST");
    match test_val {
        Ok(val) => {
            if val == String::from("true") {
                path = "./egress-data";
            } else {
                log::debug!("LOCAL_TEST is not set to true, using /etc/egress-data as data directory");
                path = "/etc/egress-data";
            }
        }
        _ => {
            log::debug!("LOCAL_TEST is not set, using /etc/egress-data as data directory");
            path = "/etc/egress-data";
        }
    }

    let egress_files = std::fs::read_dir(&path)?;

    egress_files.for_each(|entry| match entry {
        Ok(de) => {
            if de.file_type().unwrap().is_file() {
                let p = de.path();
                log::debug!("Current file name is {:#?}", p.file_name().unwrap());
                if p.extension().unwrap() == "json" {
                    let in_file = std::fs::File::open(p).unwrap();
                    let buf = std::io::BufReader::new(in_file);

                    let eg: EgressGroup = serde_json::from_reader(buf).unwrap();
                    egress_data.groups.push(eg.clone());
                } else {
                    log::warn!("Found a non-JSON file in the egress data.");
                }
            } else {
                log::debug!("Skipping non-file DirEntry {:#?}", de);
            }
        }
        Err(e) => {
            anyhow!(
                "An error occurred while attempting to read the path {} - err: {:?}",
                path,
                e
            );
        }
    });

    Ok(egress_data)
}

#[tracing::instrument(skip(_results, _sm))]
pub async fn print_conn_results(_results: &Vec<EgressGroupResult>, _sm: &ArgMatches) {
    unimplemented!()
}
