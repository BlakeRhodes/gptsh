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
use crate::utils::{get_current_dir_with_tilde, get_username};
use colored::Colorize;
use std::io::Write;
use std::io;

// Runs the continuous shell mode
pub(crate) fn run_shell_mode(no_execute: bool) {
    println!("Entering continuous shell mode. Type 'exit' to quit.");
    let prefix = "gptsh";

    loop {
        let working_directory = get_current_dir_with_tilde();
        let username = get_username();

        display_prompt(prefix, &username, &working_directory);

        let prompt = read_user_input();

        if prompt.eq_ignore_ascii_case("exit") {
            break;
        }

        if !prompt.is_empty() {
            process_prompt(&prompt, no_execute);
        }
    }
}

// Displays the shell prompt
fn display_prompt(prefix: &str, username: &str, working_directory: &str) {
    print!(
        "[{}]:{}:{}$ ",
        prefix.red(),
        username.green(),
        working_directory.blue()
    );
    io::stdout().flush().unwrap();
}

// Reads user input from stdin
fn read_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

