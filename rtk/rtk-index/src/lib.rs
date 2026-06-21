use anyhow::Result;
use std::path::Path;

pub mod parser;
pub mod db;
pub mod graph;
pub mod embeddings;


pub fn index_project(project_dir: &Path) -> Result<usize> {
    let files = parser::scan_directory(project_dir)?;
    let mut all_symbols = Vec::new();
    for file in files {
        if let Ok(syms) = parser::parse_file(&file, project_dir) {
            all_symbols.extend(syms);
        }
    }
    
    let conn = db::open_db()?;
    db::clear_index(&conn)?;
    db::insert_symbols(&conn, &all_symbols)?;
    
    Ok(all_symbols.len())
}

pub fn query_symbols(name_query: &str) -> Result<Vec<db::DbSymbol>> {
    let conn = db::open_db()?;
    db::find_symbols(&conn, name_query)
}

pub fn query_dependencies(file_path: &str) -> Result<Vec<(db::DbSymbol, Vec<String>)>> {
    let conn = db::open_db()?;
    let all_syms = db::get_all_symbols(&conn)?;
    let all_deps = db::get_all_dependencies(&conn)?;
    
    let mut file_symbols = Vec::new();
    for sym in all_syms {
        if sym.file_path == file_path {
            let mut callees = Vec::new();
            for dep in &all_deps {
                if dep.caller_id == sym.id {
                    callees.push(dep.callee_name.clone());
                }
            }
            file_symbols.push((sym, callees));
        }
    }
    Ok(file_symbols)
}

pub fn query_references(symbol_name: &str) -> Result<Vec<db::DbSymbol>> {
    let conn = db::open_db()?;
    db::get_symbol_references(&conn, symbol_name)
}

pub fn analyze_impact(symbol_name: &str) -> Result<Vec<db::DbSymbol>> {
    let conn = db::open_db()?;
    let all_syms = db::get_all_symbols(&conn)?;
    let all_deps = db::get_all_dependencies(&conn)?;
    
    let target_ids: Vec<String> = all_syms.iter().filter(|s| s.name == symbol_name).map(|s| s.id.clone()).collect();
    if target_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    let impact_graph = graph::ImpactGraph::build(all_syms, all_deps);
    
    let mut affected_ids = std::collections::HashSet::new();
    for target_id in target_ids {
        let upstream = impact_graph.resolve_upstream(&target_id);
        for u in upstream {
            affected_ids.insert(u.id);
        }
    }
    
    let conn2 = db::open_db()?;
    let reloaded = db::get_all_symbols(&conn2)?;
    let result = reloaded.into_iter().filter(|s| affected_ids.contains(&s.id)).collect();
    
    Ok(result)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphMetrics {
    pub symbols_count: usize,
    pub edges_count: usize,
    pub query_latency_ms: f64,
    pub graph_coverage: f64,
}

pub fn export_obsidian_graph(output_dir: &Path) -> Result<usize> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }
    
    let conn = db::open_db()?;
    let symbols = db::get_all_symbols(&conn)?;
    let dependencies = db::get_all_dependencies(&conn)?;
    
    let mut symbol_map = std::collections::HashMap::new();
    for sym in &symbols {
        symbol_map.insert(sym.id.clone(), sym.clone());
    }
    
    let mut outgoing: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for dep in &dependencies {
        outgoing.entry(dep.caller_id.clone()).or_default().push(dep.callee_name.clone());
    }
    
    let mut incoming: std::collections::HashMap<String, Vec<db::DbSymbol>> = std::collections::HashMap::new();
    for dep in &dependencies {
        if let Some(caller_sym) = symbol_map.get(&dep.caller_id) {
            incoming.entry(dep.callee_name.clone()).or_default().push(caller_sym.clone());
        }
    }
    
    let mut files_written = 0;
    
    for sym in &symbols {
        let file_name = format!(
            "{} ({}).md",
            sym.name,
            Path::new(&sym.file_path)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string())
        );
        let file_path = output_dir.join(&file_name);
        
        let mut md = String::new();
        md.push_str(&format!("# Symbol: {}\n\n", sym.name));
        md.push_str(&format!("- **Kind:** {}\n", sym.kind));
        md.push_str(&format!("- **Location:** `{}:{}-{}`\n\n", sym.file_path, sym.line_start, sym.line_end));
        
        md.push_str("## Calls (Outgoing)\n");
        if let Some(callees) = outgoing.get(&sym.id) {
            let mut unique_callees = callees.clone();
            unique_callees.sort();
            unique_callees.dedup();
            for callee in unique_callees {
                let mut links = Vec::new();
                for other in &symbols {
                    if other.name == callee {
                        let other_file = Path::new(&other.file_path)
                            .file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "unknown".to_string());
                        links.push(format!("[[{} ({})]]", other.name, other_file));
                    }
                }
                if links.is_empty() {
                    md.push_str(&format!("- [[{}]]\n", callee));
                } else {
                    for l in links {
                        md.push_str(&format!("- {}\n", l));
                    }
                }
            }
        } else {
            md.push_str("- None\n");
        }
        md.push_str("\n");
        
        md.push_str("## Referenced By (Incoming)\n");
        if let Some(callers) = incoming.get(&sym.name) {
            let mut unique_callers = callers.clone();
            unique_callers.sort_by(|a, b| a.id.cmp(&b.id));
            unique_callers.dedup_by(|a, b| a.id == b.id);
            for caller in unique_callers {
                let caller_file = Path::new(&caller.file_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "unknown".to_string());
                md.push_str(&format!("- [[{} ({})]]\n", caller.name, caller_file));
            }
        } else {
            md.push_str("- None\n");
        }
        
        std::fs::write(file_path, md)?;
        files_written += 1;
    }
    
    Ok(files_written)
}

pub fn get_graph_metrics() -> Result<GraphMetrics> {
    let conn = db::open_db()?;
    let symbols = db::get_all_symbols(&conn)?;
    let dependencies = db::get_all_dependencies(&conn)?;
    
    let symbols_count = symbols.len();
    let edges_count = dependencies.len();
    
    let mut connected_ids = std::collections::HashSet::new();
    for dep in &dependencies {
        connected_ids.insert(dep.caller_id.clone());
        for sym in &symbols {
            if sym.name == dep.callee_name {
                connected_ids.insert(sym.id.clone());
            }
        }
    }
    
    let graph_coverage = if symbols_count > 0 {
        (connected_ids.len() as f64 / symbols_count as f64) * 100.0
    } else {
        0.0
    };
    
    let start = std::time::Instant::now();
    let _ = db::find_symbols(&conn, "dummy_nonexistent_symbol")?;
    let query_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    
    Ok(GraphMetrics {
        symbols_count,
        edges_count,
        query_latency_ms,
        graph_coverage,
    })
}

#[allow(unused_variables)]
pub fn query_hybrid(
    query: &str,
    model_path: Option<&Path>,
    tokenizer_path: Option<&Path>,
    alpha: f32,
    limit: usize,
) -> Result<Vec<(db::DbSymbol, f64)>> {
    let conn = db::open_db()?;
    let db_symbols = db::find_symbols(&conn, query)?;

    #[cfg(feature = "embeddings")]
    {
        if let (Some(m_path), Some(t_path)) = (model_path, tokenizer_path) {
            if m_path.exists() && t_path.exists() {
                let embedder = embeddings::OnnxEmbedder::load_model(m_path, t_path)?;
                let query_embedding = embedder.embed_text(query)?;

                let all_symbols = db::get_all_symbols(&conn)?;
                let mut scored_symbols = Vec::new();

                for sym in all_symbols {
                    let sym_text = format!("{} {} {}", sym.kind, sym.name, sym.file_path);
                    if let Ok(sym_emb) = embedder.embed_text(&sym_text) {
                        let sem_score = embeddings::dot_product(&query_embedding, &sym_emb) as f64;
                        let lex_score = if sym.name.to_lowercase().contains(&query.to_lowercase()) {
                            1.0
                        } else {
                            0.0
                        };

                        let combined_score = alpha as f64 * lex_score + (1.0 - alpha as f64) * sem_score;
                        scored_symbols.push((sym, combined_score));
                    }
                }

                scored_symbols.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored_symbols.truncate(limit);
                return Ok(scored_symbols);
            }
        }
    }

    let mut results = Vec::new();
    for s in db_symbols {
        results.push((s, 1.0));
    }
    results.truncate(limit);
    Ok(results)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_index_and_query_lifecycle() {
        let tmp_db = env::temp_dir().join(format!("rtk_index_test_{}.db", std::process::id()));
        env::set_var("RTK_INDEX_DB_PATH", &tmp_db);

        let temp_project = env::temp_dir().join(format!("rtk_index_proj_{}", std::process::id()));
        fs::create_dir_all(&temp_project).unwrap();

        let code_rs = r#"
            struct Config {
                port: u16,
            }
            fn main() {
                let cfg = Config { port: 80 };
                setup_logger();
            }
            fn setup_logger() {
                println!("logging");
            }
        "#;
        fs::write(temp_project.join("main.rs"), code_rs).unwrap();

        let count = index_project(&temp_project).unwrap();
        assert_eq!(count, 3);

        let syms = query_symbols("main").unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "main");
        assert_eq!(syms[0].kind, "Function");

        let refs = query_references("setup_logger").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "main");

        let impact = analyze_impact("setup_logger").unwrap();
        assert_eq!(impact.len(), 1);
        assert_eq!(impact[0].name, "main");

        // Test export_obsidian_graph
        let obsidian_dir = temp_project.join("obsidian");
        let exported_count = export_obsidian_graph(&obsidian_dir).unwrap();
        assert_eq!(exported_count, 3);
        let exported_file = obsidian_dir.join("main (main.rs).md");
        assert!(exported_file.exists());
        let md_content = fs::read_to_string(exported_file).unwrap();
        assert!(md_content.contains("# Symbol: main"));
        assert!(md_content.contains("[[setup_logger (main.rs)]]"));

        // Test get_graph_metrics
        let metrics = get_graph_metrics().unwrap();
        assert_eq!(metrics.symbols_count, 3);
        assert_eq!(metrics.edges_count, 1);
        assert!(metrics.graph_coverage > 0.0);

        env::remove_var("RTK_INDEX_DB_PATH");
        fs::remove_file(&tmp_db).ok();
        fs::remove_dir_all(&temp_project).ok();
    }
}

