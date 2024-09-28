// src/main.rs

mod tests;

use std::env;
use std::io::{self, Write};
use std::process::Command;

use dotenv::dotenv;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn main() {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Handle help flag
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        return;
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
}

// Function to print help message
fn print_help() {
    println!("Usage: gptsh [OPTIONS] [PROMPT]");
    println!("\nOptions:");
    println!("  --help, -h        Show this help message");
    println!("  --shell           Run in continuous shell mode");
    println!("  --chat            Run in chat mode with GPT-4");
    println!("  --no-execute      Output the generated command without executing it");
}

// Function to run continuous shell mode
fn run_shell_mode(no_execute: bool) {
    println!("Entering continuous shell mode. Type 'exit' to quit.");
    loop {
        print!("gptsh> ");
        io::stdout().flush().unwrap();

        let mut prompt = String::new();
        io::stdin().read_line(&mut prompt).unwrap();
        let prompt = prompt.trim();

        if prompt.eq_ignore_ascii_case("exit") {
            break;
        }

        if !prompt.is_empty() {
            process_prompt(prompt, no_execute);
        }
    }
}

// Function to run chat mode
fn run_chat_mode() {
    println!("Entering chat mode with GPT-4. Type 'exit' or 'quit' to end the session.");

    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY not set in environment.");
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let mut messages = Vec::new();

    // Initial system prompt
    messages.push(serde_json::json!({
        "role": "system",
        "content": "You are a helpful assistant. When appropriate, you can execute shell commands to assist the user. If you detect that the user wants to exit the conversation, you should call the 'exit_chat' function."
    }));

    // Define the function specifications
    let function_definitions = vec![
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
        })
    ];

    loop {
        print!("You: ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("Exiting chat mode.");
            break;
        }

        if user_input.is_empty() {
            continue;
        }

        // Add user's message to the conversation
        messages.push(serde_json::json!({
            "role": "user",
            "content": user_input
        }));

        // Prepare the request body
        let request_body = serde_json::json!({
            "model": "gpt-4",
            "messages": messages,
            "functions": function_definitions,
            "function_call": "auto"
        });

        // Send the request to OpenAI API
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&request_body)
            .send();

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let openai_response: Value = response.json().unwrap();
                    let choices = openai_response["choices"].as_array().unwrap();
                    let choice = &choices[0];
                    let message = &choice["message"];

                    if message.get("function_call").is_some() {
                        // Assistant is requesting a function call
                        let function_call = &message["function_call"];
                        let function_name = function_call["name"].as_str().unwrap();

                        if function_name == "execute_command" {
                            let arguments = function_call["arguments"].as_str().unwrap();
                            let args: serde_json::Map<String, Value> = serde_json::from_str(arguments).unwrap();
                            let result = execute_shell_command(&args);

                            // Add the assistant's function call to messages
                            let mut assistant_message = serde_json::Map::new();
                            assistant_message.insert("role".to_string(), Value::String("assistant".to_string()));
                            assistant_message.insert("content".to_string(), Value::Null);
                            assistant_message.insert("function_call".to_string(), function_call.clone());

                            messages.push(Value::Object(assistant_message));

                            // Add the function's result to messages
                            messages.push(serde_json::json!({
                                "role": "function",
                                "name": "execute_command",
                                "content": serde_json::to_string(&result).unwrap()
                            }));

                            // Now, get the assistant's response using the function result
                            let follow_up_request_body = serde_json::json!({
                                "model": "gpt-4",
                                "messages": messages
                            });

                            let follow_up_response = client
                                .post("https://api.openai.com/v1/chat/completions")
                                .bearer_auth(&api_key)
                                .json(&follow_up_request_body)
                                .send();

                            if let Ok(follow_up_response) = follow_up_response {
                                if follow_up_response.status().is_success() {
                                    let follow_up_openai_response: Value = follow_up_response.json().unwrap();
                                    let follow_up_choices = follow_up_openai_response["choices"].as_array().unwrap();
                                    let follow_up_choice = &follow_up_choices[0];
                                    let assistant_reply = follow_up_choice["message"]["content"].as_str().unwrap_or("").trim();

                                    println!("\nGPT-4: {}\n", assistant_reply);
                                } else {
                                    handle_non_success(follow_up_response);
                                }
                            } else {
                                eprintln!("Error communicating with OpenAI API during follow-up request.");
                                std::process::exit(1);
                            }
                        } else if function_name == "exit_chat" {
                            println!("Assistant has detected that you want to exit the chat. Exiting.");
                            break;
                        } else {
                            eprintln!("Error: Assistant requested an unknown function '{}'.", function_name);
                            std::process::exit(1);
                        }
                    } else {
                        // Regular assistant response
                        let assistant_reply = message["content"].as_str().unwrap_or("").trim();
                        println!("\nGPT-4: {}\n", assistant_reply);
                    }
                } else {
                    handle_non_success(response);
                }
            }
            Err(e) => {
                eprintln!("Error communicating with OpenAI API: {}", e);
                eprintln!("Tip: Check your internet connection and try again.");
                std::process::exit(1);
            }
        }
    }
}


fn handle_non_success(follow_up_response: Response) {
    eprintln!("Error: Received non-success status code from OpenAI API: {}", follow_up_response.status());
    let error_text = follow_up_response.text().unwrap_or_default();
    eprintln!("Response body: {}", error_text);
    std::process::exit(1);
}

// Function to process the user prompt
fn process_prompt(prompt: &str, no_execute: bool) {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY not set in environment.");
            std::process::exit(1);
        }
    };

    let client = Client::new();

    let request_body = OpenAIRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: format!(
                "Translate the following prompt into a bash command without explanation:\n{}",
                prompt
            ),
        }],
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&request_body)
        .send();

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let openai_response: OpenAIResponse = response.json().unwrap();
                let command = openai_response.choices[0]
                    .message
                    .content
                    .trim()
                    .to_string();

                if no_execute {
                    println!("{}", command);
                } else {
                    println!("\nGenerated Command:\n{}\n", command);
                    // Updated prompt message
                    print!("Do you want to execute this command? (Y/n) ");
                    io::stdout().flush().unwrap();

                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation).unwrap();
                    let confirmation = confirmation.trim();

                    if confirmation.eq_ignore_ascii_case("n") || confirmation.eq_ignore_ascii_case("no") {
                        println!("Command execution cancelled.");
                    } else {
                        // Default to 'yes' if input is empty or any other input
                        execute_command(&command);
                    }
                }
            } else {
                eprintln!(
                    "Error: Received non-success status code from OpenAI API: {}",
                    response.status()
                );
                let error_text = response.text().unwrap_or_default();
                eprintln!("Response body: {}", error_text);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error communicating with OpenAI API: {}", e);
            std::process::exit(1);
        }
    }
}

// Function to execute the command
fn should_execute_command(command: &str) -> Result<(), String> {
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

fn execute_command(command: &str) {
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

// Function to check if the command is a shell built-in
fn is_shell_builtin(command: &str) -> bool {
    let builtins = ["cd", "export", "alias", "source", "unset"];
    let command = command.trim();
    if command.is_empty() {
        return false;
    }
    let first_word = command.split_whitespace().next().unwrap_or("");
    builtins.contains(&first_word)
}

// Function to execute shell commands (used in function calling)
fn execute_shell_command(args: &serde_json::Map<String, Value>) -> Value {
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

// Data structures for OpenAI API request and response
#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}
