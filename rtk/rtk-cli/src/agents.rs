use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn agents_init(template: &str) -> Result<()> {
    let agents_path = Path::new("AGENTS.md");
    if agents_path.exists() {
        return Err(anyhow::anyhow!(
            "AGENTS.md already exists in the current directory."
        ));
    }

    let content = match template {
        "solo-dev" => {
            r#"# 🤖 Agent Guidelines (Solo Developer)

Welcome! You are pair programming with a solo developer. Follow these rules to maximize token efficiency and code quality:

## ⚡ Token Efficiency
- Apply the Ponytail philosophy: YAGNI, minimal code, deletion over addition.
- Avoid repeating context or generating long verbose explanations.
- Speak in caveman style when requested or to save tokens.

## 🛠️ Development Workflow
- Run local tests (`cargo test`) frequently.
- Build release targets to check binary sizes and warnings.
- Keep dependency surface small.
"#
        }
        "team" => {
            r#"# 👥 Agent Guidelines (Team Collaboration)

You are pair programming in a team workspace. Follow these rules to align with team conventions and code quality:

## ⚡ Team Alignment
- Follow Conventional Commits for all commit messages.
- Always run impact analysis on public symbol edits.
- Document public functions, types, and modules clearly.

## 🛠️ Testing & Quality
- Ensure all CI tests pass before proposing a PR.
- Add regression tests for all resolved bug tickets.
- Do not introduce breaking API changes without team consensus.
"#
        }
        "OSS" | "oss" => {
            r#"# 🌐 Agent Guidelines (Open Source Software)

You are assisting on an open-source project. Follow these rules to help sustain community collaboration:

## ⚡ Contribution & Standards
- Keep code clean, readable, and highly accessible to new contributors.
- Do not add proprietary dependencies.
- Update `CHANGELOG.md` and `README.md` for user-facing changes.

## 🛠️ Safety & Security
- Never expose API credentials or secrets in test suites.
- Perform sanity/security checks on external payloads.
- Write robust, cross-platform unit tests.
"#
        }
        "mono-repo" | "monorepo" => {
            r#"# 📦 Agent Guidelines (Monorepo Strategy)

You are working in a multi-crate/multi-package monorepo workspace. Follow these guidelines:

## ⚡ Workspace Boundaries
- Respect module boundaries and avoid tight coupling between crates.
- Ensure workspace `Cargo.toml` remains clean and dependencies are shared.
- Run tests on affected packages only to save time and resources.

## 🛠️ Build & Architecture
- Keep crate-level public APIs minimal.
- Use explicit path-based workspace references.
"#
        }
        other => {
            return Err(anyhow::anyhow!(
                "Unknown template: '{}'. Supported templates: solo-dev, team, OSS, mono-repo.",
                other
            ));
        }
    };

    fs::write(agents_path, content).context("Failed to write AGENTS.md")?;
    println!("✅ Created AGENTS.md using template '{}'", template);

    // Also create a basic CLAUDE.md companion if not present
    let claude_path = Path::new("CLAUDE.md");
    if !claude_path.exists() {
        let claude_content = r#"# CLAUDE.md - Developer Directives
- Follow rules in AGENTS.md.
- Build command: cargo build
- Test command: cargo test
"#;
        fs::write(claude_path, claude_content).context("Failed to write CLAUDE.md")?;
        println!("✅ Created companion CLAUDE.md");
    }

    Ok(())
}

pub fn agents_doctor() -> Result<()> {
    println!("🩺 Checking Agent Guidelines and Rules...");
    println!("==================================================");

    let mut issues = 0;

    let paths = ["AGENTS.md", "CLAUDE.md"];
    for name in &paths {
        let path = Path::new(name);
        print!("Checking {name}: ");
        if !path.exists() {
            println!("⚠️  Missing (optional but recommended)");
            continue;
        }
        println!("✅ Found");

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                println!("   ❌ Error reading file: {e}");
                issues += 1;
                continue;
            }
        };

        // Check file size (too long rules waste input tokens)
        let lines = content.lines().count();
        if lines > 300 {
            println!("   ⚠️  File is large ({} lines). Large rule files consume too many LLM context tokens.", lines);
            println!("      👉 Suggestion: Run `rtk agents compact` to compress this file.");
            issues += 1;
        }

        // Check for broken markdown links (e.g., [name](file:///path))
        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let mut start = 0;
            while let Some(pos) = line[start..].find("file:///") {
                let absolute_pos = start + pos;
                let link_start = absolute_pos + "file:///".len();
                let link_end = line[link_start..]
                    .find([')', '"', ' ', '#'])
                    .map(|p| link_start + p)
                    .unwrap_or(line.len());

                let raw_path = &line[link_start..link_end];
                let cleaned_path = raw_path.replace('\\', "/");
                let p = Path::new(&cleaned_path);
                if !p.exists() {
                    println!(
                        "   ⚠️  Line {}: Broken file reference: '{}'",
                        line_num, raw_path
                    );
                    issues += 1;
                }

                start = link_end;
            }
        }
    }

    println!("==================================================");
    if issues == 0 {
        println!("✅ No issues found in agent guideline files!");
    } else {
        println!("⚠️  Found {} issue(s)/warning(s) in agent files.", issues);
    }

    Ok(())
}

pub fn agents_compact() -> Result<()> {
    let paths = ["AGENTS.md", "CLAUDE.md"];
    let mut compacted_any = false;

    for name in &paths {
        let path = Path::new(name);
        if !path.exists() {
            continue;
        }

        let original_content = fs::read_to_string(path)?;
        let original_len = original_content.len();

        let mut lines = Vec::new();
        let mut in_comment = false;

        for line in original_content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
                continue;
            }
            if trimmed.starts_with("<!--") {
                in_comment = true;
                continue;
            }
            if in_comment {
                if trimmed.ends_with("-->") {
                    in_comment = false;
                }
                continue;
            }

            lines.push(line.trim_end());
        }

        let mut collapsed_lines = Vec::new();
        let mut prev_was_blank = false;
        for line in lines {
            let is_blank = line.is_empty();
            if is_blank {
                if !prev_was_blank {
                    collapsed_lines.push(line);
                }
                prev_was_blank = true;
            } else {
                collapsed_lines.push(line);
                prev_was_blank = false;
            }
        }

        let compacted_content = collapsed_lines.join("\n");
        let compacted_len = compacted_content.len();

        if compacted_len < original_len {
            let backup_path = path.with_extension("original.md");
            fs::write(&backup_path, &original_content)?;
            fs::write(path, &compacted_content)?;

            let saved_chars = original_len - compacted_len;
            let saved_tokens = saved_chars / 4;
            println!(
                "✅ Compacted {} in place (saved {} chars, ~{} tokens). Backup saved to {}",
                name,
                saved_chars,
                saved_tokens,
                backup_path.display()
            );
            compacted_any = true;
        } else {
            println!("ℹ️  {} is already fully compacted.", name);
        }
    }

    if !compacted_any {
        println!("No changes made (files already optimized).");
    }

    Ok(())
}
