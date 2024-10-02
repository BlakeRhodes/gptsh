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
    models::{Message, OpenAIRequest, OpenAIResponse},
};
use crate::utils::start_loading_animation;

/// Path to the banned commands file.
const BANNED_COMMANDS_FILE: &str = ".gptsh_banned";

/// Handles non-success responses from the OpenAI API.
pub(crate) fn handle_non_success(response: Response) {
    eprintln!(
        "Error: Received non-success status code from OpenAI API: {}",
        response.status()
    );
    let error_text = response.text().unwrap_or_default();
    eprintln!("Response body: {}", error_text);
    std::process::exit(1);
}

/// Loads the list of banned commands from the `.gptsh_banned` file.
/// If the file does not exist, it returns an empty vector.
fn load_banned_commands() -> io::Result<Vec<String>> {
    let path = PathBuf::from(BANNED_COMMANDS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let commands = reader
        .lines()
        .filter_map(Result::ok)
        .collect::<Vec<String>>();
    Ok(commands)
}

/// Adds a new command to the `.gptsh_banned` file.
/// If the file does not exist, it creates one.
fn add_banned_command(command: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(BANNED_COMMANDS_FILE)?;
    writeln!(file, "{}", command)?;
    Ok(())
}

/// Processes the user prompt and communicates with the OpenAI API.
/// Integrates the Never Allow Commands feature to prevent execution of banned commands.
pub(crate) fn process_prompt(prompt: &str, no_execute: bool) {
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: OPENAI_API_KEY not set in environment.");
        std::process::exit(1);
    });

    let client = Client::new();

    let request_body = OpenAIRequest {
        model: "gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: format!(
                "Translate the following prompt into a bash command without explanation:\n{}",
                prompt
            ),
        }],
    };

    // Start loading animation
    let stop_signal = Arc::new(Mutex::new(false));
    let loading_handle = {
        let stop_signal_clone = Arc::clone(&stop_signal);
        thread::spawn(move || {
            start_loading_animation(stop_signal_clone);
        })
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
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
        Ok(response) => {
            if response.status().is_success() {
                let openai_response: OpenAIResponse = response.json().unwrap();
                let command_with_block = openai_response.choices[0].message.content.trim().to_string();

                // Extract the pure command without the code block
                let parsed_command = extract_command(&command_with_block).unwrap_or(&command_with_block);

                // Load banned commands
                let banned_commands = load_banned_commands().unwrap_or_else(|err| {
                    eprintln!("Error loading banned commands: {}", err);
                    Vec::new()
                });

                // Check if the pure command is banned
                if banned_commands.iter().any(|b| b == parsed_command) {
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

                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation).unwrap();
                    let confirmation = confirmation.trim();

                    match confirmation.to_lowercase().as_str() {
                        "y" | "yes" | "" => {
                            // Execute the pure command
                            execute_command(parsed_command);
                        }
                        "n" | "no" => {
                            println!("Command execution cancelled.");
                        }
                        "b" | "ban" => {
                            // Add the pure command to the banned list
                            if let Err(e) = add_banned_command(parsed_command) {
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
                handle_non_success(response);
            }
        }
        Err(e) => {
            eprintln!("Error communicating with OpenAI API: {}", e);
            std::process::exit(1);
        }
    }
}

/// Extracts the command from a code block if present.
fn extract_command(input: &str) -> Option<&str> {
    input
        .trim()
        .strip_prefix("```bash\n")
        .and_then(|s| s.strip_suffix("\n```"))
}

/// (Optional) You may want to initialize the banned commands file if it doesn't exist.
/// You can call this function during your application's initialization phase.
pub(crate) fn initialize_banned_commands_file() {
    let path = PathBuf::from(BANNED_COMMANDS_FILE);
    if !path.exists() {
        if let Err(e) = fs::File::create(&path) {
            eprintln!("Error creating banned commands file: {}", e);
            std::process::exit(1);
        }
    }
}
