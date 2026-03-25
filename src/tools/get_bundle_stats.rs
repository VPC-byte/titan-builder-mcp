use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBundleStatsParams {
    /// The hash of the bundle to query. Required.
    pub bundle_hash: String,
}

pub fn analyze_bundle_status(value: &serde_json::Value) -> String {
    let status = value
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("Unknown");

    let builder_payment_zero = value
        .get("builderPayment")
        .and_then(|p| p.as_str())
        .map(|p| p == "0")
        .unwrap_or(false);

    let error_field = value
        .get("error")
        .and_then(|e| e.as_str())
        .unwrap_or("");

    let revert_hash = if error_field.contains("BundleRevert") {
        // Try to extract the hash (0x followed by hex chars) from the error string
        error_field
            .split_whitespace()
            .find(|token| token.starts_with("0x") && token.len() >= 10)
            .map(|s| s.trim_end_matches([',', '.', ';', ')'].as_ref()))
            .unwrap_or("")
    } else {
        ""
    };

    let (root_cause, suggestions) = match status {
        "Received" => (
            "Bundle arrived too late for the mempool.".to_string(),
            vec![
                "Submit earlier in the slot".to_string(),
                "Use a regional endpoint to reduce latency".to_string(),
            ],
        ),
        "Invalid" => (
            "Malformed bundle (bad RLP, wrong chain ID, mined nonces).".to_string(),
            vec![
                "Verify transactions with estimateGas".to_string(),
                "Check block number > current or set to 0".to_string(),
                "Verify chain ID".to_string(),
            ],
        ),
        "SimulationFail" => {
            let rc = if !revert_hash.is_empty() {
                format!(
                    "Transaction {} reverted during top-of-block simulation.",
                    revert_hash
                )
            } else if builder_payment_zero {
                "Builder payment is 0 — bundle does not pay the builder.".to_string()
            } else {
                "Transaction reverted or builder payment <= 0.".to_string()
            };
            let mut sugg = Vec::new();
            if !revert_hash.is_empty() {
                sugg.push(format!(
                    "Add {} to revertingTxHashes if this revert is acceptable",
                    revert_hash
                ));
                sugg.push(
                    "Verify the transaction succeeds via estimateGas against current state"
                        .to_string(),
                );
            } else {
                sugg.push("Check error field for reverting hash".to_string());
                sugg.push(
                    "Add reverting tx to revertingTxHashes if the revert is acceptable"
                        .to_string(),
                );
            }
            sugg.push(
                "Ensure builder payment (post-bundle balance - pre-bundle balance) is > 0"
                    .to_string(),
            );
            (rc, sugg)
        }
        "SimulationPass" => (
            "Passed simulation but arrived too late.".to_string(),
            vec!["Submit earlier in the slot window".to_string()],
        ),
        "ExcludedFromBlock" => (
            "Valid but not selected (usually insufficient bribe).".to_string(),
            vec![
                "Increase priority fee — solves 99% of cases".to_string(),
            ],
        ),
        "IncludedInBlock" => (
            "In a candidate block but a more valuable block won.".to_string(),
            vec![
                "Increase bribe".to_string(),
                "Check builderPaymentWhenIncluded for context".to_string(),
            ],
        ),
        "Submitted" => (
            "Success — submitted to relay.".to_string(),
            vec!["No action needed — monitor relay for on-chain inclusion".to_string()],
        ),
        _ => (
            format!("Unrecognised status '{}' — no analysis available.", status),
            vec!["Check Titan Builder documentation for up-to-date status codes".to_string()],
        ),
    };

    let mut extra_flags: Vec<String> = Vec::new();
    if builder_payment_zero && status != "SimulationFail" {
        extra_flags.push(
            "Warning: builderPayment is \"0\" — the bundle does not pay the builder.".to_string(),
        );
    }

    let suggestions_text = suggestions
        .iter()
        .map(|s| format!("- {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    let flags_text = if extra_flags.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", extra_flags.join("\n"))
    };

    format!(
        "## Analysis\nStatus: {}\nRoot cause: {}\n\nSuggestions:\n{}{}",
        status, root_cause, suggestions_text, flags_text
    )
}
