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

use std::io;
use std::io::Write;
use crate::process_prompt;

// Function to run continuous shell mode
pub(crate) fn run_shell_mode(no_execute: bool) {
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