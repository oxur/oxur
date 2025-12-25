//! New document command implementation

use anyhow::Result;
use chrono::Local;
use design::doc::DocState;
use design::git;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

pub fn new_document(index: &DocumentIndex, title: String, author: Option<String>) -> Result<()> {
    let number = index.next_number();
    let author = author.unwrap_or_else(|| git::get_author("."));

    let today = Local::now().naive_local().date();

    let template = format!(
        r#"---
number: {}
title: "{}"
author: "{}"
created: {}
updated: {}
state: Draft
supersedes: null
superseded-by: null
---

# {}

## Overview

*Brief description of what this document covers*

## Background

*Context and motivation for this design*

## Proposal

*Detailed description of the proposed design*

## Alternatives Considered

*What other approaches were considered and why were they rejected?*

## Implementation Plan

*Steps needed to implement this design*

## Open Questions

*Unresolved questions that need discussion*

## Success Criteria

*How will we know this design is successful?*
"#,
        number, title, author, today, today, title
    );

    let filename = format!(
        "{:04}-{}.md",
        number,
        title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    );

    // Use the new directory naming scheme
    let docs_dir = PathBuf::from(index.docs_dir()).join(DocState::Draft.directory());
    fs::create_dir_all(&docs_dir)?;

    let path = docs_dir.join(&filename);
    fs::write(&path, template)?;

    println!("Created new design document:");
    println!("  Number: {:04}", number);
    println!("  Title: {}", title);
    println!("  File: {}", path.display());

    Ok(())
}
