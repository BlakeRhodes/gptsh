/*
 * Copyright 2024 Blake Rhodes
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{
    env,
    process::{Command, ExitStatus},
};

use dotenv::dotenv;

use crate::{
    chat::run_chat_mode,
    openai::process_prompt,
    shell::run_shell_mode,
};

/// Determines and runs the appropriate mode based on command-line arguments.
/// Returns `true` if the program should exit immediately.
pub(crate) fn run_mode() -> bool {
    if let Some((continuous_mode, chat_mode, no_execute, prompt_args)) = parse_arguments() {
        // Execute the appropriate mode
        if chat_mode {
            run_chat_mode(false);
        } else if continuous_mode {
            run_shell_mode(no_execute);
        } else if !prompt_args.is_empty() {
            let prompt = prompt_args.join(" ");
            process_prompt(&prompt, no_execute);
        } else {
            eprintln!("Error: No prompt provided.\n");
            print_help();
            std::process::exit(1);
        }
        false
    } else {
        // Help was printed or an error occurred; exit the program.
        true
    }
}

/// Prints the help message for the command-line tool.
pub(crate) fn print_help() {
    println!(
        "Usage: gptsh [OPTIONS] [PROMPT]\n\
         Options:\n\
           --help, -h        Show this help message\n\
           --shell           Run in continuous shell mode\n\
           --chat            Run in chat mode with GPT-4\n\
           --no-execute      Output the generated command without executing it"
    );
}

/// Checks if a given command is a shell built-in that affects the shell's state.
pub(crate) fn is_shell_builtin(command: &str) -> bool {
    const SHELL_BUILTINS: &[&str] = &["cd", "export", "alias", "source", "unset"];
    if let Some(first_word) = command.trim().split_whitespace().next() {
        SHELL_BUILTINS.contains(&first_word)
    } else {
        false
    }
}

/// Determines if the command should be executed.
/// Returns `Ok(())` if it can be executed, or an error message explaining why it cannot.
pub(crate) fn should_execute_command(command: &str) -> Result<(), String> {
    if is_shell_builtin(command) {
        Err(format!(
            "Note: The command '{}' affects the shell's state and cannot be executed directly by this program.\nPlease run the following command in your terminal:\n{}",
            command.trim(),
            command.trim()
        ))
    } else {
        Ok(())
    }
}

/// Executes a given command using Bash if it is safe to do so.
/// Prints an error message if the command cannot be executed.
pub(crate) fn execute_command(command: &str) {
    if let Err(message) = should_execute_command(command) {
        println!("{}", message);
        return;
    }

    match Command::new("bash").arg("-c").arg(command).status() {
        Ok(status) => handle_command_status(status),
        Err(e) => eprintln!("Failed to execute command: {}", e),
    }
}

/// Handles the exit status of a command execution.
fn handle_command_status(status: ExitStatus) {
    if !status.success() {
        eprintln!("Command exited with non-zero status.");
    }
}

/// Parses command-line arguments and returns a tuple containing:
/// (continuous_mode, chat_mode, no_execute, prompt_args).
/// Returns `None` if the program should exit (e.g., after printing help).
pub(crate) fn parse_arguments() -> Option<(bool, bool, bool, Vec<String>)> {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Handle help flag
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return None;
    }

    // Check for flags
    let continuous_mode = args.contains(&"--shell".to_string());
    let chat_mode = args.contains(&"--chat".to_string());
    let no_execute = args.contains(&"--no-execute".to_string());

    // Define recognized flags
    const FLAGS: &[&str] = &["--no-execute", "--shell", "--chat", "--help", "-h"];

    // Filter out flags to get the prompt arguments
    let prompt_args: Vec<String> = args
        .iter()
        .skip(1) // Skip the program name
        .filter(|arg| !FLAGS.contains(&arg.as_str()))
        .cloned()
        .collect();

    Some((continuous_mode, chat_mode, no_execute, prompt_args))
}
