use anyhow::{Context, Result};
use rtk_db::{config, dlp, tracking};

use crate::{distiller, plugins};

pub enum FilterMode {
    Stdout(fn(&str) -> String),
    Stderr(fn(&str) -> String),
    Combined(fn(&str) -> String),
    Distilled,
    PluginFilter(plugins::Plugin),
}

fn post_process_filter_output(text: &str, cmd: &str) -> String {
    let text = config::apply_regex_filters(text);
    let profile = config::get_config().get_profile_for_cmd(cmd);
    config::apply_profile_settings(&text, &profile)
}

pub fn execute_with_filter(bin: &str, args: &[String], mode: FilterMode) -> Result<()> {
    let start = std::time::Instant::now();
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;
    let duration_ms = start.elapsed().as_millis() as i64;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let cmd_label = if args.is_empty() {
        bin.to_string()
    } else {
        format!("{bin} {}", args.join(" "))
    };

    let (mut out_print, mut err_print, raw_db, filtered_db) = match mode {
        FilterMode::Stdout(filter) => {
            let filtered = post_process_filter_output(&filter(&stdout), &cmd_label);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_stdout = dlp::redact_with_source(&stdout, &cmd_label);
            (
                r_filtered.clone(),
                dlp::redact_with_source(&stderr, &cmd_label),
                r_stdout.clone(),
                r_filtered,
            )
        }
        FilterMode::Stderr(filter) => {
            let filtered = post_process_filter_output(&filter(&stderr), &cmd_label);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_stderr = dlp::redact_with_source(&stderr, &cmd_label);
            (
                dlp::redact_with_source(&stdout, &cmd_label),
                r_filtered.clone(),
                r_stderr.clone(),
                r_filtered,
            )
        }
        FilterMode::Combined(filter) => {
            let combined = format!("{stderr}\n{stdout}");
            let filtered = post_process_filter_output(&filter(&combined), &cmd_label);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_combined = dlp::redact_with_source(&combined, &cmd_label);
            (
                r_filtered.clone(),
                String::new(),
                r_combined.clone(),
                r_filtered,
            )
        }
        FilterMode::Distilled => {
            let d_stdout = distiller::distill(&stdout, None);
            let d_stderr = distiller::distill(&stderr, None);
            let r_d_out = dlp::redact_with_source(&d_stdout, &cmd_label);
            let r_d_err = dlp::redact_with_source(&d_stderr, &cmd_label);
            let r_comb = dlp::redact_with_source(
                &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                &cmd_label,
            );
            (
                r_d_out.clone(),
                r_d_err.clone(),
                r_comb,
                format!("{r_d_out}\n{r_d_err}"),
            )
        }
        FilterMode::PluginFilter(ref plugin) => {
            let capture_mode = plugin.filter_mode.as_deref().unwrap_or("stdout");
            match capture_mode {
                "stderr" => {
                    let filtered = post_process_filter_output(
                        &plugins::filter_plugin(&stderr, plugin),
                        &cmd_label,
                    );
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_stderr = dlp::redact_with_source(&stderr, &cmd_label);
                    (
                        dlp::redact_with_source(&stdout, &cmd_label),
                        r_filtered.clone(),
                        r_stderr.clone(),
                        r_filtered,
                    )
                }
                "combined" => {
                    let combined = format!("{stderr}\n{stdout}");
                    let filtered = post_process_filter_output(
                        &plugins::filter_plugin(&combined, plugin),
                        &cmd_label,
                    );
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_combined = dlp::redact_with_source(&combined, &cmd_label);
                    (
                        r_filtered.clone(),
                        String::new(),
                        r_combined.clone(),
                        r_filtered,
                    )
                }
                "distill" => {
                    let d_stdout = distiller::distill(&stdout, None);
                    let d_stderr = distiller::distill(&stderr, None);
                    let r_d_out = dlp::redact_with_source(&d_stdout, &cmd_label);
                    let r_d_err = dlp::redact_with_source(&d_stderr, &cmd_label);
                    let r_comb = dlp::redact_with_source(
                        &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                        &cmd_label,
                    );
                    (
                        r_d_out.clone(),
                        r_d_err.clone(),
                        r_comb,
                        format!("{r_d_out}\n{r_d_err}"),
                    )
                }
                _ => {
                    let filtered = post_process_filter_output(
                        &plugins::filter_plugin(&stdout, plugin),
                        &cmd_label,
                    );
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_stdout = dlp::redact_with_source(&stdout, &cmd_label);
                    (
                        r_filtered.clone(),
                        dlp::redact_with_source(&stderr, &cmd_label),
                        r_stdout.clone(),
                        r_filtered,
                    )
                }
            }
        }
    };

    match tracking::record(
        cmd_label.trim(),
        &raw_db,
        &filtered_db,
        &raw_db,
        Some(duration_ms),
    ) {
        Ok(log_id) => {
            if filtered_db.len() < raw_db.len() && !filtered_db.trim().is_empty() {
                let msg = format!("\n[Full output cached. Access with: rtk show-log {log_id}]\n");
                if !out_print.trim().is_empty() {
                    out_print.push_str(&msg);
                } else if !err_print.trim().is_empty() {
                    err_print.push_str(&msg);
                }
            }
        }
        Err(e) => eprintln!("rtk: tracking warning: {e}"),
    }

    if let Some(warning) = tracking::check_autonomy(&filtered_db) {
        if !out_print.trim().is_empty() {
            out_print.push_str(warning);
            out_print.push('\n');
        } else if !err_print.trim().is_empty() {
            err_print.push_str(warning);
            err_print.push('\n');
        }
    }

    if !out_print.is_empty() {
        print!("{out_print}");
    }
    if !err_print.is_empty() {
        eprint!("{err_print}");
    }

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

pub fn run_filtered(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stdout(filter))
}

pub fn run_filtered_stderr(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stderr(filter))
}

pub fn run_filtered_combined(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Combined(filter))
}

pub fn run_distilled(bin: &str, args: &[String]) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Distilled)
}
