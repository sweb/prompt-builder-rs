use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use ignore;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse state file: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("{0}")]
    CustomError(String),
    #[error("Failed file walker: {0}")]
    GitIgnoreError(#[from] ignore::Error),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds a file to the state
    Add {
        /// The relative path to the file to add
        #[arg(required = true, num_args = 1..)]
        files: Vec<String>,
    },
    /// Lists the files currently in the state
    List {
        #[arg(short, long)]
        long: bool,
    },
    /// Clears the state
    Clear,
    /// Prints the file contents
    Print,
    /// Prints details about this application
    Info,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct FileEntry {
    relative_path: String,
    absolute_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct State {
    files: Vec<FileEntry>,
    #[serde(skip)]
    path: PathBuf,
}

impl State {
    fn new(path: PathBuf) -> Result<Self, AppError> {
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            if contents.trim().is_empty() {
                return Ok(State {
                    files: Vec::new(),
                    path,
                });
            }
            let mut state: State = serde_json::from_str(&contents)?;
            state.path = path;
            Ok(state)
        } else {
            Ok(State {
                files: Vec::new(),
                path,
            })
        }
    }

    fn save(&self) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let state_path = ProjectDirs::from("org", "sweb", "PromptBuilder")
        .map(|proj_dirs| proj_dirs.config_dir().join("state.json"))
        .unwrap_or_else(|| {
            eprintln!(
                "Warning: Could not determine config directory. Using current directory for state"
            );
            PathBuf::from("state.json")
        });
    let mut state = State::new(state_path)?;

    match cli.command {
        Commands::Add { files } => handle_add(&mut state, files)?,
        Commands::List { long } => handle_list(&state, long),
        Commands::Clear => handle_clear(&mut state)?,
        Commands::Print => handle_print(&state)?,
        Commands::Info => {
            println!("State path: {}", state.path.display());
        }
    }
    Ok(())
}

fn handle_add(state: &mut State, patterns: Vec<String>) -> Result<(), AppError> {
    let mut builder = WalkBuilder::new(&patterns[0]);

    let mut override_builder = ignore::overrides::OverrideBuilder::new(&patterns[0]);
    override_builder.add("!*.lock")?;
    for pattern in patterns.iter().skip(1) {
        builder.add(pattern);
    }
    let overrides = override_builder.build()?;
    builder.overrides(overrides);

    let mut added_count = 0;

    let existing_paths: std::collections::HashSet<_> = state
        .files
        .iter()
        .map(|f| f.absolute_path.clone())
        .collect();
    for result in builder.build() {
        let entry = result?;
        let file_path = entry.path();
        if file_path.is_file() {
            let absolute_path = fs::canonicalize(file_path)?;
            if !existing_paths.contains(&absolute_path) {
                let entry = FileEntry {
                    relative_path: file_path.to_string_lossy().into(),
                    absolute_path,
                };
                state.files.push(entry);
                added_count += 1;
            }
        }
    }
    if added_count > 0 {
        state.save()?;
        println!("{} file(s) added successfully.", added_count);
    } else {
        println!("No new files added.")
    }
    Ok(())
}

fn handle_list(state: &State, long: bool) {
    if state.files.is_empty() {
        println!("No files have been added yet.");
    } else {
        println!("Files in state:");
        for file in &state.files {
            if long {
                println!(
                    "- {} ({})",
                    file.relative_path,
                    file.absolute_path.to_string_lossy().into_owned()
                );
            } else {
                println!("- {}", file.relative_path)
            }
        }
    }
}

fn handle_clear(state: &mut State) -> Result<(), AppError> {
    state.files.clear();
    state.save()?;
    println!("State cleared.");
    Ok(())
}

fn handle_print(state: &State) -> Result<(), AppError> {
    if state.files.is_empty() {
        Err(AppError::CustomError("No files to print!".into()))
    } else {
        println!("<files>");
        for file_entry in &state.files {
            let contents = fs::read_to_string(&file_entry.absolute_path)?;
            println!("<file path=\"{}\">", file_entry.relative_path);
            println!("{}", contents);
            println!("</file>");
        }
        println!("</files>");
        Ok(())
    }
}
