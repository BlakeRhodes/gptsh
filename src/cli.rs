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

use std::{env, io};
use std::io::Write;
use std::process::Command;
use dotenv::dotenv;
use serde_json::Value;
use crate::chat::run_chat_mode;
use crate::openai::process_prompt;
use crate::shell::run_shell_mode;

pub(crate) fn run_mode() -> bool {
    let (continuous_mode, chat_mode, no_execute, prompt_args) = match parse_arguments() {
        Some(value) => value,
        None => return true,
    };

    // Execute appropriate mode
    if chat_mode {
        run_chat_mode();
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
}

pub(crate) fn print_help() {
    println!("Usage: gptsh [OPTIONS] [PROMPT]");
    println!("\nOptions:");
    println!("  --help, -h        Show this help message");
    println!("  --shell           Run in continuous shell mode");
    println!("  --chat            Run in chat mode with GPT-4");
    println!("  --no-execute      Output the generated command without executing it");
}

// Function to check if the command is a shell built-in
pub(crate) fn is_shell_builtin(command: &str) -> bool {
    let builtins = ["cd", "export", "alias", "source", "unset"];
    let command = command.trim();
    if command.is_empty() {
        return false;
    }
    let first_word = command.split_whitespace().next().unwrap_or("");
    builtins.contains(&first_word)
}


// Function to execute shell commands (used in function calling)
pub(crate) fn execute_shell_command(args: &serde_json::Map<String, Value>) -> Value {
    if let Some(Value::String(command)) = args.get("command") {
        // Prompt user for confirmation
        println!("The assistant wants to execute the following command: '{}'", command);
        print!("Do you allow this command to be executed? (Y/n) ");
        io::stdout().flush().unwrap();

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation).unwrap();
        let confirmation = confirmation.trim();

        if confirmation.eq_ignore_ascii_case("n") || confirmation.eq_ignore_ascii_case("no") {
            return serde_json::json!({
                "error": "User denied permission to execute the command."
            });
        }

        // Execute the command
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                serde_json::json!({
                    "stdout": stdout,
                    "stderr": stderr,
                    "status": output.status.code()
                })
            }
            Err(e) => {
                serde_json::json!({
                    "error": format!("Failed to execute command: {}", e)
                })
            }
        }
    } else {
        serde_json::json!({
            "error": "No command provided."
        })
    }
}

// Function to execute the command
pub(crate) fn should_execute_command(command: &str) -> Result<(), String> {
    if is_shell_builtin(command.trim()) {
        Err(format!(
            "Note: The command '{}' affects the shell's state and cannot be executed directly by this program.\nPlease run the following command in your terminal:\n{}",
            command.trim(),
            command.trim()
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn execute_command(command: &str) {
    match should_execute_command(command) {
        Ok(_) => {
            // Execute the command
            match Command::new("bash")
                .arg("-c")
                .arg(command)
                .status()
            {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Command exited with non-zero status.");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                }
            }
        }
        Err(message) => {
            println!("{}", message);
        }
    }
}


pub(crate) fn parse_arguments() -> Option<(bool, bool, bool, Vec<String>)> {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Handle help flag
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        return None;
    }

    // Check for flags
    let continuous_mode = args.contains(&String::from("--shell"));
    let chat_mode = args.contains(&String::from("--chat"));
    let no_execute = args.contains(&String::from("--no-execute"));

    // Filter out flags to get the prompt
    let prompt_args: Vec<String> = args.iter()
        .skip(1) // Skip the program name
        .filter(|arg| {
            arg.as_str() != "--no-execute"
                && arg.as_str() != "--shell"
                && arg.as_str() != "--chat"
                && arg.as_str() != "--help"
                && arg.as_str() != "-h"
        })
        .cloned()
        .collect();
    Some((continuous_mode, chat_mode, no_execute, prompt_args))
}
