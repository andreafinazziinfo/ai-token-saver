use rtk_memory::tracking::record;
use std::process::Command;

/// Run a dotnet command, capture its output, apply distillation and DLP redaction, and record savings.
pub fn execute_dotnet(args: &[String]) {
    let output = Command::new("dotnet")
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute dotnet command"));

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let full_output = format!("{}{}", stdout_str, stderr_str);

    if full_output.trim().is_empty() {
        return;
    }

    let mut filtered_lines = Vec::new();
    let lines: Vec<&str> = full_output.lines().collect();

    for line in &lines {
        // Drop standard verbose build logs unless they are errors or warnings
        if line.contains("error CS")
            || line.contains("warning CS")
            || line.contains("Build FAILED")
            || line.starts_with("Failed!")
            || line.starts_with("Passed!")
            || line.starts_with("Total tests:")
        {
            filtered_lines.push(line.to_string());
        }
    }

    // Keep the last few lines for summary if not already included
    let len = lines.len();
    if len > 5 {
        for line in &lines[len.saturating_sub(5)..] {
            if !filtered_lines.contains(&line.to_string()) && !line.trim().is_empty() {
                filtered_lines.push(line.to_string());
            }
        }
    } else if filtered_lines.is_empty() {
        // If output was tiny, just return it
        filtered_lines = lines.iter().map(|s| s.to_string()).collect();
    }

    let filtered_output = filtered_lines.join("\n");
    let cmd_str = format!("dotnet {}", args.join(" "));
    let log_id = record(&cmd_str, &full_output, &filtered_output, &full_output).unwrap_or(0);

    let mut final_out = filtered_output.clone();
    if !filtered_output.trim().is_empty() && full_output.len() > filtered_output.len() {
        final_out.push_str(&format!(
            "\n[Full output cached. Access with: rtk show-log {}]",
            log_id
        ));
    }

    if let Some(warning) = rtk_memory::tracking::check_autonomy(&filtered_output) {
        final_out.push_str(warning);
        final_out.push('\n');
    }

    println!("{}", final_out);
}
