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

use serde::{Deserialize, Serialize};

// Data structures for OpenAI API request and response
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Message {
    pub(crate) role: String,
    pub(crate) content: String,
}

#[derive(Serialize)]
pub(crate) struct OpenAIRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
}

#[derive(Deserialize)]
pub(crate) struct OpenAIResponse {
    pub(crate) choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub(crate) struct Choice {
    pub(crate) message: MessageContent,
}

#[derive(Deserialize)]
pub(crate) struct MessageContent {
    pub(crate) content: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// Additional context provided to the LLM to tailor command generation.
    pub context: Option<String>,
}
