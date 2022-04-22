use aks_egress_checker::{telemetry::configure_telemetry, egress::{load_egress_data, EgressGroup, EgressData}, conncheck};
use clap::{Arg, Command, ArgGroup};

//use crate::telemetry::configure_telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("aks-egress")
        .about("AKS egress checker for outbound connectivity")
        .version("0.1.0")
        .author("Adam Margherio")
        .propagate_version(true)
        .arg(
            Arg::new("loglevel")
                .long("log-level")
                .help("Configures the log level for the output.")
                .long_help("Configures the log level used for output while the application is running. Available options are 'debug', 'info', 'warn', or 'error'. This is equivalent to using the environment variable `RUST_LOG=<value>` and the environment variable will be used if this is not defined.")
                .required(false)
        )
        .arg(
            Arg::new("format")
                .short('o')
                .long("output")
                .help("Output format for results.")
                .possible_values(["json", "table"])
                .default_value("table")
                .default_missing_value("table")
                .required(false)
        )
        .arg(
            Arg::new("egressVersion")
                .long("egress-version")
                .required(false)
                .hide(true)
        )
        .subcommand(
            Command::new("audit")
            .about("Performs an egress audit using the selected policies.")
            .arg(
                Arg::new("egress-groups")
                    .long("g")
                    .help("Egress groups to test for.")
                    .long_help("Egress groups to test for. The list of groups can be found using the 'list-groups' command. If not provided, the groups will default to the AKS required egress and will not check any of the cluster extensions or add-ons.")
                    .multiple_occurrences(true)
                    .required(false)
            )
        )
        .subcommand(
            Command::new("list-groups")
            .arg(
                Arg::new("name")
                    .long("group-name")
                    .help("Name of the egress group to display.")
                    .long_help("The name of a specific egress group to list the details of. This flag can be used multiple times and if it is not specified, it will default to all implemented groups.")
                    .multiple_occurrences(true)
                    .required(false)
            )

        ).get_matches();

    configure_telemetry(&matches);

    // parse the JSON for the egress rules and filter down to enabled groups
    let mut egress_data = load_egress_data().await?;

    println!("{:#?}", matches);

    match matches.subcommand() {
        Some(("list-groups", sub_matches)) => {
            let out = matches.value_of("format").unwrap();

            match out {
                "table" => print_table_output(&egress_data),
                "json" => {
                    let mut group_names: Vec<String> = Vec::new();
                    if sub_matches.is_present("name") {
                        group_names = sub_matches.values_of("name").unwrap().map(|n| String::from(n.to_lowercase())).collect();
                    } else {
                        println!("{:#?}", serde_json::to_string(&egress_data)?)
                    }

                    egress_data.filter_groups(&group_names);

                    println!("{:#?}", serde_json::to_string(&egress_data))

                }
                &_ => println!("{:#?}", egress_data),
            }
        },
        Some(("audit", sub_matches)) => {
            let mut groups: Vec<String> = Vec::new();
            if sub_matches.is_present("egress-groups") {
                sub_matches.values_of("egress-groups").unwrap().for_each(|g| groups.push(String::from(g)));
            }
            
            if !groups.is_empty() {
                egress_data.filter_groups(&groups);

                let mut conn_results = conncheck::check_connectivity(&egress_data.groups).await?;
            }
        },
        Some((&_, _)) => {
            todo!()
        }
        None => {
            todo!()
        }
    }


    Ok(())    
}



fn print_table_output(egress_data: &aks_egress_checker::egress::EgressData) {
    todo!()
}
