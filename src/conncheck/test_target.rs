use anyhow::Result;

use crate::egress::EgressRule;

/// Builds a connection string for a specific egress rule, CCP, and VM region.
///
/// This function takes references to an `EgressRule`, CCP (Control Plane) as a string, and VM region as a string, and returns a `Result<String>`
/// containing the generated connection string. The function replaces templates within the rule's destination (e.g. "{region}") with actual values.
///
/// The egress rule contains a few considerations:
/// 1. If the destination is a wildcard "*" and the rule name contains "api-server", it treats it as
///    "kubernetes.default.svc.cluster.local". The CCP FQDN value is required, but the function acknowledges the need for a smarter way to determine
///    the CCP for a given cluster in the future.
/// 2. If not as described above, it replaces templates with actual values, such as the VM's region.
///    The replacements for "{endpoint}" and "{id}" are still pending.
///
/// This function is `async` and should be awaited to obtain the `Result<String>` containing the connection string.
pub(crate) async fn build_conn_string(rule: &EgressRule, ccp: &str, vm_region: &str) -> Result<String> {
    tracing::debug!("Building connection string for attempted FQDN and port");
    let mut conn_string: String = String::new();

    if rule.dst == "*" && rule.name.contains("api-server") {
        // this wildcard is a bit...weird. this should be treated as `kubernetes.default.svc.cluster.local`
        // if the CCP FDQN isn't specified, although this could result in inaccurate connectivity tests.
        //
        // for now the ccp-fqdn value is required, but eventually we need a smarter way to determine
        // the CCP for a given cluster.
        conn_string = format!("{}:{}", ccp, rule.port);
    } else {
        // replacing templates with actual values.
        conn_string = rule.dst.replace("{region}", vm_region);
        // TODO: finish replacements for {endpoint} and {id}
    }

    Ok(conn_string)
}

async fn retrieve_cluster_fqdn() -> Result<String> {
    // using the azure SDK for rust, pull the cluster information (resource ID) and query Azure
    // to get the API server FQDN for the cluster.
    todo!()
}