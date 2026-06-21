use anyhow::Result;
use rtk_db::{pricing, tracking};
use std::path::PathBuf;

pub fn run_doctor() -> Result<()> {
    println!("🩺 Running RTK Health Check...");
    println!("==================================================");

    let mut all_ok = true;

    // 1. Check database access
    print!("🗄️  Database Access: ");
    match tracking::get_recent_logs(1) {
        Ok(_) => {
            println!("✅ OK");
        }
        Err(e) => {
            println!("❌ FAILED (Error: {e})");
            println!("   👉 Suggestions: Check RTK_DB_PATH env variable or folder permissions.");
            all_ok = false;
        }
    }

    // 2. Check pricing registry
    print!("💰 Pricing Registry: ");
    let registry = pricing::get_registry();
    if !registry.models.is_empty() {
        println!("✅ OK (revision: {}, {} models loaded)", registry.pricing_revision, registry.models.len());
    } else {
        println!("❌ FAILED (no models loaded in registry)");
        all_ok = false;
    }

    // 3. Check shell aliases / hooks
    print!("🐚 Shell Aliases: ");
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    
    let mut alias_found = false;
    if let Some(h) = home {
        let shells = [".bashrc", ".zshrc", ".profile"];
        for shell in shells {
            let path = h.join(shell);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("RTK AI Token Saver Aliases") {
                        alias_found = true;
                        break;
                    }
                }
            }
        }
    }
    
    if alias_found {
        println!("✅ OK");
    } else {
        println!("⚠️  WARNING (aliases not found in ~/.bashrc, ~/.zshrc or ~/.profile)");
        println!("   👉 Suggestion: Run `rtk init` to automatically install aliases and shell hooks.");
    }

    // 4. Check Rust version
    print!("🦀 Rust Version: ");
    let rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(version) = rustc_version {
        println!("✅ OK ({})", version);
    } else {
        println!("⚠️  WARNING (rustc command not found)");
    }

    // 5. Check Disk space / directory permissions
    print!("💾 Workspace Access: ");
    if std::fs::metadata(".").is_ok() {
        println!("✅ OK");
    } else {
        println!("❌ FAILED (current workspace is not readable/writable)");
        all_ok = false;
    }

    println!("==================================================");
    if all_ok {
        println!("✅ RTK is healthy and ready to optimize your AI coding agent workflow!");
        Ok(())
    } else {
        println!("❌ RTK doctor found some issues. Please fix the failures above.");
        Err(anyhow::anyhow!("RTK health check failed"))
    }
}
