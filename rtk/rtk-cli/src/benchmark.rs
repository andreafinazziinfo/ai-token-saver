use anyhow::{Context, Result};
use rtk_db::{pricing, tracking};
use serde_json::json;
use std::fs::File;
use std::io::Write;

fn get_git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn export_json(output_path: &str) -> Result<()> {
    let logs = tracking::get_recent_logs(50000).context("fetch logs for json export")?;
    let commit_sha = get_git_commit();
    let registry = pricing::get_registry();

    let now = std::time::SystemTime::now();
    let timestamp = match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs().to_string(),
        Err(_) => "0".to_string(),
    };

    let results: Vec<serde_json::Value> = logs
        .into_iter()
        .map(|log| {
            let saved = log.original_tokens - log.filtered_tokens;
            let pct = if log.original_tokens > 0 {
                (saved as f64 / log.original_tokens as f64) * 100.0
            } else {
                0.0
            };
            let model = log.model.unwrap_or_else(|| "unknown".to_string());
            let cost_saved = pricing::calculate_savings(saved, &model);

            json!({
                "id": log.id,
                "cmd": log.cmd,
                "original_tokens": log.original_tokens,
                "filtered_tokens": log.filtered_tokens,
                "saved_tokens": saved,
                "savings_pct": pct,
                "cost_saved_usd": cost_saved,
                "duration_ms": log.duration_ms.unwrap_or(0),
                "model": model,
                "timestamp": log.timestamp
            })
        })
        .collect();

    let export_data = json!({
        "metadata": {
            "timestamp": timestamp,
            "commit_sha": commit_sha,
            "rtk_version": env!("CARGO_PKG_VERSION"),
            "pricing_revision": registry.pricing_revision
        },
        "results": results
    });

    let content = serde_json::to_string_pretty(&export_data).context("serialize json export")?;
    std::fs::write(output_path, content)
        .with_context(|| format!("write json to {}", output_path))?;
    Ok(())
}

pub fn export_csv(output_path: &str) -> Result<()> {
    let logs = tracking::get_recent_logs(50000).context("fetch logs for csv export")?;
    let mut file =
        File::create(output_path).with_context(|| format!("create csv file {}", output_path))?;

    // Header
    writeln!(
        file,
        "id,timestamp,command,model,original_tokens,filtered_tokens,saved_tokens,savings_pct,cost_saved_usd,duration_ms"
    )?;

    for log in logs {
        let saved = log.original_tokens - log.filtered_tokens;
        let pct = if log.original_tokens > 0 {
            (saved as f64 / log.original_tokens as f64) * 100.0
        } else {
            0.0
        };
        let model = log.model.unwrap_or_else(|| "unknown".to_string());
        let cost_saved = pricing::calculate_savings(saved, &model);

        let escaped_cmd = log.cmd.replace('"', "\"\"");

        writeln!(
            file,
            "{},{},\"{}\",{},{},{},{},{:.2},{:.6},{}",
            log.id,
            log.timestamp,
            escaped_cmd,
            model,
            log.original_tokens,
            log.filtered_tokens,
            saved,
            pct,
            cost_saved,
            log.duration_ms.unwrap_or(0)
        )?;
    }

    Ok(())
}
