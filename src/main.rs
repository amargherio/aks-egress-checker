use aks_egress_checker::egress::{EgressGroup, EgressRule};
use aks_egress_checker::{
    conncheck,
    egress::{load_egress_data, print_conn_results, EgressData},
    telemetry::configure_telemetry,
};
use clap::{builder::PossibleValue, Arg, ArgAction, ArgMatches, Command};
use tabled::{builder::Builder, Style};

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
                .short('f')
                .long("output-file")
                .help("File path to save the results at. Recommended for JSON output.")
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
                .about("Prints the group and rule definition for one or more egress groups.")
            .arg(
                Arg::new("egress-groups")
                    .long("egress-group")
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
                    let filtered_groups = parse_group_args(sub_matches);
                    match filtered_groups {
                        Some(groups) => {
                            egress_data.filter_groups(&groups);

                            println!("{:#?}", serde_json::to_string(&egress_data))
                        }
                        None => println!("{:#?}", serde_json::to_string(&egress_data)?),
                    }
                }
                &_ => println!("{:#?}", egress_data),
            }
        }
        Some(("audit", sub_matches)) => {
            if let Some(groups) = parse_group_args(sub_matches) {
                egress_data.filter_groups(&groups);

                let conn_results = conncheck::check_connectivity(
                    &egress_data.groups,
                    sub_matches.get_one::<String>("ccp-fqdn").unwrap(),
                )
                .await?;

                print_conn_results(&conn_results, &sub_matches).await;
            } else {
                let conn_results = conncheck::check_connectivity(
                    &egress_data.groups,
                    sub_matches.get_one::<String>("ccp-fqdn").unwrap(),
                )
                .await?;

                print_conn_results(&conn_results, &sub_matches).await;
            }
        }
        Some((&_, _)) => {
            unimplemented!()
        }
        None => {
            unimplemented!()
        }
    }

    Ok(())
}

fn parse_group_args<'a>(sm: &'a ArgMatches) -> Option<Vec<&String>> {
    let mut group_names: Vec<&String> = Vec::new();

    match sm.get_many::<String>("egress-groups") {
        Some(groups) => {
            groups.for_each(|g| group_names.push(g));
            Some(group_names.clone())
        }
        None => None,
    }
}

fn print_table_output(egress_data: &EgressData) {
    let mut builder = Builder::default();
    let columns = vec![
        "Egress Group",
        "Name",
        "Destination",
        "Port",
        "Protocol",
        "Required for private clusters?",
        "Enabled for checking?",
    ];
    builder.set_columns(columns);

    // for each set of egress groups, grab some basics and then iterate the rules to print each one
    egress_data.groups.iter().for_each(|g: &EgressGroup| {
        g.rules
            .iter()
            .filter(|r| r.rule_enabled)
            .for_each(|r: &EgressRule| {
                let mut enabled = String::new();
                let mut required_private = String::new();

                if r.rule_enabled {
                    enabled = String::from("Yes");
                } else {
                    enabled = String::from("No");
                }

                if r.required_private {
                    required_private = String::from("Yes");
                } else {
                    required_private = String::from("No");
                }
                //builder.add_record(vec![g.name.clone(), r.name.clone(), r.dst.clone(), r.port.clone(), r.protocol.clone(), r.description.clone(), required_private.clone(), enabled.clone()]);
                builder.add_record(vec![
                    g.name.clone(),
                    r.name.clone(),
                    r.dst.clone(),
                    r.port.clone(),
                    r.protocol.clone(),
                    required_private.clone(),
                    enabled.clone(),
                ]);
            });
    });

    let mut table = builder.build();
    table.with(Style::modern());

    println!("{}", table);
}
