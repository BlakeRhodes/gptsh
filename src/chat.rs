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

use std::{env, io, thread};
use std::io::Write;
use std::sync::{Arc, Mutex};
use reqwest::blocking::Client;
use serde_json::Value;
use crate::cli::execute_shell_command;
use crate::openai::{handle_non_success, start_loading_animation};

// Function to run chat mode
pub(crate) fn run_chat_mode() {
    println!("Entering chat mode. Type 'exit' or 'quit' to end the session.");

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
            "model": "gpt-4o",
            "messages": messages,
            "functions": function_definitions,
            "function_call": "auto"
        });

        // Start loading animation
        let stop_signal = Arc::new(Mutex::new(false));
        let stop_signal_clone = Arc::clone(&stop_signal);
        let handle = thread::spawn(move || {
            start_loading_animation(stop_signal_clone);
        });

        // Send the request to OpenAI API
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&request_body)
            .send();

        // Stop loading animation
        *stop_signal.lock().unwrap() = true;
        handle.join().unwrap(); // Wait for the animation thread to finish

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
                                "model": "gpt-4o",
                                "messages": messages
                            });

                            // Start loading animation
                            let stop_signal = Arc::new(Mutex::new(false));
                            let stop_signal_clone = Arc::clone(&stop_signal);
                            let handle = thread::spawn(move || {
                                start_loading_animation(stop_signal_clone);
                            });

                            let follow_up_response = client
                                .post("https://api.openai.com/v1/chat/completions")
                                .bearer_auth(&api_key)
                                .json(&follow_up_request_body)
                                .send();

                            // Stop loading animation
                            *stop_signal.lock().unwrap() = true;
                            handle.join().unwrap(); // Wait for the animation thread to finish


                            if let Ok(follow_up_response) = follow_up_response {
                                if follow_up_response.status().is_success() {
                                    let follow_up_openai_response: Value = follow_up_response.json().unwrap();
                                    let follow_up_choices = follow_up_openai_response["choices"].as_array().unwrap();
                                    let follow_up_choice = &follow_up_choices[0];
                                    let assistant_reply = follow_up_choice["message"]["content"].as_str().unwrap_or("").trim();

                                    println!("\ngptsh: {}\n", assistant_reply);
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
                        println!("\ngptsh: {}\n", assistant_reply);
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
