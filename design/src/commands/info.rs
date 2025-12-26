use anyhow::Result;
use colored::Colorize;
use design::state::StateManager;

/// Info subcommands
#[derive(Debug, Clone)]
pub enum InfoCommand {
    Overview,
    States,
    Fields,
    Config,
    Stats,
    Dirs,
}

impl InfoCommand {
    pub fn from_str(s: Option<&str>) -> Self {
        match s {
            Some("states") => InfoCommand::States,
            Some("fields") | Some("metadata") => InfoCommand::Fields,
            Some("config") => InfoCommand::Config,
            Some("stats") => InfoCommand::Stats,
            Some("dirs") | Some("structure") => InfoCommand::Dirs,
            _ => InfoCommand::Overview,
        }
    }
}

pub fn execute(subcommand: Option<String>, state_mgr: &StateManager) -> Result<()> {
    let cmd = InfoCommand::from_str(subcommand.as_deref());

    match cmd {
        InfoCommand::Overview => show_overview(state_mgr)?,
        InfoCommand::States => show_states()?,
        InfoCommand::Fields => show_fields()?,
        InfoCommand::Config => show_config(state_mgr)?,
        InfoCommand::Stats => show_stats(state_mgr)?,
        InfoCommand::Dirs => show_dirs(state_mgr)?,
    }

    Ok(())
}

fn show_overview(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.3
    println!("{}", "Info overview not yet implemented".yellow());
    Ok(())
}

fn show_states() -> Result<()> {
    // Will implement in Task 10.4
    println!("{}", "States info not yet implemented".yellow());
    Ok(())
}

fn show_fields() -> Result<()> {
    // Will implement in Task 10.5
    println!("{}", "Fields info not yet implemented".yellow());
    Ok(())
}

fn show_config(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.6
    println!("{}", "Config info not yet implemented".yellow());
    Ok(())
}

fn show_stats(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.7
    println!("{}", "Stats info not yet implemented".yellow());
    Ok(())
}

fn show_dirs(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.8
    println!("{}", "Dirs info not yet implemented".yellow());
    Ok(())
}
