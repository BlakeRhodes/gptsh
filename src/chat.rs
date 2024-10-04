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

use crate::openai::handle_non_success;
use crate::utils::start_loading_animation;
use reqwest::blocking::Client;
use serde_json::Value;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Constants for configuration
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL_NAME: &str = "gpt-4";
const SYSTEM_PROMPT: &str =
    "You are a helpful assistant chatting in a terminal, use proper formatting so that your answers are easy to read. Address the user as pal or buddy.";

/// Entry point for running the chat mode.
///
/// # Arguments
///
/// * `verbose` - A boolean flag to enable verbose output.
pub(crate) fn run_chat_mode(verbose: bool) {
    announce_entry_to_chat_mode();

    let api_key = match fetch_api_key() {
        Ok(key) => key,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let client = Client::new();
    let mut messages = initialize_messages_with_system_prompt();

    loop {
        let user_input = read_user_input().trim().to_string();
        if should_exit(&user_input) {
            println!("See you later pal.");
            break;
        }

        if user_input.is_empty() {
            continue;
        }

        add_user_message(&mut messages, &user_input);
        let request_body = prepare_request_body(&messages);

        let stop_signal = start_loading_indicator();
        let response = send_request(&client, &api_key, &request_body);
        stop_loading_indicator(stop_signal);

        match handle_response(response, &mut messages, &client, &api_key, verbose) {
            Some(true) => {
                println!("See you later pal.");
                break;
            }
            Some(false) => continue,
            None => {}
        }
    }
}

/// Announces entry into chat mode.
fn announce_entry_to_chat_mode() {
    println!("Entering chat mode. Type 'exit' or 'quit' to end the session.");
}

/// Fetches the OpenAI API key from environment variables.
///
/// # Returns
///
/// * `Result<String, String>` - The API key or an error message.
fn fetch_api_key() -> Result<String, String> {
    env::var("OPENAI_API_KEY")
        .map_err(|_| "Error: OPENAI_API_KEY not set in environment.".to_string())
}

/// Initializes the conversation with the system prompt.
///
/// # Returns
///
/// * `Vec<Value>` - A vector of JSON values representing the initial messages.
fn initialize_messages_with_system_prompt() -> Vec<Value> {
    vec![serde_json::json!({
        "role": "system",
        "content": SYSTEM_PROMPT
    })]
}

/// Reads user input from the terminal.
///
/// # Returns
///
/// * `String` - The user's input.
fn read_user_input() -> String {
    print!("You: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => (), // Successfully read input; do nothing
        Err(_) => eprintln!("Failed to read input."),
    }
    input
}

/// Determines if the user wants to exit the chat.
///
/// # Arguments
///
/// * `input` - The user's input.
///
/// # Returns
///
/// * `bool` - `true` if the user wants to exit, else `false`.
fn should_exit(input: &str) -> bool {
    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
        println!("Exiting chat mode.");
        true
    } else {
        false
    }
}

/// Adds the user's message to the conversation history.
///
/// # Arguments
///
/// * `messages` - Mutable reference to the messages vector.
/// * `user_input` - The user's input.
fn add_user_message(messages: &mut Vec<Value>, user_input: &str) {
    messages.push(serde_json::json!({
        "role": "user",
        "content": user_input
    }));
}

/// Prepares the JSON request body for the OpenAI API.
///
/// # Arguments
///
/// * `messages` - Reference to the messages vector.
///
/// # Returns
///
/// * `Value` - The JSON request body.
fn prepare_request_body(messages: &[Value]) -> Value {
    serde_json::json!({
        "model": MODEL_NAME,
        "messages": messages,
        "functions": get_function_definitions(),
        "function_call": "auto"
    })
}

/// Defines the available functions that the assistant can call.
///
/// # Returns
///
/// * `Vec<Value>` - A vector of JSON values representing function definitions.
fn get_function_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "execute_command",
            "description": "Executes a shell command and returns the output.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute."
                    }
                },
                "required": ["command"]
            }
        }),
        serde_json::json!({
            "name": "exit_chat",
            "description": "Signals that the user wants to exit the chat.",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        }),
    ]
}

/// Starts the loading indicator in a separate thread.
///
/// # Returns
///
/// * `Arc<Mutex<bool>>` - A shared signal to stop the loading indicator.
fn start_loading_indicator() -> Arc<Mutex<bool>> {
    let stop_signal = Arc::new(Mutex::new(false));
    let signal_clone = Arc::clone(&stop_signal);

    thread::spawn(move || {
        start_loading_animation(signal_clone);
    });

    stop_signal
}

/// Sends the prepared request to the OpenAI API.
///
/// # Arguments
///
/// * `client` - Reference to the HTTP client.
/// * `api_key` - The OpenAI API key.
/// * `request_body` - Reference to the JSON request body.
///
/// # Returns
///
/// * `reqwest::Result<reqwest::blocking::Response>` - The HTTP response.
fn send_request(
    client: &Client,
    api_key: &str,
    request_body: &Value,
) -> reqwest::Result<reqwest::blocking::Response> {
    client
        .post(OPENAI_API_URL)
        .bearer_auth(api_key)
        .json(request_body)
        .send()
}

/// Stops the loading indicator.
///
/// # Arguments
///
/// * `stop_signal` - The shared signal to stop the loading indicator.
fn stop_loading_indicator(stop_signal: Arc<Mutex<bool>>) {
    if let Ok(mut signal) = stop_signal.lock() {
        *signal = true;
    }
    thread::sleep(Duration::from_millis(100));
    print!("\x1b[2K\r"); // Clear the current line
    io::stdout().flush().unwrap();
}

/// Handles the API response.
///
/// # Arguments
///
/// * `response` - The API response.
/// * `messages` - Mutable reference to the messages vector.
/// * `client` - Reference to the HTTP client.
/// * `api_key` - The OpenAI API key.
/// * `verbose` - Verbose flag.
///
/// # Returns
///
/// * `Option<bool>` - Signals whether to exit the chat.
fn handle_response(
    response: reqwest::Result<reqwest::blocking::Response>,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
    verbose: bool,
) -> Option<bool> {
    match response {
        Ok(resp) if resp.status().is_success() => {
            let openai_response: Value = match resp.json() {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to parse JSON response: {}", e);
                    return None;
                }
            };
            process_openai_response(openai_response, messages, client, api_key, verbose)
        }
        Ok(resp) => {
            handle_non_success(resp);
            None
        }
        Err(e) => {
            eprintln!("Error communicating with OpenAI API: {}", e);
            None
        }
    }
}

/// Processes the successful OpenAI API response.
///
/// # Arguments
///
/// * `response` - The parsed JSON response.
/// * `messages` - Mutable reference to the messages vector.
/// * `client` - Reference to the HTTP client.
/// * `api_key` - The OpenAI API key.
/// * `verbose` - Verbose flag.
///
/// # Returns
///
/// * `Option<bool>` - Signals whether to exit the chat.
fn process_openai_response(
    response: Value,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
    verbose: bool,
) -> Option<bool> {
    let choices = match response["choices"].as_array() {
        Some(arr) => arr,
        None => {
            eprintln!("Unexpected response format: 'choices' field is missing.");
            return None;
        }
    };

    if choices.is_empty() {
        eprintln!("No choices found in the response.");
        return None;
    }

    let choice = &choices[0];
    let message = &choice["message"];

    let mut assistant_message = serde_json::json!({ "role": "assistant" });

    if let Some(content) = message["content"].as_str() {
        assistant_message["content"] = Value::String(content.to_string());
    }

    if let Some(function_call) = message.get("function_call") {
        assistant_message["function_call"] = function_call.clone();
    }

    messages.push(assistant_message);

    if let Some(function_call) = message.get("function_call") {
        handle_function_call(function_call, messages, client, api_key, verbose)
    } else {
        if let Some(content) = message["content"].as_str() {
            println!("\ngptsh: {}\n", content.trim());
        }
        None
    }
}

/// Handles function calls requested by the assistant.
///
/// # Arguments
///
/// * `function_call` - The function call object.
/// * `messages` - Mutable reference to the messages vector.
/// * `client` - Reference to the HTTP client.
/// * `api_key` - The OpenAI API key.
/// * `verbose` - Verbose flag.
///
/// # Returns
///
/// * `Option<bool>` - Signals whether to exit the chat.
fn handle_function_call(
    function_call: &Value,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
    verbose: bool,
) -> Option<bool> {
    let function_name = match function_call["name"].as_str() {
        Some(name) => name,
        None => {
            eprintln!("Function call missing 'name' field.");
            return None;
        }
    };

    match function_name {
        "execute_command" => {
            execute_command(function_call, messages, verbose);
            // Prepare and send a new request after executing the command
            let request_body = prepare_request_body(&messages);
            let stop_signal = start_loading_indicator();
            let response = send_request(client, api_key, &request_body);
            stop_loading_indicator(stop_signal);
            handle_response(response, messages, client, api_key, verbose)
        }
        "exit_chat" => Some(true),
        _ => {
            eprintln!("Error: Assistant requested an unknown function '{}'.", function_name);
            None
        }
    }
}

/// Executes a shell command as per the function call.
///
/// # Arguments
///
/// * `function_call` - The function call object.
/// * `messages` - Mutable reference to the messages vector.
/// * `verbose` - Verbose flag.
fn execute_command(function_call: &Value, messages: &mut Vec<Value>, verbose: bool) {
    let arguments_str = function_call["arguments"].as_str().unwrap_or_default();
    let arguments: Value = match serde_json::from_str(arguments_str) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Failed to parse function arguments: {}", e);
            return;
        }
    };

    let command = arguments["command"].as_str().unwrap_or_default();

    if command.is_empty() {
        eprintln!("No command provided to execute.");
        return;
    }

    println!("About to execute command: '{}'", command);
    println!("Do you want to proceed? [Y/n]");

    let confirmation = read_confirmation();

    if confirmation {
        let adjusted_command = adjust_command(command);
        match execute_shell_command(&adjusted_command) {
            Ok(output) => {
                if verbose {
                    if !output.stdout.is_empty() {
                        println!("Command output:\n{:?}", output.stdout);
                    }
                    if !output.stderr.is_empty() {
                        eprintln!("Command error:\n{:?}", output.stderr);
                    }
                }

                // Add the command's output to messages for further processing or display
                messages.push(serde_json::json!({
                    "role": "function",
                    "name": "execute_command",
                    "content": output.stdout
                }));
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
            }
        }

        // Ensure all output is written to the terminal
        io::stdout().flush().expect("Failed to flush stdout");
    } else {
        println!("Command execution cancelled.");
    }
}

/// Reads and interprets user confirmation.
///
/// # Returns
///
/// * `bool` - `true` if the user confirmed, else `false`.
fn read_confirmation() -> bool {
    let mut input = String::new();
    if let Err(_) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input.");
        return false;
    }

    input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y")
}

/// Adjusts specific commands for compatibility or desired behavior.
///
/// # Arguments
///
/// * `command` - The original command.
///
/// # Returns
///
/// * `&str` - The adjusted command.
fn adjust_command(command: &str) -> &str {
    if command.trim() == "ls" {
        "ls -C"
    } else {
        command
    }
}

/// Executes a shell command.
///
/// # Arguments
///
/// * `command` - The command to execute.
///
/// # Returns
///
/// * `Result<std::process::Output, std::io::Error>` - The command's output or an error.
fn execute_shell_command(command: &str) -> Result<std::process::Output, std::io::Error> {
    Command::new("sh").arg("-c").arg(command).output()
}
