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
use reqwest::blocking::{Client, Response};
use crate::cli::execute_command;
use crate::models::{Message, OpenAIRequest, OpenAIResponse};

pub(crate) fn handle_non_success(follow_up_response: Response) {
    eprintln!("Error: Received non-success status code from OpenAI API: {}", follow_up_response.status());
    let error_text = follow_up_response.text().unwrap_or_default();
    eprintln!("Response body: {}", error_text);
    std::process::exit(1);
}

// Function to process the user prompt
pub(crate) fn process_prompt(prompt: &str, no_execute: bool) {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY not set in environment.");
            std::process::exit(1);
        }
    };

    let client = Client::new();

    let request_body = OpenAIRequest {
        model: "gpt-40".to_string(),
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
