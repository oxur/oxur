//! Index generation command implementation

use anyhow::Result;
use design::doc::DocState;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Generate the index markdown or JSON
pub fn generate_index(index: &DocumentIndex, format: &str) -> Result<()> {
    match format {
        "markdown" | "md" => generate_markdown_index(index),
        "json" => generate_json_index(index),
        _ => {
            eprintln!("Unknown format: {}. Using markdown.", format);
            generate_markdown_index(index)
        }
    }
}

fn generate_markdown_index(index: &DocumentIndex) -> Result<()> {
    let mut content = String::new();

    // Header
    content.push_str("# Design Document Index\n\n");
    content.push_str("This index is automatically generated. Do not edit manually.\n\n");

    // Table section
    content.push_str("## All Documents by Number\n\n");
    content.push_str("| Number | Title | State | Updated |\n");
    content.push_str("|--------|-------|-------|----------|\n");

    let docs = index.all();

    for doc in &docs {
        content.push_str(&format!(
            "| {:04} | {} | {} | {} |\n",
            doc.metadata.number,
            doc.metadata.title,
            doc.metadata.state.as_str(),
            doc.metadata.updated
        ));
    }

    content.push('\n');

    // State sections
    content.push_str("## Documents by State\n");

    for state in DocState::all_states() {
        let state_docs = index.by_state(state);

        if !state_docs.is_empty() {
            content.push_str(&format!("\n### {}\n\n", state.as_str()));

            for doc in state_docs {
                let rel_path = doc.path.strip_prefix(index.docs_dir()).unwrap_or(&doc.path);
                let path_str = rel_path.to_string_lossy();

                content.push_str(&format!(
                    "- [{:04} - {}]({})\n",
                    doc.metadata.number, doc.metadata.title, path_str
                ));
            }
        }
    }

    // Write to file
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
    fs::write(&index_path, content)?;

    println!("Generated index at: {}", index_path.display());
    Ok(())
}

fn generate_json_index(index: &DocumentIndex) -> Result<()> {
    #[derive(serde::Serialize)]
    struct JsonDoc {
        number: u32,
        title: String,
        author: String,
        state: String,
        created: String,
        updated: String,
        path: String,
    }

    let docs: Vec<JsonDoc> = index
        .all()
        .iter()
        .map(|doc| {
            let rel_path = doc.path.strip_prefix(index.docs_dir()).unwrap_or(&doc.path);

            JsonDoc {
                number: doc.metadata.number,
                title: doc.metadata.title.clone(),
                author: doc.metadata.author.clone(),
                state: doc.metadata.state.as_str().to_string(),
                created: doc.metadata.created.to_string(),
                updated: doc.metadata.updated.to_string(),
                path: rel_path.to_string_lossy().to_string(),
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&docs)?;

    let index_path = PathBuf::from(index.docs_dir()).join("00-index.json");
    fs::write(&index_path, json)?;

    println!("Generated JSON index at: {}", index_path.display());
    Ok(())
}
