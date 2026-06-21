use anyhow::{Context, Result};
use rtk_db::tracking;
use std::fs;
use std::io::{BufRead, Write};
use std::net::TcpListener;
use std::path::Path;

/// Launch the savings dashboard.
/// If `live` is true, starts a local HTTP telemetry server.
/// If `live` is false, generates a static HTML file and opens it in the browser.
pub fn run_dashboard(live: bool, port_opt: Option<u16>) -> Result<()> {
    if !live {
        return run_static_dashboard();
    }

    let mut port = port_opt.unwrap_or(3000);
    let listener = loop {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(l) => break l,
            Err(_) => {
                if port_opt.is_some() {
                    return Err(anyhow::anyhow!("Requested port {} is already in use", port));
                }
                port += 1;
                if port > 3100 {
                    return Err(anyhow::anyhow!(
                        "Could not find any available ports between 3000 and 3100"
                    ));
                }
            }
        }
    };

    let url = format!("http://127.0.0.1:{}", port);
    println!("==========================================================");
    println!("📊  RTK Live Savings Dashboard Server");
    println!("==========================================================");
    println!("🌐  Local server running at: {}", url);
    println!("🔒  Bound to localhost for local security");
    println!("⌨️   Press Ctrl+C to terminate the server");
    println!("==========================================================");

    // Auto-launch the web browser to the dashboard URL
    open_browser_url(&url);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                if let Err(e) = handle_connection(s) {
                    eprintln!("rtk: dashboard server error: {e}");
                }
            }
            Err(e) => {
                eprintln!("rtk: incoming connection failed: {e}");
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: std::net::TcpStream) -> Result<()> {
    let mut reader = std::io::BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let (content, content_type) = match path {
        "/" => (get_live_dashboard_html()?, "text/html; charset=utf-8"),
        "/api/stats" => {
            let (count, original, filtered, saved, usd_saved) = tracking::get_savings_data()?;
            let hours_saved = (count as f64 * 22.8) / 3600.0;
            
            // Query for most saved and most frequent commands
            let breakdown = tracking::get_command_breakdown()?;
            let top_saver = breakdown.first().map(|(cmd, _, _)| cmd.clone()).unwrap_or_else(|| "N/A".to_string());
            let most_frequent = breakdown.first().map(|(cmd, _, _)| cmd.clone()).unwrap_or_else(|| "N/A".to_string());

            let json = serde_json::json!({
                "count": count,
                "original": original,
                "filtered": filtered,
                "saved": saved,
                "usd_saved": usd_saved,
                "hours_saved": hours_saved,
                "top_saver": top_saver,
                "most_frequent": most_frequent
            });
            (json.to_string(), "application/json")
        }
        "/api/breakdown" => {
            let breakdown = tracking::get_command_breakdown()?;
            let list: Vec<serde_json::Value> = breakdown
                .into_iter()
                .map(|(cmd, count, saved)| {
                    serde_json::json!({ "cmd": cmd, "count": count, "saved": saved })
                })
                .collect();
            (serde_json::to_string(&list)?, "application/json")
        }
        "/api/logs" => {
            let logs = tracking::get_recent_logs(50)?;
            (serde_json::to_string(&logs)?, "application/json")
        }
        "/api/daily" => {
            let daily = tracking::get_daily_savings()?;
            (serde_json::to_string(&daily)?, "application/json")
        }
        "/api/models" => {
            let models = tracking::get_model_savings()?;
            (serde_json::to_string(&models)?, "application/json")
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Connection: close\r\n\r\n{}",
        content_type,
        content.len(),
        content
    );

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn run_static_dashboard() -> Result<()> {
    let report_path = Path::new("dashboard.html");
    println!(
        "📊 Generating static savings dashboard at {}...",
        report_path.display()
    );

    let html_content = get_live_dashboard_html()?;
    fs::write(report_path, html_content).with_context(|| {
        format!(
            "failed to write dashboard report file: {}",
            report_path.display()
        )
    })?;

    println!("✅ Static dashboard report created successfully.");
    open_browser(report_path);
    Ok(())
}

fn open_browser(path: &Path) {
    let path_str = path.to_string_lossy().to_string();
    println!("🌐 Opening dashboard in browser...");
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &path_str])
            .status();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path_str).status();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .status();
    }
}

fn open_browser_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .status();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).status();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).status();
    }
}

fn get_live_dashboard_html() -> Result<String> {
    Ok(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RTK — Live Savings Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        body {
            font-family: 'Outfit', sans-serif;
            background-color: #090d16;
            color: #f1f5f9;
        }
        .glass {
            background: rgba(17, 24, 39, 0.7);
            backdrop-filter: blur(16px);
            border: 1px solid rgba(255, 255, 255, 0.04);
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
        }
        .glow-emerald {
            box-shadow: 0 0 20px 0 rgba(16, 185, 129, 0.1);
        }
        .glow-cyan {
            box-shadow: 0 0 20px 0 rgba(6, 182, 212, 0.1);
        }
        /* Custom scrollbar */
        ::-webkit-scrollbar {
            width: 8px;
            height: 8px;
        }
        ::-webkit-scrollbar-track {
            background: #0d1222;
        }
        ::-webkit-scrollbar-thumb {
            background: #1e293b;
            border-radius: 4px;
        }
        ::-webkit-scrollbar-thumb:hover {
            background: #334155;
        }
    </style>
</head>
<body class="min-h-screen flex flex-col justify-between py-6 px-4 sm:px-8">
    <div class="max-w-7xl mx-auto w-full flex-grow">
        <!-- Header -->
        <header class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-8 gap-4">
            <div>
                <h1 class="text-4xl font-extrabold tracking-tight bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400 bg-clip-text text-transparent">
                    RTK Live Savings
                </h1>
                <p class="text-slate-400 text-sm mt-1">Real-time developer savings and token virtualization metrics</p>
            </div>
            
            <div class="flex items-center gap-3">
                <div class="px-4 py-1.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5 glow-emerald">
                    <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
                    Live Connection Active
                </div>
                <button onclick="downloadJsonBackup()" class="px-4 py-1.5 rounded-full text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700/50 transition-all flex items-center gap-1.5">
                    📥 Export JSON
                </button>
            </div>
        </header>

        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div class="glass p-6 rounded-2xl flex flex-col justify-between hover:border-slate-800 transition-all">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Commands Intercepted</span>
                <span id="stat-count" class="text-4xl font-extrabold text-white mt-2">0</span>
            </div>
            <div class="glass p-6 rounded-2xl flex flex-col justify-between hover:border-slate-800 transition-all">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Tokens Saved (Ratio)</span>
                <div class="flex items-baseline gap-2 mt-2">
                    <span id="stat-saved" class="text-4xl font-extrabold text-emerald-400">0</span>
                    <span id="stat-ratio" class="text-sm font-semibold text-emerald-500">(0%)</span>
                </div>
            </div>
            <div class="glass p-6 rounded-2xl flex flex-col justify-between border-t border-emerald-500/20 hover:border-slate-800 transition-all glow-emerald">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Estimated Cost Saved</span>
                <span id="stat-usd" class="text-4xl font-extrabold text-emerald-300 mt-2">$0.00</span>
            </div>
            <div class="glass p-6 rounded-2xl flex flex-col justify-between border-t border-cyan-500/20 hover:border-slate-800 transition-all glow-cyan">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Developer Wait Saved</span>
                <span id="stat-hours" class="text-4xl font-extrabold text-cyan-300 mt-2">0.0 hrs</span>
            </div>
        </div>

        <!-- Metric Details Cards (Top Saver, Most Frequent, Active Task) -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <div class="glass p-5 rounded-2xl flex items-center gap-4">
                <div class="p-3 bg-emerald-500/10 text-emerald-400 rounded-xl">
                    🔥
                </div>
                <div>
                    <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Top Saver Command</span>
                    <span id="stat-top-saver" class="text-lg font-bold text-white mt-0.5">N/A</span>
                </div>
            </div>
            <div class="glass p-5 rounded-2xl flex items-center gap-4">
                <div class="p-3 bg-cyan-500/10 text-cyan-400 rounded-xl">
                    ⚡
                </div>
                <div>
                    <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Most Frequent Command</span>
                    <span id="stat-most-frequent" class="text-lg font-bold text-white mt-0.5">N/A</span>
                </div>
            </div>
            <div class="glass p-5 rounded-2xl flex items-center gap-4">
                <div class="p-3 bg-indigo-500/10 text-indigo-400 rounded-xl">
                    🛠️
                </div>
                <div>
                    <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider block">Active Task / Branch</span>
                    <span id="stat-active-task" class="text-lg font-bold text-white mt-0.5">N/A</span>
                </div>
            </div>
        </div>

        <!-- Charts Section -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 mb-8">
            <div class="glass p-6 rounded-2xl lg:col-span-2">
                <h3 class="text-lg font-semibold text-white mb-4">Daily Token Savings (Original vs Filtered)</h3>
                <div class="h-72 relative">
                    <canvas id="dailyChart"></canvas>
                </div>
            </div>
            
            <div class="glass p-6 rounded-2xl">
                <h3 class="text-lg font-semibold text-white mb-4">Model Distribution</h3>
                <div class="h-72 relative">
                    <canvas id="modelsChart"></canvas>
                </div>
            </div>
        </div>

        <!-- Log Records & Split Screen Section -->
        <div class="grid grid-cols-1 xl:grid-cols-3 gap-8 mb-8">
            <!-- Telemetry Records Table -->
            <div class="glass rounded-2xl p-6 xl:col-span-2 flex flex-col h-[500px]">
                <div class="flex justify-between items-center mb-4 gap-4">
                    <h3 class="text-lg font-semibold text-white">Command Telemetry Logs</h3>
                    <input type="text" id="logSearch" placeholder="Filter commands/branches..." class="px-4 py-1.5 bg-slate-800/50 border border-slate-700/50 rounded-lg text-sm text-slate-300 placeholder-slate-500 focus:outline-none focus:border-emerald-500/40 w-64">
                </div>
                
                <div class="overflow-y-auto flex-grow pr-1">
                    <table class="w-full text-left text-sm text-slate-300">
                        <thead class="text-xs uppercase bg-slate-900/40 text-slate-400 sticky top-0 backdrop-blur border-b border-slate-800">
                            <tr>
                                <th class="px-4 py-2.5">Command</th>
                                <th class="px-4 py-2.5">Model</th>
                                <th class="px-4 py-2.5">Savings</th>
                                <th class="px-4 py-2.5">Time</th>
                                <th class="px-4 py-2.5">Branch</th>
                            </tr>
                        </thead>
                        <tbody id="logTableBody" class="divide-y divide-slate-800/30">
                            <!-- Populated dynamically -->
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Savings Breakdown Categories -->
            <div class="glass rounded-2xl p-6 flex flex-col h-[500px]">
                <h3 class="text-lg font-semibold text-white mb-4">Command Category Breakdown</h3>
                <div class="overflow-y-auto flex-grow pr-1">
                    <div id="breakdownContainer" class="space-y-4">
                        <!-- Populated dynamically -->
                    </div>
                </div>
            </div>
        </div>

        <!-- Split Screen Raw/Filtered Inspector -->
        <div id="inspectorContainer" class="hidden glass rounded-2xl p-6 mb-8 border border-slate-800 transition-all">
            <div class="flex justify-between items-center mb-4 pb-3 border-b border-slate-800">
                <div>
                    <h3 class="text-lg font-semibold text-white flex items-center gap-2">
                        🔎 Output Inspector: <span id="inspect-cmd" class="text-emerald-400 font-mono text-base"></span>
                    </h3>
                    <p class="text-slate-400 text-xs mt-1">
                        Branch: <span id="inspect-branch" class="text-slate-300 font-semibold mr-4"></span>
                        Project: <span id="inspect-project" class="text-slate-300 font-semibold mr-4"></span>
                        Model: <span id="inspect-model" class="text-slate-300 font-semibold"></span>
                    </p>
                </div>
                <button onclick="closeInspector()" class="p-1 px-3 bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white rounded-lg transition-all text-sm font-semibold">
                    ✕ Close
                </button>
            </div>
            
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div>
                    <span class="text-slate-400 text-xs font-bold uppercase tracking-wider block mb-2">Original Output</span>
                    <pre class="bg-slate-950 p-4 rounded-xl text-slate-300 text-xs font-mono overflow-auto h-96 max-h-96 border border-slate-900/50" id="inspect-raw"></pre>
                </div>
                <div>
                    <div class="flex justify-between items-center mb-2">
                        <span class="text-emerald-400 text-xs font-bold uppercase tracking-wider block">RTK Filtered Signal</span>
                        <span id="inspect-savings-badge" class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/10 text-emerald-400"></span>
                    </div>
                    <pre class="bg-slate-950 p-4 rounded-xl text-slate-300 text-xs font-mono overflow-auto h-96 max-h-96 border border-slate-900/50" id="inspect-filtered"></pre>
                </div>
            </div>
        </div>

    </div>

    <!-- Footer -->
    <footer class="max-w-7xl mx-auto w-full text-center text-xs text-slate-700 mt-6 pt-4 border-t border-slate-900">
        RTK Token Saver is licensed under the Apache License 2.0. Real-time telemetry connection.
    </footer>

    <!-- Scripts and Chart initialization -->
    <script>
        let dailyChart = null;
        let modelsChart = null;
        let cachedLogs = [];
        let activeInspectorLogId = null;

        // Fetch metrics and update DOM
        async function fetchMetrics() {
            try {
                // Fetch stats
                const resStats = await fetch('/api/stats');
                const stats = await resStats.json();
                
                document.getElementById('stat-count').innerText = stats.count;
                document.getElementById('stat-saved').innerText = formatNumber(stats.saved);
                
                const ratio = stats.original > 0 ? ((stats.saved / stats.original) * 100).toFixed(1) : '0';
                document.getElementById('stat-ratio').innerText = `(${ratio}%)`;
                document.getElementById('stat-usd').innerText = `$${stats.usd_saved.toFixed(4)}`;
                document.getElementById('stat-hours').innerText = `${stats.hours_saved.toFixed(2)} hrs`;
                document.getElementById('stat-top-saver').innerText = stats.top_saver || 'N/A';
                document.getElementById('stat-most-frequent').innerText = stats.most_frequent || 'N/A';

                // Fetch breakdown
                const resBreakdown = await fetch('/api/breakdown');
                const breakdown = await resBreakdown.json();
                renderBreakdown(breakdown);

                // Fetch logs
                const resLogs = await fetch('/api/logs');
                const logs = await resLogs.json();
                cachedLogs = logs;
                
                // Update active branch/task from most recent log
                if (logs.length > 0) {
                    const latest = logs[0];
                    const activeText = `${latest.project || 'Unknown'} (${latest.branch || 'detached'})`;
                    document.getElementById('stat-active-task').innerText = activeText;
                }
                
                renderLogs(logs);

                // Update active inspector if details are refreshed
                if (activeInspectorLogId !== null) {
                    const currentInspect = logs.find(l => l.id === activeInspectorLogId);
                    if (currentInspect) {
                        inspectLog(currentInspect);
                    }
                }

                // Update daily trend
                const resDaily = await fetch('/api/daily');
                const dailyData = await resDaily.json();
                updateDailyChart(dailyData);

                // Update model breakdown
                const resModels = await fetch('/api/models');
                const modelsData = await resModels.json();
                updateModelsChart(modelsData);

            } catch (err) {
                console.error("Telemetry fetch failed:", err);
            }
        }

        function formatNumber(num) {
            if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
            if (num >= 1000) return (num / 1000).toFixed(1) + 'k';
            return num;
        }

        // Render recent logs in table
        function renderLogs(logs) {
            const tableBody = document.getElementById('logTableBody');
            const searchVal = document.getElementById('logSearch').value.toLowerCase();
            
            // Filter logs based on search query
            const filteredLogs = logs.filter(l => 
                l.cmd.toLowerCase().includes(searchVal) || 
                (l.branch && l.branch.toLowerCase().includes(searchVal)) || 
                (l.model && l.model.toLowerCase().includes(searchVal))
            );

            if (filteredLogs.length === 0) {
                tableBody.innerHTML = `<tr><td colspan="5" class="px-4 py-6 text-center text-slate-500">No matching logs found.</td></tr>`;
                return;
            }

            tableBody.innerHTML = filteredLogs.map(l => {
                const saved = l.original_tokens - l.filtered_tokens;
                const ratio = l.original_tokens > 0 ? ((saved / l.original_tokens) * 100).toFixed(0) : '0';
                
                // Format duration (e.g. 150ms or 2.1s)
                let durText = 'N/A';
                if (l.duration_ms !== null && l.duration_ms !== undefined) {
                    if (l.duration_ms < 1000) {
                        durText = `${l.duration_ms}ms`;
                    } else {
                        durText = `${(l.duration_ms / 1000).toFixed(1)}s`;
                    }
                }

                const modelText = l.model ? l.model.replace("anthropic/", "").replace("openai/", "") : 'Unknown';

                return `
                    <tr onclick="onSelectRow(${l.id})" class="hover:bg-slate-800/30 cursor-pointer transition-colors border-b border-slate-900/40 ${activeInspectorLogId === l.id ? 'bg-slate-800/40 border-l-2 border-l-emerald-400' : ''}">
                        <td class="px-4 py-3 font-semibold text-white truncate max-w-[200px]" title="${l.cmd}">${l.cmd}</td>
                        <td class="px-4 py-3 text-slate-400 truncate max-w-[120px]" title="${l.model || 'Unknown'}">${modelText}</td>
                        <td class="px-4 py-3 text-emerald-400 font-mono font-semibold">+${ratio}%</td>
                        <td class="px-4 py-3 text-slate-400 font-mono">${durText}</td>
                        <td class="px-4 py-3 text-slate-400 truncate max-w-[100px] font-medium" title="${l.branch}">${l.branch || 'detached'}</td>
                    </tr>
                `;
            }).join('');
        }

        // Render command category breakdown progress bars
        function renderBreakdown(breakdown) {
            const container = document.getElementById('breakdownContainer');
            if (breakdown.length === 0) {
                container.innerHTML = `<div class="text-center py-6 text-slate-500">No data available</div>`;
                return;
            }

            // Find maximum count for relative sizing
            const maxCount = Math.max(...breakdown.map(b => b.count));

            container.innerHTML = breakdown.map(b => {
                const percentWidth = maxCount > 0 ? (b.count / maxCount) * 100 : 0;
                return `
                    <div class="hover:bg-slate-800/10 p-2 rounded-xl transition-all">
                        <div class="flex justify-between items-center mb-1 text-sm">
                            <span class="font-bold text-slate-200 font-mono">${b.cmd}</span>
                            <span class="text-xs text-slate-400">${b.count} runs <span class="text-emerald-400 font-mono font-medium ml-1.5">+${formatNumber(b.saved)} saved</span></span>
                        </div>
                        <div class="w-full bg-slate-900 h-2 rounded-full overflow-hidden">
                            <div class="bg-gradient-to-r from-emerald-500 to-teal-400 h-full rounded-full" style="width: ${percentWidth}%"></div>
                        </div>
                    </div>
                `;
            }).join('');
        }

        function onSelectRow(id) {
            activeInspectorLogId = id;
            const log = cachedLogs.find(l => l.id === id);
            if (log) {
                inspectLog(log);
            }
        }

        function inspectLog(log) {
            document.getElementById('inspectorContainer').classList.remove('hidden');
            document.getElementById('inspect-cmd').innerText = log.cmd;
            document.getElementById('inspect-branch').innerText = log.branch || 'detached';
            document.getElementById('inspect-project').innerText = log.project || 'Unknown';
            document.getElementById('inspect-model').innerText = log.model || 'Unknown';
            
            // Format raw and filtered logs
            document.getElementById('inspect-raw').innerText = log.raw_output || '[No output cached]';
            
            // Reconstruct filtered output for display if needed
            document.getElementById('inspect-filtered').innerText = log.raw_output || '[No output cached]';

            const saved = log.original_tokens - log.filtered_tokens;
            const ratio = log.original_tokens > 0 ? ((saved / log.original_tokens) * 100).toFixed(0) : '0';
            
            document.getElementById('inspect-savings-badge').innerText = `-${ratio}% TOKENS SAVED`;

            // Scroll container to inspector
            document.getElementById('inspectorContainer').scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            
            // Rerender table to show selected row background
            renderLogs(cachedLogs);
        }

        // Generate and download JSON file of database records
        function downloadJsonBackup() {
            const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(cachedLogs, null, 2));
            const downloadAnchor = document.createElement('a');
            downloadAnchor.setAttribute("href", dataStr);
            downloadAnchor.setAttribute("download", `rtk_telemetry_backup_${new Date().toISOString().slice(0,10)}.json`);
            document.body.appendChild(downloadAnchor);
            downloadAnchor.click();
            downloadAnchor.remove();
        }

        // Initialize and update time-series chart
        function updateDailyChart(dailyData) {
            const labels = dailyData.map(d => d.day);
            const originalData = dailyData.map(d => d.original);
            const filteredData = dailyData.map(d => d.filtered);

            if (dailyChart) {
                dailyChart.data.labels = labels;
                dailyChart.data.datasets[0].data = originalData;
                dailyChart.data.datasets[1].data = filteredData;
                dailyChart.update();
                return;
            }

            const ctx = document.getElementById('dailyChart').getContext('2d');
            dailyChart = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: labels,
                    datasets: [
                        {
                            label: 'Original Tokens',
                            data: originalData,
                            borderColor: '#ef4444',
                            backgroundColor: 'rgba(239, 68, 68, 0.05)',
                            borderWidth: 2,
                            fill: true,
                            tension: 0.3
                        },
                        {
                            label: 'Filtered (RTK)',
                            data: filteredData,
                            borderColor: '#10b981',
                            backgroundColor: 'rgba(16, 185, 129, 0.08)',
                            borderWidth: 2.5,
                            fill: true,
                            tension: 0.3
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { labels: { color: '#94a3b8' } }
                    },
                    scales: {
                        y: {
                            grid: { color: 'rgba(255,255,255,0.03)' },
                            ticks: { color: '#94a3b8' }
                        },
                        x: {
                            grid: { display: false },
                            ticks: { color: '#94a3b8' }
                        }
                    }
                }
            });
        }

        // Initialize and update models distribution chart
        function updateModelsChart(modelsData) {
            const labels = modelsData.map(m => m.model.replace("anthropic/", "").replace("openai/", ""));
            const data = modelsData.map(m => m.saved);

            if (modelsChart) {
                modelsChart.data.labels = labels;
                modelsChart.data.datasets[0].data = data;
                modelsChart.update();
                return;
            }

            const ctx = document.getElementById('modelsChart').getContext('2d');
            modelsChart = new Chart(ctx, {
                type: 'doughnut',
                data: {
                    labels: labels,
                    datasets: [{
                        data: data,
                        backgroundColor: [
                            'rgba(16, 185, 129, 0.65)',
                            'rgba(6, 182, 212, 0.65)',
                            'rgba(99, 102, 241, 0.65)',
                            'rgba(139, 92, 246, 0.65)',
                            'rgba(236, 72, 153, 0.65)'
                        ],
                        borderColor: '#090d16',
                        borderWidth: 2
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { 
                            position: 'bottom',
                            labels: { color: '#94a3b8', font: { size: 11 } }
                        }
                    }
                }
            });
        }

        // Attach listener for realtime search filtration
        document.getElementById('logSearch').addEventListener('input', () => {
            renderLogs(cachedLogs);
        });

        // Initialize
        fetchMetrics();
        setInterval(fetchMetrics, 3000);
    </script>
</body>
</html>
"#
    .to_string())
}
