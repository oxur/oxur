# Phase 7: Advanced Document Onboarding - Detailed Implementation Guide

## Overview
Transform the `add` command into a sophisticated document onboarding system that can intelligently process any markdown file from anywhere on the filesystem, with smart defaults, user prompts, and comprehensive sanitization.

**Prerequisites:** Phases 1-6 should be complete

---

## Problem Statement

### Current Limitations

The Phase 4 `add` command does basic processing, but doesn't handle:
- Messy filenames (spaces, special chars, unicode)
- Missing or incomplete metadata (no interactive prompting)
- Content quality issues (broken markdown, inconsistent formatting)
- Multiple files at once (batch operations)
- Preview before commit (show what will change)

### Real-World Scenarios

```bash
# User has a design doc from their personal notes
~/Documents/My Cool Feature Idea!!!.md

# Should become:
docs/01-draft/0012-my-cool-feature-idea.md

# With proper:
# - Number prefix
# - Sanitized filename
# - Complete YAML frontmatter
# - Normalized markdown
```

---

## Task 7.1: Filename Sanitization

### Purpose
Clean and normalize filenames to be filesystem-friendly and consistent.

### Implementation Steps

#### Step 1: Create Filename Utilities Module
File: `design/src/filename.rs`

```rust
//! Filename sanitization and normalization

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Sanitize a filename to be filesystem-friendly
pub fn sanitize_filename(name: &str) -> String {
    // Remove extension if present
    let stem = if let Some(pos) = name.rfind('.') {
        &name[..pos]
    } else {
        name
    };
    
    // Remove number prefix if present (we'll add our own)
    let re = Regex::new(r"^\d{4}-").unwrap();
    let without_number = re.replace(stem, "");
    
    // Normalize unicode to NFD form
    let normalized: String = without_number.nfd().collect();
    
    // Convert to lowercase
    let mut result = normalized.to_lowercase();
    
    // Replace spaces and underscores with hyphens
    result = result.replace(' ', "-");
    result = result.replace('_', "-");
    
    // Remove special characters (keep alphanumeric and hyphens)
    let re = Regex::new(r"[^a-z0-9-]").unwrap();
    result = re.replace_all(&result, "").to_string();
    
    // Collapse multiple hyphens
    let re = Regex::new(r"-+").unwrap();
    result = re.replace_all(&result, "-").to_string();
    
    // Trim hyphens from start and end
    result = result.trim_matches('-').to_string();
    
    // Enforce maximum length (filesystem limit is usually 255, use 100 for safety)
    if result.len() > 100 {
        result.truncate(100);
        result = result.trim_matches('-').to_string();
    }
    
    // Ensure not empty
    if result.is_empty() {
        result = "untitled".to_string();
    }
    
    result
}

/// Build filename with number prefix
pub fn build_filename(number: u32, title: &str) -> String {
    let sanitized = sanitize_filename(title);
    format!("{:04}-{}.md", number, sanitized)
}

/// Extract title-like string from filename
pub fn filename_to_title(filename: &str) -> String {
    // Remove extension
    let stem = if let Some(pos) = filename.rfind('.') {
        &filename[..pos]
    } else {
        filename
    };
    
    // Remove number prefix
    let re = Regex::new(r"^\d{4}-").unwrap();
    let without_number = re.replace(stem, "");
    
    // Replace hyphens and underscores with spaces
    let with_spaces = without_number.replace('-', " ").replace('_', " ");
    
    // Title case each word
    with_spaces
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_basic() {
        assert_eq!(sanitize_filename("My Cool Feature"), "my-cool-feature");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize_filename("Feature!!!"), "feature");
        assert_eq!(sanitize_filename("my_feature_name"), "my-feature-name");
    }

    #[test]
    fn test_sanitize_unicode() {
        assert_eq!(sanitize_filename("Café"), "cafe");
        assert_eq!(sanitize_filename("naïve"), "naive");
    }

    #[test]
    fn test_sanitize_multiple_hyphens() {
        assert_eq!(sanitize_filename("my---feature"), "my-feature");
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_filename("!!!"), "untitled");
    }

    #[test]
    fn test_filename_to_title() {
        assert_eq!(filename_to_title("0001-my-feature.md"), "My Feature");
        assert_eq!(filename_to_title("my_cool_thing.md"), "My Cool Thing");
    }
}
```

#### Step 2: Add Dependencies
File: `design/Cargo.toml`

```toml
[dependencies]
unicode-normalization = "0.1"
```

#### Step 3: Export Module
File: `design/src/lib.rs`

```rust
pub mod filename;
```

### Testing Steps for 7.1
1. Test with spaces and underscores
2. Test with special characters
3. Test with unicode characters
4. Test with very long filenames
5. Test with empty/all-special-char filenames
6. Test filename_to_title conversion

---

## Task 7.2: Interactive Metadata Prompting

### Purpose
Prompt user for missing metadata fields with smart defaults.

### Implementation Steps

#### Step 1: Create Interactive Prompt Module
File: `design/src/prompt.rs`

```rust
//! Interactive prompting for user input

use anyhow::Result;
use std::io::{self, Write};

/// Prompt user for input with a default value
pub fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", message, default);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt user for input (required)
pub fn prompt_required(message: &str) -> Result<String> {
    loop {
        print!("{}: ", message);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        
        println!("This field is required. Please enter a value.");
    }
}

/// Prompt user to select from options
pub fn prompt_select(message: &str, options: &[&str], default_idx: usize) -> Result<String> {
    println!("{}", message);
    for (idx, opt) in options.iter().enumerate() {
        let marker = if idx == default_idx { "*" } else { " " };
        println!("  {}{}) {}", marker, idx + 1, opt);
    }
    
    print!("Select [{}]: ", default_idx + 1);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(options[default_idx].to_string());
    }
    
    if let Ok(idx) = trimmed.parse::<usize>() {
        if idx > 0 && idx <= options.len() {
            return Ok(options[idx - 1].to_string());
        }
    }
    
    println!("Invalid selection, using default.");
    Ok(options[default_idx].to_string())
}

/// Prompt user for yes/no confirmation
pub fn prompt_confirm(message: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", message, default_str);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.starts_with('y'))
    }
}
```

#### Step 2: Export Module
File: `design/src/lib.rs`

```rust
pub mod prompt;
```

### Testing Steps for 7.2
1. Test prompt_with_default with user input
2. Test prompt_with_default with just Enter (default)
3. Test prompt_required (refuses empty)
4. Test prompt_select with valid/invalid input
5. Test prompt_confirm with y/n/Enter

---

## Task 7.3: Smart Metadata Extraction

### Purpose
Extract or infer metadata from document content intelligently.

### Implementation Steps

#### Step 1: Create Metadata Extraction Module
File: `design/src/extract.rs`

```rust
//! Metadata extraction from document content

use crate::doc::DocState;
use chrono::NaiveDate;
use regex::Regex;

/// Extracted metadata from a document
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub state_hint: Option<DocState>,
    pub has_frontmatter: bool,
    pub first_heading: Option<String>,
}

impl ExtractedMetadata {
    /// Extract metadata from document content
    pub fn from_content(content: &str) -> Self {
        let mut meta = ExtractedMetadata {
            title: None,
            author: None,
            state_hint: None,
            has_frontmatter: content.trim_start().starts_with("---\n"),
            first_heading: None,
        };
        
        // Skip frontmatter if present
        let content_start = if meta.has_frontmatter {
            if let Some(end) = content[4..].find("\n---\n") {
                end + 8
            } else {
                0
            }
        } else {
            0
        };
        
        let body = &content[content_start..];
        
        // Extract first H1 heading
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let heading = trimmed[2..].trim().to_string();
                meta.first_heading = Some(heading.clone());
                if meta.title.is_none() {
                    meta.title = Some(heading);
                }
                break;
            }
        }
        
        // Look for author hints in content
        let author_patterns = [
            r"(?i)author:\s*(.+)",
            r"(?i)by\s+(.+)",
            r"(?i)written by\s+(.+)",
        ];
        
        for pattern in &author_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(caps) = re.captures(body) {
                if let Some(author_match) = caps.get(1) {
                    meta.author = Some(author_match.as_str().trim().to_string());
                    break;
                }
            }
        }
        
        // Detect state hints from content
        meta.state_hint = detect_state_hint(body);
        
        meta
    }
}

/// Detect likely state from document content
fn detect_state_hint(content: &str) -> Option<DocState> {
    let lower = content.to_lowercase();
    
    // Look for explicit state markers
    if lower.contains("work in progress") || lower.contains("wip") {
        return Some(DocState::Draft);
    }
    
    if lower.contains("ready for review") || lower.contains("please review") {
        return Some(DocState::UnderReview);
    }
    
    if lower.contains("approved") || lower.contains("accepted") {
        return Some(DocState::Accepted);
    }
    
    if lower.contains("implemented") || lower.contains("complete") {
        return Some(DocState::Final);
    }
    
    if lower.contains("rejected") || lower.contains("not approved") {
        return Some(DocState::Rejected);
    }
    
    if lower.contains("deferred") || lower.contains("postponed") {
        return Some(DocState::Deferred);
    }
    
    // Default to Draft if unclear
    None
}

/// Check if content looks like valid markdown
pub fn is_valid_markdown(content: &str) -> bool {
    // Basic checks
    if content.is_empty() {
        return false;
    }
    
    // Check for common markdown elements
    let has_heading = content.contains('#');
    let has_text = content.len() > 50;
    
    // Check it's not binary
    let is_text = content.chars().all(|c| c.is_ascii() || c.is_whitespace());
    
    has_text && is_text
}

/// Detect markdown flavor and issues
pub fn analyze_markdown(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    
    // Check for multiple H1s (should only have one)
    let h1_count = content.lines().filter(|line| line.trim().starts_with("# ")).count();
    if h1_count == 0 {
        issues.push("No H1 heading found".to_string());
    } else if h1_count > 1 {
        issues.push(format!("Multiple H1 headings found ({})", h1_count));
    }
    
    // Check for inconsistent list markers
    let bullet_types: Vec<char> = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+') {
                trimmed.chars().next()
            } else {
                None
            }
        })
        .collect();
    
    if bullet_types.len() > 0 {
        let first = bullet_types[0];
        if !bullet_types.iter().all(|&c| c == first) {
            issues.push("Inconsistent bullet point markers (-, *, +)".to_string());
        }
    }
    
    // Check for very long lines (> 120 chars)
    let long_lines = content.lines().filter(|line| line.len() > 120).count();
    if long_lines > 5 {
        issues.push(format!("Many long lines ({} lines > 120 chars)", long_lines));
    }
    
    issues
}
```

#### Step 2: Export Module
File: `design/src/lib.rs`

```rust
pub mod extract;
```

### Testing Steps for 7.3
1. Test title extraction from H1
2. Test author detection from content
3. Test state hint detection
4. Test markdown validation
5. Test markdown analysis

---

## Task 7.4: Content Normalization

### Purpose
Clean and standardize markdown content.

### Implementation Steps

#### Step 1: Create Content Normalizer
File: `design/src/normalize.rs`

```rust
//! Markdown content normalization

use regex::Regex;

/// Normalize markdown content
pub fn normalize_markdown(content: &str) -> String {
    let mut normalized = content.to_string();
    
    // Standardize bullet points to use '-'
    normalized = standardize_bullets(&normalized);
    
    // Ensure single blank line between sections
    normalized = normalize_spacing(&normalized);
    
    // Ensure headings have blank line before/after
    normalized = normalize_headings(&normalized);
    
    // Trim trailing whitespace from lines
    normalized = trim_line_whitespace(&normalized);
    
    // Ensure file ends with single newline
    normalized = normalized.trim_end().to_string() + "\n";
    
    normalized
}

fn standardize_bullets(content: &str) -> String {
    let re = Regex::new(r"^(\s*)[\*\+]\s").unwrap();
    content
        .lines()
        .map(|line| re.replace(line, "${1}- ").to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_spacing(content: &str) -> String {
    // Replace 3+ newlines with 2 newlines
    let re = Regex::new(r"\n{3,}").unwrap();
    re.replace_all(content, "\n\n").to_string()
}

fn normalize_headings(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let is_heading = line.trim().starts_with('#');
        
        if is_heading {
            // Add blank line before heading (unless first line or already blank)
            if i > 0 && !lines[i - 1].trim().is_empty() {
                result.push("");
            }
            
            result.push(line);
            
            // Add blank line after heading (unless last line or already blank)
            if i < lines.len() - 1 && !lines[i + 1].trim().is_empty() {
                result.push("");
            }
        } else {
            result.push(line);
        }
    }
    
    result.join("\n")
}

fn trim_line_whitespace(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Strip incomplete or malformed YAML frontmatter
pub fn strip_bad_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    
    if !trimmed.starts_with("---\n") {
        return content.to_string();
    }
    
    // Find closing ---
    if let Some(end_pos) = trimmed[4..].find("\n---\n") {
        let frontmatter = &trimmed[4..end_pos + 4];
        
        // Check if frontmatter looks valid (has : on most lines)
        let lines: Vec<&str> = frontmatter.lines().collect();
        let valid_lines = lines.iter().filter(|line| line.contains(':')).count();
        
        if valid_lines < lines.len() / 2 {
            // Looks malformed, strip it
            return trimmed[end_pos + 8..].to_string();
        }
    }
    
    content.to_string()
}
```

#### Step 2: Export Module
File: `design/src/lib.rs`

```rust
pub mod normalize;
```

### Testing Steps for 7.4
1. Test bullet standardization
2. Test spacing normalization
3. Test heading spacing
4. Test whitespace trimming
5. Test frontmatter stripping

---

## Task 7.5: Enhanced Add Command with Interactive Mode

### Purpose
Rebuild the add command with all new capabilities.

### Implementation Steps

#### Step 1: Update Add Command
File: `design/src/commands/add.rs`

Major rewrite to incorporate new features:

```rust
//! Enhanced add command with interactive onboarding

use anyhow::{Context, Result};
use colored::*;
use design::doc::*;
use design::extract::*;
use design::filename::*;
use design::normalize::*;
use design::prompt::*;
use design::state::StateManager;
use std::fs;
use std::path::PathBuf;

pub fn add_document(
    state_mgr: &mut StateManager,
    doc_path: &str,
    dry_run: bool,
    interactive: bool,
    auto_yes: bool,
) -> Result<()> {
    println!("{} {}\n", "Adding document:".bold(), doc_path);
    
    let mut path = PathBuf::from(doc_path);
    
    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    // Read content
    let mut content = fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    // Step 0: Validate it's markdown
    if !is_valid_markdown(&content) {
        anyhow::bail!("File doesn't appear to be valid markdown");
    }
    
    // Analyze content
    let issues = analyze_markdown(&content);
    if !issues.is_empty() && interactive {
        println!("{}", "Content Issues Detected:".yellow().bold());
        for issue in &issues {
            println!("  {} {}", "⚠".yellow(), issue);
        }
        println!();
        
        if !auto_yes {
            let should_normalize = prompt_confirm(
                "Apply automatic normalization?",
                true,
            )?;
            
            if should_normalize {
                content = normalize_markdown(&content);
                println!("  {} Content normalized\n", "✓".green());
            }
        } else {
            content = normalize_markdown(&content);
        }
    }
    
    // Extract metadata
    let extracted = ExtractedMetadata::from_content(&content);
    
    // Step 1: Determine title
    let title = if interactive {
        determine_title_interactive(&extracted, &path)?
    } else {
        determine_title_auto(&extracted, &path)
    };
    
    println!("{}", "Step 1: Title".cyan());
    println!("  Title: {}\n", title.bold());
    
    // Step 2: Number assignment and filename sanitization
    let number = state_mgr.state().next_number;
    let new_filename = build_filename(number, &title);
    
    println!("{}", "Step 2: Number & Filename".cyan());
    println!("  Number: {:04}", number);
    println!("  New filename: {}\n", new_filename.bold());
    
    if interactive && !auto_yes {
        let confirmed = prompt_confirm("Proceed with this filename?", true)?;
        if !confirmed {
            anyhow::bail!("User cancelled");
        }
    }
    
    // Step 3: Determine author
    let author = if interactive {
        determine_author_interactive(&extracted)?
    } else {
        determine_author_auto(&extracted)
    };
    
    println!("{}", "Step 3: Author".cyan());
    println!("  Author: {}\n", author.bold());
    
    // Step 4: Determine initial state
    let state = if interactive {
        determine_state_interactive(&extracted)?
    } else {
        extracted.state_hint.unwrap_or(DocState::Draft)
    };
    
    println!("{}", "Step 4: Initial State".cyan());
    println!("  State: {}\n", state.as_str().bold());
    
    // Step 5: Build complete metadata
    let today = chrono::Local::now().naive_local().date();
    let metadata = DocMetadata {
        number,
        title: title.clone(),
        author: author.clone(),
        created: today,
        updated: today,
        state,
        supersedes: None,
        superseded_by: None,
    };
    
    // Step 6: Process content
    println!("{}", "Step 5: Processing Content".cyan());
    
    // Strip bad frontmatter if present
    if extracted.has_frontmatter {
        content = strip_bad_frontmatter(&content);
        println!("  {} Removed existing frontmatter", "✓".green());
    }
    
    // Build new content with proper frontmatter
    let frontmatter = build_yaml_frontmatter(&metadata);
    let new_content = frontmatter + &content;
    
    println!("  {} Added complete YAML frontmatter\n", "✓".green());
    
    if dry_run {
        println!("{}", "DRY RUN - No changes made".yellow().bold());
        println!("\nWould create:");
        println!("  {}", PathBuf::from(state_mgr.docs_dir())
            .join(state.directory())
            .join(&new_filename)
            .display());
        return Ok(());
    }
    
    // Step 7: Move to correct location
    println!("{}", "Step 6: Moving to Repository".cyan());
    
    let state_dir = PathBuf::from(state_mgr.docs_dir()).join(state.directory());
    fs::create_dir_all(&state_dir)?;
    
    let final_path = state_dir.join(&new_filename);
    
    // Write the new file
    fs::write(&final_path, new_content)
        .context("Failed to write file")?;
    
    println!("  {} Created: {}\n", "✓".green(), final_path.display());
    
    // Step 8: Git add
    println!("{}", "Step 7: Git Staging".cyan());
    design::git::git_add(&final_path)?;
    println!("  {} Staged with git\n", "✓".green());
    
    // Step 9: Update state
    state_mgr.record_file_change(&final_path)?;
    
    // Step 10: Clean up original if different
    if path != final_path {
        if interactive && !auto_yes {
            let should_delete = prompt_confirm(
                &format!("Delete original file at {}?", path.display()),
                false,
            )?;
            
            if should_delete {
                fs::remove_file(&path)?;
                println!("  {} Deleted original file\n", "✓".green());
            }
        }
    }
    
    println!(
        "{} Successfully added: {}",
        "✓".green().bold(),
        new_filename.bold()
    );
    
    Ok(())
}

fn determine_title_interactive(
    extracted: &ExtractedMetadata,
    path: &PathBuf,
) -> Result<String> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    let default = extracted.title.as_ref()
        .or(extracted.first_heading.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| filename_to_title(filename));
    
    prompt_with_default("Document title", &default)
}

fn determine_title_auto(extracted: &ExtractedMetadata, path: &PathBuf) -> String {
    extracted.title.clone()
        .or(extracted.first_heading.clone())
        .unwrap_or_else(|| {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            filename_to_title(filename)
        })
}

fn determine_author_interactive(extracted: &ExtractedMetadata) -> Result<String> {
    let git_author = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let default = extracted.author.as_ref()
        .unwrap_or(&git_author);
    
    prompt_with_default("Author", default)
}

fn determine_author_auto(extracted: &ExtractedMetadata) -> String {
    extracted.author.clone()
        .unwrap_or_else(|| {
            design::git::get_author(std::path::Path::new("."))
        })
}

fn determine_state_interactive(extracted: &ExtractedMetadata) -> Result<DocState> {
    let states = DocState::all_state_names();
    let default_idx = if let Some(hint) = extracted.state_hint {
        DocState::all_states().iter().position(|&s| s == hint).unwrap_or(0)
    } else {
        0 // Draft
    };
    
    let selected = prompt_select("Initial state", &states, default_idx)?;
    Ok(DocState::from_str_flexible(&selected).unwrap())
}
```

#### Step 2: Update CLI
File: `design/src/cli.rs`

```rust
/// Add a new document with full processing
Add {
    /// Path to document file
    path: String,
    
    /// Show what would be done without making changes
    #[arg(long)]
    dry_run: bool,
    
    /// Interactive mode (prompt for metadata)
    #[arg(short, long)]
    interactive: bool,
    
    /// Auto-yes to prompts (non-interactive with defaults)
    #[arg(short = 'y', long)]
    yes: bool,
},
```

#### Step 3: Update Main
File: `design/src/main.rs`

```rust
Commands::Add { path, dry_run, interactive, yes } => {
    add_document(&mut state_mgr, &path, dry_run, interactive, yes)
}
```

### Testing Steps for 7.5
1. Test with messy filename
2. Test with missing metadata (interactive)
3. Test with auto mode (--yes)
4. Test with dry-run
5. Test with bad frontmatter
6. Test with non-markdown file
7. Test content normalization

---

## Task 7.6: Batch Operations

### Purpose
Support adding multiple files at once.

### Implementation Steps

#### Step 1: Add Batch Command
File: `design/src/commands/add.rs`

```rust
/// Add multiple documents
pub fn add_batch(
    state_mgr: &mut StateManager,
    patterns: Vec<String>,
    dry_run: bool,
    interactive: bool,
    auto_yes: bool,
) -> Result<()> {
    use glob::glob;
    
    let mut files = Vec::new();
    
    // Expand patterns
    for pattern in patterns {
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                if path.is_file() {
                    files.push(path);
                }
            }
        }
    }
    
    if files.is_empty() {
        println!("No files found matching patterns");
        return Ok(());
    }
    
    println!("{} Found {} files\n", "→".cyan(), files.len());
    
    if interactive && !auto_yes {
        for file in &files {
            println!("  - {}", file.display());
        }
        println!();
        
        let confirmed = prompt_confirm(
            &format!("Add all {} files?", files.len()),
            true,
        )?;
        
        if !confirmed {
            return Ok(());
        }
    }
    
    let mut succeeded = 0;
    let mut failed = 0;
    
    for (idx, file) in files.iter().enumerate() {
        println!(
            "{} [{}/{}] Processing: {}",
            "→".cyan(),
            idx + 1,
            files.len(),
            file.display()
        );
        
        match add_document(
            state_mgr,
            file.to_str().unwrap(),
            dry_run,
            false, // Non-interactive for batch
            true,  // Auto-yes
        ) {
            Ok(_) => {
                succeeded += 1;
                println!();
            }
            Err(e) => {
                eprintln!("{} Failed: {}\n", "✗".red(), e);
                failed += 1;
            }
        }
    }
    
    println!(
        "\n{} Batch complete: {} succeeded, {} failed",
        "✓".green().bold(),
        succeeded,
        failed
    );
    
    Ok(())
}
```

#### Step 2: Update CLI for Batch
File: `design/src/cli.rs`

```rust
/// Add multiple documents (supports glob patterns)
AddBatch {
    /// File patterns (e.g., *.md, docs/*.md)
    patterns: Vec<String>,
    
    /// Show what would be done without making changes
    #[arg(long)]
    dry_run: bool,
    
    /// Interactive mode (prompt for each file)
    #[arg(short, long)]
    interactive: bool,
},
```

#### Step 3: Add Dependencies
File: `design/Cargo.toml`

```toml
[dependencies]
glob = "0.3"
```

### Testing Steps for 7.6
1. Test with single glob pattern
2. Test with multiple patterns
3. Test with --dry-run
4. Test with mixed success/failure
5. Test progress reporting

---

## Task 7.7: Preview Mode Enhancement

### Purpose
Show before/after comparison when adding files.

### Implementation Steps

#### Step 1: Add Preview Function
File: `design/src/commands/add.rs`

```rust
use prettytable::{Table, Row, Cell, format};

/// Show preview of what will happen
pub fn preview_add(doc_path: &str, state_mgr: &StateManager) -> Result<()> {
    let path = PathBuf::from(doc_path);
    
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }
    
    let content = fs::read_to_string(&path)?;
    let extracted = ExtractedMetadata::from_content(&content);
    
    let title = determine_title_auto(&extracted, &path);
    let author = determine_author_auto(&extracted);
    let state = extracted.state_hint.unwrap_or(DocState::Draft);
    let number = state_mgr.state().next_number;
    let new_filename = build_filename(number, &title);
    
    let final_path = PathBuf::from(state_mgr.docs_dir())
        .join(state.directory())
        .join(&new_filename);
    
    println!("\n{}", "Preview of Changes".bold().underline());
    println!();
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    table.add_row(Row::new(vec![
        Cell::new("Property"),
        Cell::new("Before"),
        Cell::new("After"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("Location"),
        Cell::new(&path.display().to_string()),
        Cell::new(&final_path.display().to_string()).style_spec("Fg"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("Filename"),
        Cell::new(path.file_name().unwrap().to_str().unwrap()),
        Cell::new(&new_filename).style_spec("Fg"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("Number"),
        Cell::new("-"),
        Cell::new(&format!("{:04}", number)).style_spec("Fg"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("Title"),
        Cell::new(extracted.title.as_deref().unwrap_or("-")),
        Cell::new(&title).style_spec("Fg"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("Author"),
        Cell::new(extracted.author.as_deref().unwrap_or("-")),
        Cell::new(&author).style_spec("Fg"),
    ]));
    
    table.add_row(Row::new(vec![
        Cell::new("State"),
        Cell::new("-"),
        Cell::new(state.as_str()).style_spec("Fg"),
    ]));
    
    table.printstd();
    println!();
    
    Ok(())
}
```

#### Step 2: Add Dependencies
File: `design/Cargo.toml`

```toml
[dependencies]
prettytable-rs = "0.10"
```

#### Step 3: Add Preview Flag
File: `design/src/cli.rs`

```rust
Add {
    // ... existing fields ...
    
    /// Show preview without making changes
    #[arg(long)]
    preview: bool,
},
```

### Testing Steps for 7.7
1. Test preview with various files
2. Verify before/after table formatting
3. Test with different metadata scenarios

---

## Verification Checklist

Before considering Phase 7 complete:

- [ ] Filename sanitization works for all edge cases
- [ ] Interactive prompts work correctly
- [ ] Smart defaults are sensible
- [ ] Metadata extraction finds title/author
- [ ] Content normalization improves markdown
- [ ] Batch operations handle multiple files
- [ ] Preview mode shows accurate changes
- [ ] Dry-run mode is safe
- [ ] Auto-yes mode works for automation
- [ ] Error handling is robust
- [ ] Progress reporting is clear

---

## Usage Examples

```bash
# Basic add (interactive)
oxd add ~/Downloads/my-feature.md --interactive

# Auto mode with defaults
oxd add ~/Downloads/my-feature.md --yes

# Preview before adding
oxd add ~/Downloads/my-feature.md --preview

# Dry run
oxd add ~/Downloads/my-feature.md --dry-run

# Batch add
oxd add-batch ~/Downloads/*.md

# Batch with confirmation
oxd add-batch ~/Downloads/*.md --interactive
```

**Interactive Session Example:**
```
Adding document: ~/Downloads/My Cool Feature!!!.md

Content Issues Detected:
  ⚠ Multiple H1 headings found (2)
  ⚠ Inconsistent bullet point markers (-, *, +)

Apply automatic normalization? [Y/n]: y
  ✓ Content normalized

Step 1: Title
Document title [My Cool Feature]: 
  Title: My Cool Feature

Step 2: Number & Filename
  Number: 0012
  New filename: 0012-my-cool-feature.md

Proceed with this filename? [Y/n]: y

Step 3: Author
Author [Alice Smith]: 

  Author: Alice Smith

Step 4: Initial State
Initial state
 *1) Draft
  2) Under Review
  3) Revised
  ...
Select [1]: 1

  State: Draft

...

✓ Successfully added: 0012-my-cool-feature.md
```

---

## Notes for Claude Code

- **Interactive mode is key** - makes onboarding smooth
- **Smart defaults reduce friction** - extract what we can
- **Sanitization must be safe** - don't lose data
- **Batch mode for efficiency** - but still validate each file
- **Preview builds confidence** - show what will happen
- **Content normalization is optional** - some users want control
- **Test edge cases** - unicode, very long names, special chars
- **Maintain transaction safety** - don't half-process files
