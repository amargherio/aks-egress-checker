use aks_egress_checker::{
    conncheck,
    egress::{load_egress_data, EgressData, EgressGroup},
    telemetry::configure_telemetry,
};
use clap::{builder::PossibleValue, Arg, ArgAction, ArgGroup, Command, ArgMatches};

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
                .short('v')
                .help("Configures the log level for the output.")
                .long_help("Configures the log level used for output while the application is running. Available options are 'debug', 'info', 'warn', or 'error'. This is equivalent to using the environment variable `RUST_LOG=<value>` and the environment variable will be used if this is not defined.")
                .required(false)
        )
        .arg(
            Arg::new("format")
                .short('o')
                .long("output")
                .help("Output format for results.")
                .value_parser([
                    PossibleValue::new("json"),
                    PossibleValue::new("table"),
                ])
                .default_value("table")
                .default_missing_value("table")
                .required(false)
        )
        .arg(
            Arg::new("output-file-path")
                .short("f")
                .long("output-file")
                .help("File path to save the results at. Recommended for JSON output.")
                .required("false")
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
                    .long("group-name")
                    .short('g')
                    .help("Egress groups to test for.")
                    .long_help(
                        "Egress groups to test for. The list of groups can be found using the 'list-groups' command.
                        This can be used multiple times to indicate the set of egress connectivity to check.
                        
                        If not provided, the groups will default to the AKS required egress and will not check any of the cluster extensions or add-ons.")
                    .action(ArgAction::Append)
                    .required(false)
            )
            .arg(
                Arg::new("ccp-fqdn")
                    .long("ccp-fqdn")
                    .help("Fully qualified domain name for the AKS control plane.")
                    .long_help("The fully qualified domain name for the AKS control plane. This can be found by viewing the properties of your AKS cluster in Azure Portal, by checking the results of `az aks show` for the cluster, or by viewing the kubeconfig for the cluster. If no CCP URL is provided, the check will attempt to use kubernetes.default.svc.cluster.local, although this may lead to incorrect results.")
                    .required(true)
            )
        )
        .subcommand(
            Command::new("list-groups")
            .arg(
                Arg::new("name")
                    .long("group-name")
                    .short('g')
                    .help("Name of the egress group to display.")
                    .long_help(
                        "The name of a specific egress group to list the details of. 
                        This flag can be used multiple times and if it is not specified, it will default to all implemented groups.
                        
                        If a value of 'required-egress-only' is supplied, this will print only the required egress.
                        
                        If a value of 'optional-egress-only' is supplied, this will print only the optional egress."
                    )
                    .action(ArgAction::Append)
                    .required(false)
            )
        ).get_matches();

    configure_telemetry(&matches);

    // parse the JSON for the egress rules and filter down to enabled groups
    let mut egress_data = load_egress_data().await?;

    println!("{:#?}", matches);

    match matches.subcommand() {
        Some(("list-groups", sub_matches)) => {
            let out = matches.get_one::<String>("format").unwrap();

            match out.as_str() {
                "table" => print_table_output(&egress_data),
                "json" => {
                    if let Some(groups) = parse_group_args(sub_matches, &egress_data) {
                        egress_data.filter_groups(&groups);

                        println!("{:#?}", serde_json::to_string(&egress_data))
                    } else {
                        println!("{:#?}", serde_json::to_string(&egress_data)?)
                    }
                }
                &_ => println!("{:#?}", egress_data),
            }
        }
        Some(("audit", sub_matches)) => {
            if let Some(groups) = parse_group_args(sub_matches, &egress_data) {
                egress_data.filter_groups(&groups);

                let conn_results = conncheck::check_connectivity(
                    &egress_data.groups,
                    sub_matches.get_one::<String>("ccp-fqdn").unwrap(),
                )
                .await?;
            } else {
                let conn_result = conncheck::check_connectivity(
                    &egress_data.groups,
                    sub_matches.get_one::<String>("ccp-fqdn").unwrap(),
                )
                .await?;
            }

            // TODO: Print out the results to the terminal
        }
        Some((&_, _)) => {
            todo!()
        }
        None => {
            todo!()
        }
    }

    Ok(())
}

fn parse_group_args(sm: ArgMatches, egress_data: &EgressData) -> Option<Vec<&str>> {
    let mut group_names: Vec<&str> = Vec::new();

    match sub_matches.get_many::<String>("egress-groups") {
        Some(groups) => {
            groups.for_each(|g| group_names.push(g.as_str()));
            Some(group_names)
        },
        None => None
    }
}

fn print_table_output(egress_data: &aks_egress_checker::egress::EgressData) {
    todo!()
}
