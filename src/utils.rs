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
use std::time::Duration;

// Gets the current directory, replacing the home directory path with '~'
pub fn get_current_dir_with_tilde() -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let home_dir = dirs::home_dir().expect("Failed to get home directory");

    let current_dir_str = current_dir
        .to_str()
        .expect("Current directory path is not valid UTF-8");
    let home_dir_str = home_dir
        .to_str()
        .expect("Home directory path is not valid UTF-8");

    if current_dir_str.starts_with(home_dir_str) {
        current_dir_str.replacen(home_dir_str, "~", 1)
    } else {
        current_dir_str.to_string()
    }
}

// Retrieves the username from environment variables
pub fn get_username() -> String {
    env::var("USER").unwrap_or_else(|_| "Unknown User".to_string())
}

/// Starts the loading animation in a separate thread.
pub(crate) fn start_loading_animation(stop_signal: Arc<Mutex<bool>>) {
    let spinner_chars = ['/', '-', '\\', '|'];
    let mut i = 0;
    while !*stop_signal.lock().unwrap() {
        print!("\r{}", spinner_chars[i]);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
        i = (i + 1) % spinner_chars.len();
    }
    // Clear the spinner and move to a new line
    println!("\r ");
}