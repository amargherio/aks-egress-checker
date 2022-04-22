use clap::ArgMatches;
use std::env;

fn build_subscribers() {

}

pub fn configure_telemetry(matches: &ArgMatches) {
    // get the desired log level - check the flags, then envvar, and default
    // to info if no level provided.
    if matches.is_present("loglevel") {
        env::set_var("RUST_LOG", matches.value_of("loglevel").unwrap());
    } else if env::var("RUST_LOG").is_ok() {
        // at this point we do nothing because the env var is already configured.
    } else {
        // we've missed the flag and the env var, so we create the env var with our default value.
        env::set_var("RUST_LOG", "info")
    }
}