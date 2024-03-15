use clap::ArgMatches;
use tracing_subscriber::{filter::EnvFilter, fmt, Registry, prelude::*};

use std::env;
use tracing_subscriber::layer::SubscriberExt;

pub fn configure_telemetry(matches: &ArgMatches) {
    // get the desired log level - check the flags, then envvar, and default
    // to info if no level provided.
    if let Some(level) = matches.get_one::<String>("loglevel") {
        env::set_var("RUST_LOG", level);
    } else if env::var("RUST_LOG").is_ok() {
        // at this point we do nothing because the env var is already configured.
    } else {
        // we've missed the flag and the env var, so we create the env var with our default value.
        env::set_var("RUST_LOG", "info")
    }

    // Initialize the log tracer and the global subscriber.
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::registry().with(fmt::layer()).with(env_filter).init();
}
