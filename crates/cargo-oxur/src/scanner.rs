//! File scanner for finding Oxur source files

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scanner for finding Oxur files in a project
pub struct OxurScanner {
    root: PathBuf,
}

impl OxurScanner {
    /// Create a new scanner for the given project root
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Find all Oxur files in the project
    ///
    /// Scans for files with extensions: .oxr, .oxur, .ox, .lisp
    /// Excludes: target/, .git/, and hidden directories
    pub fn find_oxur_files(&self) -> Result<Vec<PathBuf>> {
        let mut oxur_files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.is_excluded_relative(e.path()))
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_oxur_file(path) {
                oxur_files.push(path.to_path_buf());
            }
        }

        Ok(oxur_files)
    }

    /// Check if a path should be excluded from scanning (relative to root)
    fn is_excluded_relative(&self, path: &Path) -> bool {
        // Get path relative to root
        let relative = match path.strip_prefix(&self.root) {
            Ok(rel) => rel,
            Err(_) => return false, // If we can't get relative path, don't exclude
        };

        // Check if any component in the relative path should be excluded
        for component in relative.components() {
            if let Some(name) = component.as_os_str().to_str() {
                // Exclude target directory
                if name == "target" {
                    return true;
                }

                // Exclude .git directory
                if name == ".git" {
                    return true;
                }

                // Exclude other hidden directories (but allow . and ..)
                if name.starts_with('.') && name != "." && name != ".." {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a path should be excluded from scanning (for testing)
    #[allow(dead_code)]
    fn is_excluded(path: &Path) -> bool {
        // Check if any component in the path is a target or .git directory
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                // Exclude target directory
                if name == "target" {
                    return true;
                }

                // Exclude .git directory
                if name == ".git" {
                    return true;
                }

                // Exclude other hidden directories (but allow . and ..)
                if name.starts_with('.') && name != "." && name != ".." {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a file is an Oxur source file
    fn is_oxur_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            matches!(ext_str.as_ref(), "oxr" | "oxur" | "ox" | "lisp")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_oxur_file() {
        assert!(OxurScanner::is_oxur_file(Path::new("test.oxr")));
        assert!(OxurScanner::is_oxur_file(Path::new("test.oxur")));
        assert!(OxurScanner::is_oxur_file(Path::new("test.ox")));
        assert!(OxurScanner::is_oxur_file(Path::new("test.lisp")));
        assert!(!OxurScanner::is_oxur_file(Path::new("test.rs")));
        assert!(!OxurScanner::is_oxur_file(Path::new("test.txt")));
        assert!(!OxurScanner::is_oxur_file(Path::new("test")));
    }

    #[test]
    fn test_is_excluded() {
        assert!(OxurScanner::is_excluded(Path::new("target")));
        assert!(OxurScanner::is_excluded(Path::new("target/debug")));
        assert!(OxurScanner::is_excluded(Path::new(".git")));
        assert!(OxurScanner::is_excluded(Path::new(".git/objects")));
        assert!(!OxurScanner::is_excluded(Path::new("src")));
        assert!(!OxurScanner::is_excluded(Path::new("src/main.oxr")));
    }

    #[test]
    fn test_find_oxur_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create test structure
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("target/debug"))?;
        fs::write(root.join("src/lib.oxr"), "(deffn test ())")?;
        fs::write(root.join("src/main.oxr"), "(deffn main ())")?;
        fs::write(root.join("src/utils.rs"), "fn test() {}")?;
        fs::write(root.join("target/debug/output.rs"), "// generated")?;

        let scanner = OxurScanner::new(root);
        let files = scanner.find_oxur_files()?;

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("lib.oxr")));
        assert!(files.iter().any(|f| f.ends_with("main.oxr")));
        assert!(!files.iter().any(|f| f.ends_with("utils.rs")));
        assert!(!files.iter().any(|f| f.ends_with("output.rs")));

        Ok(())
    }
}
