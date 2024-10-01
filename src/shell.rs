use std::fs::OpenOptions;
use crate::cli::execute_command;
use crate::openai::process_prompt;
use crate::utils::{get_current_dir_with_tilde, get_username};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{Editor};
use rustyline::history::FileHistory;

// Enum representing the different modes of the shell
#[derive(Debug, PartialEq)]
enum Mode {
    LlmSuggestion,
    DirectCommand,
}

// Struct to hold the shell state, including the current mode
struct ShellState {
    mode: Mode,
}

impl ShellState {
    // Create a new ShellState, defaulting to LLM suggestion mode
    fn new() -> Self {
        Self {
            mode: Mode::LlmSuggestion,
        }
    }
}

// Main function to run the shell in continuous mode
pub(crate) fn run_shell_mode(no_execute: bool) {
    let mut state = ShellState::new();
    println!("Entering continuous shell mode. Type 'exit' to quit.");

    // Initialize rustyline Editor for input handling with history
    let mut rl = Editor::<(), FileHistory>::new().expect("Failed to initialize editor");

    // Load history (this returns a Result)
    if rl.load_history(".gptsh_history").is_err() {
        let _ = OpenOptions::new()
            .write(true)
            .create(true)  // Create the file if it does not exist
            .append(true)  // Append to the file instead of overwriting
            .open(".gptsh_history");
    }

    loop {
        display_prompt(&state.mode);
        let prompt_text = display_prompt(&state.mode);
        let prompt = match rl.readline(prompt_text.as_str()) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                // Handle Ctrl-C
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Handle Ctrl-D
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        let trimmed_prompt = prompt.trim();

        if trimmed_prompt.eq_ignore_ascii_case("exit") {
            break;
        }

        if !trimmed_prompt.is_empty() {
            let _ = rl.add_history_entry(trimmed_prompt);
            if is_mode_switch_command(trimmed_prompt) {
                // Mode switch now also runs the command
                switch_mode(&mut state, trimmed_prompt, no_execute);
            } else {
                handle_input(trimmed_prompt, &state, no_execute);
            }
        }
    }

    // Save the history on exit
    rl.save_history(".gptsh_history").expect("Failed to save history");
}

// Function to check if a command is meant to switch modes
fn is_mode_switch_command(input: &str) -> bool {
    input.starts_with("u-")
}

// Function to switch between the different modes of the shell and execute the command
fn switch_mode(state: &mut ShellState, input: &str, no_execute: bool) {
    state.mode = match state.mode {
        Mode::LlmSuggestion => {
            println!("Switching to Direct Command Mode");
            Mode::DirectCommand
        }
        Mode::DirectCommand => {
            println!("Switching to LLM Suggestion Mode");
            Mode::LlmSuggestion
        }
    };

    // After switching modes, execute the command without the "u-" prefix
    let trimmed_input = trim_mode_prefix(input);
    handle_input(trimmed_input, state, no_execute);
}

// Updated handle_input function to delegate command handling
fn handle_input(input: &str, state: &ShellState, no_execute: bool) {
    match state.mode {
        Mode::LlmSuggestion => process_llm_suggestion(input, no_execute),
        Mode::DirectCommand => execute_direct_command(input),
    }
}

// Helper function to remove the mode switch prefix "u-" from the input
fn trim_mode_prefix(input: &str) -> &str {
    if is_mode_switch_command(input) {
        &input[2..] // Remove the "u-" prefix
    } else {
        input
    }
}

// Function to process a command in LLM suggestion mode
fn process_llm_suggestion(input: &str, no_execute: bool) {
    process_prompt(input, no_execute);
}

// Function to execute a command in direct mode
fn execute_direct_command(input: &str) {
    execute_command(input);
}

// Displays the shell prompt based on the current mode
fn display_prompt(mode: &Mode) -> String {
    let working_directory = get_current_dir_with_tilde();
    let username = get_username();

    let prompt_prefix = match mode {
        Mode::LlmSuggestion => "gptsh".red(),
        Mode::DirectCommand => "you".yellow(),
    };

    let prompt = format!(
        "[{}]:{}:{}$ ",
        prompt_prefix,
        username.green(),
        working_directory.blue()
    );

    return prompt;
}
