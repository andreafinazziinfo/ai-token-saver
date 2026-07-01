use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct ParsedSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub calls: Vec<String>,
}

fn read_gitignore(root: &Path) -> Vec<String> {
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        if let Ok(content) = fs::read_to_string(gitignore_path) {
            return content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
        }
    }
    Vec::new()
}

fn is_ignored(path: &Path, root: &Path, patterns: &[String]) -> bool {
    let rel_path = path.strip_prefix(root).unwrap_or(path);
    let path_str = rel_path.to_string_lossy().replace('\\', "/");

    for sys in &[
        "target",
        ".git",
        "node_modules",
        ".rtk",
        "dist",
        "build",
        "venv",
        ".venv",
        "__pycache__",
        ".cargo",
        ".venvs",
    ] {
        if path_str.split('/').any(|part| part == *sys) {
            return true;
        }
    }

    for pat in patterns {
        let pat_clean = pat.trim_start_matches('/').trim_end_matches('/');
        if pat_clean.is_empty() {
            continue;
        }
        if path_str.contains(pat_clean) {
            return true;
        }
    }

    false
}

fn scan_dir_rec(
    dir: &Path,
    root: &Path,
    patterns: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if is_ignored(dir, root, patterns) {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_rec(&path, root, patterns, files)?;
        } else {
            if !is_ignored(&path, root, patterns) {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(ext, "rs" | "py" | "go" | "ts" | "tsx" | "js" | "jsx") {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let patterns = read_gitignore(dir);
    scan_dir_rec(dir, dir, &patterns, &mut files)?;
    Ok(files)
}

fn get_identifier_text(node: Node, code: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" | "field_identifier" => Some(node.utf8_text(code).unwrap_or("").to_string()),
        "field_expression" | "attribute" => {
            if let Some(field) = node.child_by_field_name("field") {
                Some(field.utf8_text(code).unwrap_or("").to_string())
            } else if let Some(attribute) = node.child_by_field_name("attribute") {
                Some(attribute.utf8_text(code).unwrap_or("").to_string())
            } else {
                Some(node.utf8_text(code).unwrap_or("").to_string())
            }
        }
        _ => {
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    if let Some(res) = get_identifier_text(cursor.node(), code) {
                        return Some(res);
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            None
        }
    }
}

fn traverse_ast<'a>(
    node: Node<'a>,
    code: &[u8],
    file_path: &str,
    ext: &str,
    symbols: &mut Vec<ParsedSymbol>,
    current_symbol: &mut Option<ParsedSymbol>,
) {
    let kind = node.kind();
    let mut is_symbol = false;
    let mut symbol_name = String::new();
    let mut symbol_kind = String::new();

    match ext {
        "rs" => match kind {
            "function_item" => {
                is_symbol = true;
                symbol_kind = "Function".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "struct_item" => {
                is_symbol = true;
                symbol_kind = "Struct".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "enum_item" => {
                is_symbol = true;
                symbol_kind = "Enum".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "trait_item" => {
                is_symbol = true;
                symbol_kind = "Trait".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            _ => {}
        },
        "py" => match kind {
            "function_definition" => {
                is_symbol = true;
                symbol_kind = "Function".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "class_definition" => {
                is_symbol = true;
                symbol_kind = "Class".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            _ => {}
        },
        "go" => match kind {
            "function_declaration" => {
                is_symbol = true;
                symbol_kind = "Function".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "method_declaration" => {
                is_symbol = true;
                symbol_kind = "Method".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "type_spec" => {
                is_symbol = true;
                symbol_kind = "Struct".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            _ => {}
        },
        "ts" | "tsx" | "js" | "jsx" => match kind {
            "function_declaration" => {
                is_symbol = true;
                symbol_kind = "Function".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "class_declaration" => {
                is_symbol = true;
                symbol_kind = "Class".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            "method_definition" => {
                is_symbol = true;
                symbol_kind = "Method".to_string();
                if let Some(n) = node.child_by_field_name("name") {
                    symbol_name = n.utf8_text(code).unwrap_or("").to_string();
                }
            }
            _ => {}
        },
        _ => {}
    }

    let mut new_sym = None;
    if is_symbol && !symbol_name.is_empty() {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        let id = format!("{}:{}:{}", symbol_kind, file_path, symbol_name);

        new_sym = Some(ParsedSymbol {
            id,
            name: symbol_name,
            kind: symbol_kind,
            file_path: file_path.to_string(),
            line_start: start_pos.row + 1,
            line_end: end_pos.row + 1,
            calls: Vec::new(),
        });
    }

    if let Some(ns) = new_sym {
        if let Some(os) = current_symbol.take() {
            symbols.push(os);
        }
        *current_symbol = Some(ns);
    }

    if current_symbol.is_some() {
        let is_call = match ext {
            "rs" | "go" | "ts" | "tsx" | "js" | "jsx" => kind == "call_expression",
            "py" => kind == "call",
            _ => false,
        };

        if is_call {
            let callee_name = if let Some(func_node) = node.child_by_field_name("function") {
                get_identifier_text(func_node, code)
            } else {
                None
            };

            if let Some(cname) = callee_name {
                if let Some(ref mut sym) = current_symbol {
                    if !sym.calls.contains(&cname) && cname != sym.name {
                        sym.calls.push(cname);
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            traverse_ast(cursor.node(), code, file_path, ext, symbols, current_symbol);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

pub fn parse_file(path: &Path, root: &Path) -> Result<Vec<ParsedSymbol>> {
    let code = fs::read_to_string(path)?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut parser = Parser::new();
    let lang = match ext {
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "ts" | "tsx" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "js" | "jsx" => tree_sitter_javascript::LANGUAGE.into(),
        _ => return Ok(Vec::new()),
    };

    parser
        .set_language(&lang)
        .context("Failed to set tree-sitter language")?;
    let tree = parser.parse(&code, None).context("Failed to parse code")?;

    let rel_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
        .replace('\\', "/");
    let code_bytes = code.as_bytes();

    let mut symbols = Vec::new();
    let mut current_symbol = None;

    traverse_ast(
        tree.root_node(),
        code_bytes,
        &rel_path,
        ext,
        &mut symbols,
        &mut current_symbol,
    );

    if let Some(sym) = current_symbol {
        symbols.push(sym);
    }

    Ok(symbols)
}

/// A single identifier occurrence in a file (0-based byte offsets, 1-based line).
#[derive(Debug, Clone)]
pub struct IdentSite {
    pub line: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Find every identifier-leaf node in `path` whose text equals `name`.
/// AST-aware: only matches identifier tokens (never strings/comments), so it is
/// safe for renames — unlike a raw text search. Returns [] for unsupported files.
pub fn find_identifier_sites(path: &Path, name: &str) -> Result<Vec<IdentSite>> {
    let code = fs::read_to_string(path)?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut parser = Parser::new();
    let lang = match ext {
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "ts" | "tsx" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "js" | "jsx" => tree_sitter_javascript::LANGUAGE.into(),
        _ => return Ok(Vec::new()),
    };
    parser
        .set_language(&lang)
        .context("Failed to set tree-sitter language")?;
    let Some(tree) = parser.parse(&code, None) else {
        return Ok(Vec::new());
    };

    let code_bytes = code.as_bytes();
    let mut sites = Vec::new();
    collect_identifier_sites(tree.root_node(), code_bytes, name, &mut sites);
    Ok(sites)
}

fn collect_identifier_sites(node: Node, code: &[u8], name: &str, out: &mut Vec<IdentSite>) {
    // Identifier leaves: kind ends with "identifier" (identifier, field_identifier,
    // type_identifier, property_identifier, ...) and has no named children.
    if node.child_count() == 0 && node.kind().ends_with("identifier") {
        if node.utf8_text(code).unwrap_or("") == name {
            out.push(IdentSite {
                line: node.start_position().row + 1,
                byte_start: node.start_byte(),
                byte_end: node.end_byte(),
            });
        }
        return;
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            collect_identifier_sites(cursor.node(), code, name, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_identifier_sites_ast_aware() {
        let dir = std::env::temp_dir().join(format!("rtk_rename_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("sample.rs");
        // `foo` appears as: a fn name, a call, inside a string, and in a comment.
        // Only the two identifier occurrences must be found.
        let code = "fn foo() {}\n// call foo here\nfn main() { let s = \"foo\"; foo(); }\n";
        std::fs::write(&file, code).unwrap();

        let sites = find_identifier_sites(&file, "foo").unwrap();
        assert_eq!(
            sites.len(),
            2,
            "should match identifiers only, not string/comment"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
