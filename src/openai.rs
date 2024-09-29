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
    io::{self, Write},
    sync::{Arc, Mutex},
    thread
    ,
};

use reqwest::blocking::{Client, Response};

use crate::{
    cli::execute_command,
    models::{Message, OpenAIRequest, OpenAIResponse},
};
use crate::utils::start_loading_animation;

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

/// Processes the user prompt and communicates with the OpenAI API.
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
                let command = openai_response.choices[0].message.content.trim().to_string();

                if no_execute {
                    println!("{}", command);
                } else {
                    println!("\nGenerated Command:\n{}\n", command);

                    // Prompt user for confirmation
                    print!("Do you want to execute this command? (Y/n) ");
                    io::stdout().flush().unwrap();

                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation).unwrap();
                    let confirmation = confirmation.trim();

                    if confirmation.eq_ignore_ascii_case("n") || confirmation.eq_ignore_ascii_case("no") {
                        println!("Command execution cancelled.");
                    } else {
                        // Default to 'yes' if input is empty or any other input
                        let parsed_command = extract_command(&command).unwrap_or(&command);
                        execute_command(parsed_command);
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


