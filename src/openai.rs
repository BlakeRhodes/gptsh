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
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use reqwest::blocking::{Client, Response};

use crate::{
    cli::execute_command,
    models::{Config, Message, OpenAIRequest, OpenAIResponse},
    utils::start_loading_animation,
};

/// Constants for configuration file paths.
const BANNED_COMMANDS_FILE: &str = ".gptsh_banned";
const ALLOWED_COMMANDS_FILE: &str = ".gptsh_allowed";
const CONFIG_FILE: &str = ".gptsh_config";
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL_NAME: &str = "gpt-4";

/// Handles non-success responses from the OpenAI API by logging the error and exiting the application.
///
/// # Arguments
///
/// * `response` - The HTTP response from the OpenAI API.
pub(crate) fn handle_non_success(response: Response) {
    eprintln!(
        "Error: Received non-success status code from OpenAI API: {}",
        response.status()
    );
    let error_text = response.text().unwrap_or_default();
    eprintln!("Response body: {}", error_text);
    std::process::exit(1);
}

/// Initializes the necessary configuration and command files if they do not exist.
/// This should be called during the application's initialization phase.
pub(crate) fn initialize_files() {
    initialize_file(BANNED_COMMANDS_FILE);
    initialize_file(ALLOWED_COMMANDS_FILE);
    initialize_file(CONFIG_FILE);
}

/// Creates a file at the specified path if it does not already exist.
///
/// # Arguments
///
/// * `file_path` - The path to the file to be created.
fn initialize_file(file_path: &str) {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        if let Err(e) = fs::File::create(&path) {
            eprintln!("Error creating {} file: {}", file_path, e);
            std::process::exit(1);
        }
    }
}

/// Loads the list of banned commands from the `.gptsh_banned` file.
/// Returns an empty vector if the file does not exist or is empty.
///
/// # Returns
///
/// * `io::Result<Vec<String>>` - A vector of banned commands or an I/O error.
fn load_banned_commands() -> io::Result<Vec<String>> {
    load_commands_from_file(BANNED_COMMANDS_FILE)
}

/// Adds a new command to the `.gptsh_banned` file, creating the file if it does not exist.
///
/// # Arguments
///
/// * `command` - The command to be banned.
///
/// # Returns
///
/// * `io::Result<()>` - An empty result or an I/O error.
fn add_banned_command(command: &str) -> io::Result<()> {
    append_command_to_file(BANNED_COMMANDS_FILE, command)
}

/// Loads the list of allowed commands from the `.gptsh_allowed` file.
/// Returns an empty vector if the file does not exist or is empty.
///
/// # Returns
///
/// * `io::Result<Vec<String>>` - A vector of allowed commands or an I/O error.
fn load_allowed_commands() -> io::Result<Vec<String>> {
    load_commands_from_file(ALLOWED_COMMANDS_FILE)
}

/// Loads commands from a specified file, returning an empty vector if the file does not exist.
///
/// # Arguments
///
/// * `file_path` - The path to the file containing the commands.
///
/// # Returns
///
/// * `io::Result<Vec<String>>` - A vector of commands or an I/O error.
fn load_commands_from_file(file_path: &str) -> io::Result<Vec<String>> {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let commands = reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>();
    Ok(commands)
}

/// Appends a command to a specified file, creating the file if it does not exist.
///
/// # Arguments
///
/// * `file_path` - The path to the file.
/// * `command` - The command to append.
///
/// # Returns
///
/// * `io::Result<()>` - An empty result or an I/O error.
fn append_command_to_file(file_path: &str, command: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    writeln!(file, "{}", command.trim())?;
    Ok(())
}

/// Loads the context from the `.gptsh_config` file.
/// Returns an empty string if the file does not exist or if the context is not set.
///
/// # Returns
///
/// * `io::Result<String>` - The context string or an I/O error.
fn load_context() -> io::Result<String> {
    let path = PathBuf::from(CONFIG_FILE);
    if !path.exists() {
        return Ok(String::new());
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader).unwrap_or_default();
    Ok(config.context.unwrap_or_default())
}

/// Extracts a bash command from a code block formatted string.
/// For example, it will extract `ls -la` from "```bash\nls -la\n```".
///
/// # Arguments
///
/// * `input` - The input string potentially containing a bash code block.
///
/// # Returns
///
/// * `Option<&str>` - The extracted command if a code block is present, else `None`.
fn extract_command(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    if trimmed.starts_with("```bash") && trimmed.ends_with("```") {
        trimmed
            .strip_prefix("```bash\n")
            .and_then(|s| s.strip_suffix("\n```"))
    } else {
        Some(trimmed)
    }
}

/// Processes the user prompt by interacting with the OpenAI API, managing command execution,
/// and handling banned and allowed commands.
///
/// # Arguments
///
/// * `prompt` - The user's input prompt.
/// * `no_execute` - If `true`, the command will not be executed but printed instead.
pub(crate) fn process_prompt(prompt: &str, no_execute: bool) {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY not set in environment.");
            std::process::exit(1);
        }
    };

    let client = Client::new();

    // Load the context from the configuration file
    let context = match load_context() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Error loading context: {}", err);
            String::new()
        }
    };

    // Prepare the conversation messages
    let mut messages = Vec::new();
    if !context.is_empty() {
        messages.push(Message {
            role: "system".to_string(),
            content: context.clone(),
        });
    }

    messages.push(Message {
        role: "user".to_string(),
        content: format!(
            "Translate the following prompt into a bash command without explanation:\n{}",
            prompt
        ),
    });

    let request_body = OpenAIRequest {
        model: MODEL_NAME.to_string(),
        messages
    };

    // Start loading animation
    let stop_signal = Arc::new(Mutex::new(false));
    let loading_handle = {
        let stop_signal_clone = Arc::clone(&stop_signal);
        thread::spawn(move || {
            start_loading_animation(stop_signal_clone);
        })
    };

    // Send the request to OpenAI API
    let response = client
        .post(OPENAI_API_URL)
        .bearer_auth(api_key)
        .json(&request_body)
        .send();

    // Stop loading animation
    {
        let mut stop = stop_signal.lock().unwrap();
        *stop = true;
    }
    loading_handle.join().unwrap();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let openai_response: OpenAIResponse = match resp.json() {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Failed to parse OpenAI response: {}", e);
                        std::process::exit(1);
                    }
                };

                if openai_response.choices.is_empty() {
                    eprintln!("OpenAI response contains no choices.");
                    std::process::exit(1);
                }

                let command_with_block = openai_response.choices[0]
                    .message
                    .content
                    .trim()
                    .to_string();

                // Extract the pure command without the code block
                let parsed_command = extract_command(&command_with_block).unwrap_or(&command_with_block).trim().to_string();

                // Load allowed and banned commands
                let allowed_commands = match load_allowed_commands() {
                    Ok(commands) => commands,
                    Err(err) => {
                        eprintln!("Error loading allowed commands: {}", err);
                        Vec::new()
                    }
                };

                let banned_commands = match load_banned_commands() {
                    Ok(commands) => commands,
                    Err(err) => {
                        eprintln!("Error loading banned commands: {}", err);
                        Vec::new()
                    }
                };

                // Check if the command is in the allowed list
                if allowed_commands.iter().any(|a| a == &parsed_command) {
                    if no_execute {
                        println!("{}", parsed_command);
                    } else {
                        println!("\nGenerated Command:\n```bash\n{}\n```", parsed_command);
                        execute_command(&parsed_command);
                    }
                    return;
                }

                // Check if the command is banned
                if banned_commands.iter().any(|b| b == &parsed_command) {
                    println!(
                        "Warning: The command \"{}\" is banned and will not be executed.",
                        parsed_command
                    );
                    return;
                }

                if no_execute {
                    println!("{}", parsed_command);
                } else {
                    println!("\nGenerated Command:\n```bash\n{}\n```", parsed_command);

                    // Prompt user for confirmation with 'y', 'n', 'b' options
                    print!("Do you want to execute this command? (Y/n/b for ban) ");
                    io::stdout().flush().unwrap();

                    let confirmation = read_user_confirmation();

                    match confirmation.as_str() {
                        "y" | "yes" | "" => {
                            // Execute the command
                            execute_command(&parsed_command);
                        }
                        "n" | "no" => {
                            println!("Command execution cancelled.");
                        }
                        "b" | "ban" => {
                            // Add the command to the banned list
                            if let Err(e) = add_banned_command(&parsed_command) {
                                eprintln!("Error banning the command: {}", e);
                            } else {
                                println!("Command \"{}\" has been banned.", parsed_command);
                            }
                        }
                        _ => {
                            println!("Invalid input. Command execution cancelled.");
                        }
                    }
                }
            } else {
                handle_non_success(resp);
            }
        }
        Err(e) => {
            eprintln!("Error communicating with OpenAI API: {}", e);
            std::process::exit(1);
        }
    }
}

/// Reads and interprets user confirmation input.
///
/// # Returns
///
/// * `String` - The user's input in lowercase.
fn read_user_confirmation() -> String {
    let mut input = String::new();
    if let Err(_) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input.");
        return String::new();
    }
    input.trim().to_lowercase()
}

