use reqwest::blocking::Client;
use serde_json::Value;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{env, io, thread};
use std::time::Duration;
use crate::openai::{handle_non_success, start_loading_animation};

pub(crate) fn run_chat_mode() {
    announce_entry_to_chat_mode();
    let api_key = fetch_api_key();
    let client = Client::new();
    let mut messages = initialize_messages_with_system_prompt();

    loop {
        let user_input = read_user_input().trim().to_string();
        if should_exit(&user_input) {
            break;
        }

        if user_input.is_empty() {
            continue;
        }

        add_user_message_to_conversation(&mut messages, &user_input);
        let request_body = prepare_request_body(&messages);

        let stop_signal = start_loading_indicator();
        let response = send_request_to_openai_api(&client, &api_key, &request_body);
        stop_loading_indicator(stop_signal);

        match process_response(response, &mut messages, &client, &api_key) {
            Some(true) => {
                println!("See you later pal.");
                break;
            }
            Some(false) => continue,
            None => {}
        }
    }
}

fn announce_entry_to_chat_mode() {
    println!("Entering chat mode. Type 'exit' or 'quit' to end the session.");
}

fn fetch_api_key() -> String {
    env::var("OPENAI_API_KEY").expect("Error: OPENAI_API_KEY not set in environment.")
}

fn initialize_messages_with_system_prompt() -> Vec<Value> {
    vec![serde_json::json!({
        "role": "system",
        "content": "You are a helpful assistant chatting in a terminal, use proper formatting so that your answers are easy to read. Address the user as pal or buddy."
    })]
}

fn read_user_input() -> String {
    print!("You: ");
    io::stdout().flush().unwrap();
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();
    user_input
}

fn should_exit(user_input: &str) -> bool {
    if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
        println!("Exiting chat mode.");
        true
    } else {
        false
    }
}

fn add_user_message_to_conversation(messages: &mut Vec<Value>, user_input: &str) {
    messages.push(serde_json::json!({
        "role": "user",
        "content": user_input
    }));
}

fn prepare_request_body(messages: &Vec<Value>) -> Value {
    serde_json::json!({
        "model": "gpt-4",
        "messages": messages,
        "functions": function_definitions(),
        "function_call": "auto"
    })
}

fn function_definitions() -> Vec<Value> {
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
        })
    ]
}

fn start_loading_indicator() -> Arc<Mutex<bool>> {
    let stop_signal = Arc::new(Mutex::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);
    thread::spawn(move || {
        start_loading_animation(stop_signal_clone);
    });
    stop_signal
}

fn send_request_to_openai_api(
    client: &Client,
    api_key: &str,
    request_body: &Value,
) -> reqwest::Result<reqwest::blocking::Response> {
    client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(request_body)
        .send()
}

fn stop_loading_indicator(stop_signal: Arc<Mutex<bool>>) {
    *stop_signal.lock().unwrap() = true;
    thread::sleep(Duration::from_millis(100));
    print!("\x1b[2K");
    print!("\r");
    io::stdout().flush().unwrap();
}

fn process_response(
    response: reqwest::Result<reqwest::blocking::Response>,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
) -> Option<bool> {
    match response {
        Ok(response) if response.status().is_success() => {
            let openai_response: Value = response.json().unwrap();
            handle_openai_response(openai_response, messages, client, api_key)
        }
        Ok(response) => {
            handle_non_success(response);
            None
        }
        Err(e) => {
            eprintln!("Error communicating with OpenAI API: {}", e);
            None
        }
    }
}

fn handle_openai_response(
    openai_response: Value,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
) -> Option<bool> {
    let choices = openai_response["choices"].as_array().unwrap();
    let choice = &choices[0];
    let message = &choice["message"];

    // Add the assistant's message to the conversation history
    let mut assistant_message = serde_json::json!({
        "role": "assistant",
    });

    if let Some(content) = message["content"].as_str() {
        assistant_message["content"] = serde_json::Value::String(content.to_string());
    }

    if let Some(function_call) = message.get("function_call") {
        assistant_message["function_call"] = function_call.clone();
    }

    messages.push(assistant_message);

    if let Some(function_call) = message.get("function_call") {
        handle_function_call(function_call, messages, client, api_key)
    } else {
        println!(
            "\ngptsh: {}\n",
            message["content"].as_str().unwrap_or("").trim()
        );
        None
    }
}

fn handle_function_call(
    function_call: &Value,
    messages: &mut Vec<Value>,
    client: &Client,
    api_key: &str,
) -> Option<bool> {
    let function_name = function_call["name"].as_str().unwrap();

    match function_name {
        "execute_command" => {
            execute_command_function(function_call, messages);

            // After executing the function, make another API call
            let request_body = prepare_request_body(&messages);

            let stop_signal = start_loading_indicator();
            let response = send_request_to_openai_api(client, api_key, &request_body);
            stop_loading_indicator(stop_signal);

            match process_response(response, messages, client, api_key) {
                Some(true) => Some(true),
                Some(false) => None,
                None => None,
            }
        }
        "exit_chat" => Some(true),
        _ => {
            eprintln!(
                "Error: Assistant requested an unknown function '{}'.",
                function_name
            );
            None
        }
    }
}

fn execute_command_function(function_call: &Value, messages: &mut Vec<Value>) {
    let arguments_str = function_call["arguments"].as_str().unwrap_or_default();
    let arguments: Value =
        serde_json::from_str(arguments_str).unwrap_or_else(|_| serde_json::json!({}));
    let command = arguments["command"].as_str().unwrap_or_default();

    println!("About to execute command: '{}'", command);
    println!("Do you want to proceed? [Y/n]");

    let mut confirmation = String::new();
    io::stdin()
        .read_line(&mut confirmation)
        .expect("Failed to read line");
    if confirmation.trim().is_empty() || confirmation.trim().eq_ignore_ascii_case("y") {
        // Adjust the command for 'ls' to force column output
        let adjusted_command = if command.trim() == "ls" {
            "ls -C"
        } else {
            command
        };

        // Execute the command
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(adjusted_command)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stdout.is_empty() {
                    println!("Command output:\n{}", stdout);
                }
                if !stderr.is_empty() {
                    eprintln!("Command error:\n{}", stderr);
                }

                // Add the command's output to messages for further processing or display
                messages.push(serde_json::json!({
                    "role": "function",
                    "name": "execute_command",
                    "content": stdout.to_string()
                }));
            }
            Err(e) => {
                let error_message = format!("Failed to execute command: {}", e);
                eprintln!("{}", error_message);
            }
        }

        // Flush stdout to ensure all output is written to the terminal
        io::stdout().flush().expect("Failed to flush stdout");
    } else {
        println!("Command execution cancelled.");
    }
}
