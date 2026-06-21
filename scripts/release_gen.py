#!/usr/bin/env python3
import os
import sys
import sqlite3
import re
from pathlib import Path

def get_db_path():
    if "RTK_DB_PATH" in os.environ:
        return Path(os.environ["RTK_DB_PATH"])
    
    home = Path.home()
    if os.name == "nt": # Windows
        local_appdata = os.environ.get("LOCALAPPDATA")
        if local_appdata:
            return Path(local_appdata) / "rtk" / "rtk.db"
        return home / "AppData" / "Local" / "rtk" / "rtk.db"
    else: # Linux/macOS/WSL
        return home / ".local" / "share" / "rtk" / "rtk.db"

def get_version():
    try:
        cargo_path = Path(__file__).parent.parent / "rtk" / "rtk-cli" / "Cargo.toml"
        if cargo_path.exists():
            content = cargo_path.read_text()
            match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
            if match:
                return match.group(1)
    except Exception:
        pass
    return "0.1.0"

def main():
    db_path = get_db_path()
    version = get_version()
    
    total_commands = 0
    total_orig = 0
    total_filt = 0
    total_saved = 0
    pct = 0.0
    cost_saved_usd = 0.0
    
    if db_path.exists():
        try:
            conn = sqlite3.connect(str(db_path))
            cursor = conn.cursor()
            
            # Check if tracking table exists
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='tracking'")
            if cursor.fetchone():
                cursor.execute("SELECT COUNT(*), SUM(original_tokens), SUM(filtered_tokens) FROM tracking")
                row = cursor.fetchone()
                if row and row[0] > 0:
                    total_commands = row[0]
                    total_orig = row[1] or 0
                    total_filt = row[2] or 0
                    total_saved = max(0, total_orig - total_filt)
                    if total_orig > 0:
                        pct = (total_saved / total_orig) * 100.0
                    
                    # Estimate cost: assume $3.00 per million tokens input (Claude 3.5 Sonnet average)
                    cost_saved_usd = (total_saved / 1_000_000.0) * 3.00
            conn.close()
        except Exception as e:
            print(f"Warning: Failed to read database: {e}", file=sys.stderr)
            
    print(f"# Release Notes — RTK v{version}")
    print("\nWe are thrilled to announce the latest release of **RTK (Rust Context Engine)**! This release introduces advanced context optimization, hybrid retrieval structures, and key performance improvements.")
    print("\n## 📊 Real-world Efficiency Statistics")
    print(f"Based on your local telemetry database (`{db_path.name}`):")
    print(f"- **Total CLI Command Wrappers run**: `{total_commands}`")
    print(f"- **Original prompt tokens processed**: `{total_orig:,}` tokens")
    print(f"- **Optimized prompt tokens sent**: `{total_filt:,}` tokens")
    print(f"- **Net tokens saved by filtering**: **`{total_saved:,}` tokens**")
    print(f"- **Average context reduction rate**: **`{pct:.2f}%`**")
    print(f"- **Estimated developer spending saved**: **`${cost_saved_usd:.4f} USD**")
    print("\n## 🚀 What's New in this Version")
    print("- **Hybrid Retrieval (embeddings feature)**: Combined lexical (BM25) and semantic (ONNX) search for symbol definitions.")
    print("- **Compression Policy Engine (`rtk context compact`)**: Dynamically compact standard input context to fit within strict token budgets.")
    print("- **Observability Telemetry Exporter (`rtk telemetry export`)**: Export cost metrics directly in Prometheus/JSON format for Grafana dashboards.")
    print("- **Obsidian Backlinks Export (`rtk graph export`)**: Visualize your code graph in Obsidian with fully cross-referenced wiki notes.")
    print("\n## ⚙️ How to Upgrade")
    print("Download the latest binary release and reinstall the MCP server configuration:")
    print("```bash")
    print("rtk doctor")
    print("rtk mcp install --client claude")
    print("```")

if __name__ == "__main__":
    main()
