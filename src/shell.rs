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

use crate::process_prompt;
use colored::Colorize;
use std::io::Write;
use std::{env, io};

// Function to run continuous shell mode
pub(crate) fn run_shell_mode(no_execute: bool) {
    println!("Entering continuous shell mode. Type 'exit' to quit.");
    loop {
        let working_directory = get_current_dir_with_tilde();
        let username = get_username();
        let prefix = "gptsh";
        print!("[{}]:{}:{}$ ", prefix.red(), username.green(), working_directory.blue());
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

fn get_current_dir_with_tilde() -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let home_dir = dirs::home_dir().expect("Failed to get home directory");

    let current_dir_str = current_dir.to_str().expect("Current directory path is not valid UTF-8");
    let home_dir_str = home_dir.to_str().expect("Home directory path is not valid UTF-8");

    if current_dir_str.starts_with(home_dir_str) {
        current_dir_str.replacen(home_dir_str, "~", 1)
    } else {
        current_dir_str.to_string()
    }
}

fn get_username() -> String {
    env::var("USER").unwrap_or_else(|_| "Unknown User".to_string())
}
