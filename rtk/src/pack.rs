use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Packs directory contents into a single XML representation.
/// Respects basic ignore patterns (node_modules, target, .git, etc.)
/// and only includes text files.
pub fn pack_directory(dir_path: &Path) -> Result<String> {
    let mut out = String::new();
    out.push_str("<repository>\n");

    let canonical_root = dir_path.canonicalize()
        .with_context(|| format!("failed to canonicalize root path: {}", dir_path.display()))?;

    pack_recursive(&canonical_root, &canonical_root, &mut out)?;

    out.push_str("</repository>\n");
    Ok(out)
}

fn pack_recursive(root: &Path, current: &Path, out: &mut String) -> Result<()> {
    if current.is_dir() {
        let dir_name = current.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
            
        // Standard folder ignore list
        if dir_name == ".git"
            || dir_name == "node_modules"
            || dir_name == "target"
            || dir_name == "dist"
            || dir_name == ".venv"
            || dir_name == ".pytest_cache"
            || dir_name == "__pycache__"
            || dir_name == ".idea"
            || dir_name == ".vscode"
            || dir_name == "build"
        {
            return Ok(());
        }

        for entry in fs::read_dir(current)? {
            let entry = entry?;
            pack_recursive(root, &entry.path(), out)?;
        }
    } else if current.is_file() {
        if is_binary_file(current) {
            return Ok(());
        }

        let relative_path = current.strip_prefix(root)
            .unwrap_or(current)
            .to_string_lossy()
            .replace('\\', "/");

        if let Ok(content) = fs::read_to_string(current) {
            // Write to XML format
            out.push_str(&format!("  <file path=\"{relative_path}\">\n"));
            out.push_str("    <![CDATA[\n");
            out.push_str(&content);
            if !content.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("    ]]>\n");
            out.push_str("  </file>\n");
        }
    }
    Ok(())
}

fn is_binary_file(path: &Path) -> bool {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Standard binary extension list
    matches!(
        ext.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "ico"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "zip"
            | "tar"
            | "gz"
            | "pdf"
            | "db"
            | "sqlite"
            | "bin"
            | "woff"
            | "woff2"
            | "ttf"
            | "eot"
            | "mp3"
            | "mp4"
            | "wav"
            | "avi"
            | "mov"
            | "dmg"
            | "iso"
            | "lock"
            | "pyc"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_pack_directory() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_pack_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a couple of files
        let file1_path = temp_dir.join("main.rs");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, "fn main() {{}}").unwrap();

        // Create an ignored directory and a file inside it
        let ignored_dir = temp_dir.join("target");
        fs::create_dir_all(&ignored_dir).unwrap();
        let file2_path = ignored_dir.join("output.bin");
        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(&[0, 1, 2, 3]).unwrap();

        // Pack the directory
        let packed = pack_directory(&temp_dir).unwrap();
        assert!(packed.contains("<repository>"));
        assert!(packed.contains("<file path=\"main.rs\">"));
        assert!(packed.contains("fn main() {}"));
        assert!(!packed.contains("output.bin"), "should ignore target/ files");

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }
}
