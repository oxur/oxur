//! cargo-oxur - Cargo subcommand for building Oxur projects
//!
//! Provides `cargo oxur build`, `cargo oxur run`, `cargo oxur repl`, etc.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use oxur_cli::common::output;
use oxur_cli::config::ReplConfig;
use oxur_cli::ReplArgs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod cache;
mod scanner;

use cache::CompilationCache;
use scanner::OxurScanner;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
struct Cli {
    #[command(subcommand)]
    command: CargoCommands,
}

#[derive(Subcommand)]
enum CargoCommands {
    /// Oxur build system
    Oxur(OxurArgs),
}

#[derive(Parser)]
struct OxurArgs {
    #[command(subcommand)]
    command: OxurCommands,
}

#[derive(Subcommand)]
enum OxurCommands {
    /// Build the current Oxur project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Path to Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build and run the current Oxur project
    Run {
        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Path to Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,

        /// Arguments to pass to the binary
        args: Vec<String>,
    },

    /// Quick syntax check (parse + expand only)
    Check {
        /// Path to Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start the interactive REPL
    Repl(ReplArgs),
}

fn main() -> Result<()> {
    let Cli { command } = Cli::parse();

    match command {
        CargoCommands::Oxur(args) => match args.command {
            OxurCommands::Build { release, manifest_path, verbose } => {
                handle_build(release, manifest_path, verbose)
            }

            OxurCommands::Run { release, manifest_path, args } => {
                handle_run(release, manifest_path, args)
            }

            OxurCommands::Check { manifest_path, verbose } => handle_check(manifest_path, verbose),

            OxurCommands::Repl(args) => handle_repl(args),
        },
    }
}

fn handle_build(release: bool, manifest_path: Option<PathBuf>, verbose: bool) -> Result<()> {
    let project_root = find_project_root(manifest_path.as_deref())?;

    if verbose {
        println!("Building Oxur project at: {}", project_root.display());
        println!("Mode: {}", if release { "release" } else { "debug" });
    }

    // Step 1: Scan for Oxur files
    let scanner = OxurScanner::new(&project_root);
    let oxur_files = scanner.find_oxur_files()?;

    if oxur_files.is_empty() {
        println!("No Oxur files found. Nothing to compile.");
        return Ok(());
    }

    if verbose {
        println!("Found {} Oxur file(s):", oxur_files.len());
        for file in &oxur_files {
            println!("  - {}", file.display());
        }
    }

    // Step 2: Setup directories
    let target_dir = project_root.join("target");
    let oxur_gen_dir = target_dir.join("oxur-gen");
    let cache_dir = target_dir.join("oxur-cache");

    std::fs::create_dir_all(&oxur_gen_dir).context("Failed to create oxur-gen directory")?;
    std::fs::create_dir_all(&cache_dir).context("Failed to create oxur-cache directory")?;

    // Step 3: Load cache
    let cache_file = cache_dir.join("hashes.json");
    let mut cache = CompilationCache::load(&cache_file)?;

    // Step 4: Compile Oxur files that need it
    let mut compiled_count = 0;
    for oxur_file in &oxur_files {
        if cache.needs_recompile(oxur_file)? {
            if verbose {
                println!("Compiling: {}", oxur_file.display());
            }

            compile_oxur_file(oxur_file, &oxur_gen_dir, &project_root, verbose)?;
            cache.update(oxur_file)?;
            compiled_count += 1;
        } else if verbose {
            println!("Cached: {}", oxur_file.display());
        }
    }

    // Step 5: Save cache
    cache.save(&cache_file)?;

    if verbose && compiled_count > 0 {
        println!("Compiled {} Oxur file(s)", compiled_count);
    }

    // Step 6: Run cargo build
    if verbose {
        println!("Running cargo build...");
    }

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("build");
    cargo_cmd.current_dir(&project_root);

    if release {
        cargo_cmd.arg("--release");
    }

    let status = cargo_cmd.status().context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    if verbose {
        println!("Build complete!");
    }

    Ok(())
}

fn handle_run(release: bool, manifest_path: Option<PathBuf>, args: Vec<String>) -> Result<()> {
    // First build
    handle_build(release, manifest_path.clone(), false)?;

    // Then run
    let project_root = find_project_root(manifest_path.as_deref())?;

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("run");
    cargo_cmd.current_dir(&project_root);

    if release {
        cargo_cmd.arg("--release");
    }

    if !args.is_empty() {
        cargo_cmd.arg("--");
        cargo_cmd.args(&args);
    }

    let status = cargo_cmd.status().context("Failed to run cargo run")?;

    if !status.success() {
        anyhow::bail!("cargo run failed");
    }

    Ok(())
}

fn handle_check(manifest_path: Option<PathBuf>, verbose: bool) -> Result<()> {
    let project_root = find_project_root(manifest_path.as_deref())?;

    if verbose {
        println!("Checking Oxur project at: {}", project_root.display());
    }

    // Scan for Oxur files
    let scanner = OxurScanner::new(&project_root);
    let oxur_files = scanner.find_oxur_files()?;

    if oxur_files.is_empty() {
        println!("No Oxur files found.");
        return Ok(());
    }

    let mut errors = Vec::new();

    for oxur_file in &oxur_files {
        if verbose {
            println!("Checking: {}", oxur_file.display());
        }

        // Read and parse
        let source = std::fs::read_to_string(oxur_file)
            .with_context(|| format!("Failed to read {}", oxur_file.display()))?;

        let mut parser = oxur_lang::Parser::new(source);
        match parser.parse() {
            Ok(surface_forms) => {
                // Try expansion
                let mut expander = oxur_lang::Expander::new();
                if let Err(e) = expander.expand(surface_forms) {
                    errors.push(format!("{}: {}", oxur_file.display(), e));
                }
            }
            Err(e) => {
                errors.push(format!("{}: {}", oxur_file.display(), e));
            }
        }
    }

    if errors.is_empty() {
        println!("Check complete. All {} file(s) are valid.", oxur_files.len());
        Ok(())
    } else {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        anyhow::bail!("{} error(s) found", errors.len());
    }
}

fn compile_oxur_file(
    oxur_file: &Path,
    oxur_gen_dir: &Path,
    project_root: &Path,
    verbose: bool,
) -> Result<()> {
    // Read source
    let source = std::fs::read_to_string(oxur_file)
        .with_context(|| format!("Failed to read {}", oxur_file.display()))?;

    // Parse
    let mut parser = oxur_lang::Parser::new(source);
    let surface_forms =
        parser.parse().with_context(|| format!("Failed to parse {}", oxur_file.display()))?;

    // Expand
    let mut expander = oxur_lang::Expander::new();
    let core_forms = expander
        .expand(surface_forms)
        .with_context(|| format!("Failed to expand {}", oxur_file.display()))?;

    // Determine output path
    let relative_path = oxur_file.strip_prefix(project_root).unwrap_or(oxur_file);

    let output_path = oxur_gen_dir.join(relative_path).with_extension("rs");

    // Create parent directory
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Compile using oxur-comp
    let temp_dir = oxur_gen_dir.join(".build");
    std::fs::create_dir_all(&temp_dir)?;

    // We don't actually need a binary, just the Rust source
    // So we'll use the lowering and codegen directly
    // Create an empty SourceMap (no source position tracking for cargo-oxur)
    let source_map = oxur_comp::SourceMap::new();
    let mut lowerer = oxur_comp::lowering::Lowerer::new(source_map);
    let (file, _source_map) = lowerer
        .lower(core_forms)
        .with_context(|| format!("Failed to lower {}", oxur_file.display()))?;

    let codegen = oxur_comp::codegen::CodeGenerator::new();
    let rust_source = codegen
        .generate(&file)
        .with_context(|| format!("Failed to generate Rust for {}", oxur_file.display()))?;

    // Write Rust source to oxur-gen
    std::fs::write(&output_path, &rust_source)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    // Also copy to original location with .rs extension (so cargo can find it)
    let source_rs_path = oxur_file.with_extension("rs");
    std::fs::write(&source_rs_path, &rust_source)
        .with_context(|| format!("Failed to write {}", source_rs_path.display()))?;

    if verbose {
        println!("  → {}", output_path.display());
        println!("  → {}", source_rs_path.display());
    }

    Ok(())
}

fn find_project_root(manifest_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = manifest_path {
        if path.is_file() {
            Ok(path.parent().unwrap().to_path_buf())
        } else {
            Ok(path.to_path_buf())
        }
    } else {
        // Find Cargo.toml in current directory or parents
        let current = std::env::current_dir()?;
        let mut dir = current.as_path();

        loop {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return Ok(dir.to_path_buf());
            }

            match dir.parent() {
                Some(parent) => dir = parent,
                None => anyhow::bail!("Could not find Cargo.toml in current directory or parents"),
            }
        }
    }
}

/// Handle the REPL command with its various modes
fn handle_repl(args: ReplArgs) -> Result<()> {
    // Load configuration with layered resolution:
    // defaults -> file -> env -> CLI args
    let config = ReplConfig::load(args.no_color)?;

    if let Some(ref addr) = args.serve {
        // Server mode: start ReplServer and listen
        run_server_mode(addr, args.ack)
    } else if let Some(ref addr) = args.connect {
        // Connect mode: connect to existing server
        let addr = addr.clone().unwrap_or_else(|| "127.0.0.1:5099".to_string());
        run_connect_mode(&addr, config)
    } else {
        // Interactive mode (default): in-memory server + client
        run_interactive_mode(config)
    }
}

/// Run the default interactive REPL mode
///
/// Creates an in-process server and client connected via channels,
/// providing the fastest possible REPL experience.
fn run_interactive_mode(config: ReplConfig) -> Result<()> {
    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the interactive REPL
    rt.block_on(oxur_cli::repl::interactive::run(config))
}

/// Run the REPL in server-only mode
///
/// Starts a REPL server listening on the specified address.
/// Use a path for Unix socket, or HOST:PORT for TCP.
fn run_server_mode(addr: &str, ack_port: Option<u16>) -> Result<()> {
    output::info(&format!("Starting REPL server on {}...", addr));

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the server
    rt.block_on(oxur_cli::repl::server::run(addr, ack_port))
}

/// Run the REPL in connect mode
///
/// Connects to an existing REPL server and provides terminal interface.
fn run_connect_mode(addr: &str, config: ReplConfig) -> Result<()> {
    output::info(&format!("Connecting to REPL server at {}...", addr));

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Run the connect mode client
    rt.block_on(oxur_cli::repl::connect::run(addr, config))
}
