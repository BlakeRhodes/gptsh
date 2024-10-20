![OpenAI](https://a11ybadges.com/badge?logo=openai)  
[![Rust](https://github.com/BlakeRhodes/gptsh/actions/workflows/rust.yml/badge.svg)](https://github.com/BlakeRhodes/gptsh/actions/workflows/rust.yml)
# GPTsh - A GPT-Powered Command Line Tool

`gptsh` is an experimental shell like environment that integrates GPT generated commands with human-in-the-loop for your workflows. Designed to enhance command-line productivity, it enables you to issue natural language commands, generate code snippets, and learn a new trick or two.

## Usage Examples

Once `gptsh` is installed, you can start interacting with the shell using natural language commands or standard shell syntax.

### Example 1: Running Shell Commands
````
$ gptsh list all the files in the current directory
 

Generated Command:
```bash
ls
```

Do you want to execute this command? (Y/n) Y
CoolTerm  Documents  Music     Public		  snap	     Videos
Desktop   Downloads  Pictures  RustroverProjects  Templates
````


### Example 2: Generating a Python Script
````
$ gptsh generate a python script to find all my python files
 

Generated Command:
```bash
echo -e "import os\n\nfor root, dirs, files in os.walk('.'): \n\tfor file in files:\n\t\tif file.endswith('.py'):\n\t\t\tprint(os.path.join(root, file))" > find_python_files.py
```

Do you want to execute this command? (Y/n) Y
$ ls
CoolTerm   Downloads             Pictures           snap      
Desktop    find_python_files.py  Public             Templates
Documents  Music                 RustroverProjects  Videos


````

### Example 3: Task Automation
````
$ gptsh create the automation to compression and back up my pictures weekly 

Generated Command:
```bash
echo "0 0 * * 0 tar -czf \$HOME/backup/pictures_backup_\$(date +\%Y\%m\%d).tar.gz -C \$HOME/Pictures ." | crontab -
```

Do you want to execute this command? (Y/n) y
$ gptsh show me my cron jobs
 

Generated Command:
```bash
crontab -l
```

Do you want to execute this command? (Y/n) 
0 0 * * 0 tar -czf $HOME/backup/pictures_backup_$(date +\%Y\%m\%d).tar.gz -C $HOME/Pictures .
````

### Solve Complex Social Issues
````
$ gptsh create a single bash script to solve world hunger and bring world peace
 

Generated Command:
```bash
#!/bin/bash
echo "Solving world hunger and bringing world peace cannot be achieved by a script. It requires global cooperation, sustainable development, and compassionate leadership."
```

Do you want to execute this command? (Y/n) Y           
Solving world hunger and bringing world peace cannot be achieved by a script. It requires global cooperation, sustainable development, and compassionate leadership.
````

At least we tried.

## Features

- **Chat Mode**: Interact with GPT-4o to generate shell commands or general responses.
- **Continuous Shell Mode**: Run shell commands generated by GPT-4o in an interactive environment.
- **No Execute Mode**: Output the generated shell commands without running them.
- **Environment Integration**: Load configuration from a `.env` file, including the OpenAI API key.

## Usage

### Prerequisites

Before building or using this project, ensure you have the following:

- Rust programming language installed. You can get Rust [here](https://www.rust-lang.org/learn/get-started).
- OpenAI API key: You will need an API key from OpenAI to use the chat features. Set it in a `.env` file.

`gptsh` can be run in various modes depending on your needs.

### Providing Prompts

You can provide a shell prompt directly via the command line:

````
$ gptsh creat a new directory and add a couple of sample files to it
 

Generated Command:
```bash
mkdir new_directory && touch new_directory/sample_file1.txt new_directory/sample_file2.txt
```

Do you want to execute this command? (Y/n) 

````


### Help

To view help and available commands:

```
gptsh --help
Usage: gptsh [OPTIONS] [PROMPT]
Options:
--help, -h        Show this help message
--shell           Run in continuous shell mode
--chat            Run in chat mode with GPT-4
--no-execute      Output the generated command without executing it
```

### Continuous Shell Mode

This mode allows you to interact with gptsh continuously, generating and executing commands on the fly.

````
$ gptsh --shell
Entering continuous shell mode. Type 'exit' to quit.
[gptsh]:user:~/RustroverProjects/gptsh$ clear out my chron jobs
 

Generated Command:
```bash
crontab -r
```

Do you want to execute this command? (Y/n) 
[gptsh]:user:~/RustroverProjects/gptsh$ 

````

### Chat Mode

This mode interacts with GPT-4 as a chat assistant.

````
$ gptsh --chat
Entering chat mode. Type 'exit' or 'quit' to end the session.
You: What's up frendo?
 

gptsh: Hello, pal! I'm here to assist you. How may I aid you today?

You: tell me about how write good software
 

gptsh: Absolutely, buddy! Writing good software revolves around several principles:

**1. Understand the Problem:** Before writing a single line of code, it's essential to have a clear understanding of the problem you aim to solve through software.

**2. Plan:**
Layout a blueprint of your software architecture. Sketch workflows and identify components and their interactions.

**3. Write Clean and Readable Code:**
Your code should be easy to understand and maintain. Consistently follow naming conventions, use accurate comments, and avoid coding practices that make your code complex.

**4. Maintainability:**
The codebase should be organised to make updates and modifications easy. High cohesion (grouping related functionalities) and low coupling (minimise interdependence between components) are key principles here.

**5. Test Thoroughly:**
Robust software is tested well before deployment. Automated tests, manual testing, and even user acceptance testing help identify bugs and issues before they become problems.

**6. Refine and Refactor:**
Good software is continuously improved. Regularly check for areas of improvement and refactor code to make it more efficient, readable or maintainable.

**7. Documentation:**
A well-documented codebase is a dream to work with, often easier to understand and debug. Usage, configuration, installation, and any other essential information should be clearly documented.

These are the fundamental principles of writing good software. However, the list doesn't end here. There are many more practices that depend on context, personal experience, team guidelines, etc.

You: Awesome! save that for me to read later
 

gptsh: Sure thing, pal!

 
About to execute command: 'echo '1. Understand the Problem: Before writing a single line of code, it\'s essential to have a clear understanding of the problem you aim to solve through software.

...

7. Documentation: A well-documented codebase is a dream to work with, often easier to understand and debug. Usage, configuration, installation, and any other essential information should be clearly documented.
' > good_software_guidelines.md'
Do you want to proceed? [Y/n] Y

 

gptsh: Done, pal! I have written the information about good software practices to a file named `good_software_guidelines.md`. You can access it anytime for your reference!

Anything else I can assist you with today?

You: nope, let's call it a day
 
See you later pal.
$

````

### No Execute Mode

If you want gptsh to generate commands but not execute them, use the `--no-execute` flag:

````
$ gptsh --no-execute remove all these dang files!
 
```bash
rm -rf *
```

````

## Building from Source

Follow these steps to build gptsh from source:

1. Ensure you have Rust installed on your machine. Install Rust [here](https://www.rust-lang.org/learn/get-started) if needed.

2. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/gptsh.git
   cd gptsh
   ```

3. Build the project in release mode:

   ```bash
   cargo build --release
   ```

4. After building, the binary will be located in the `target/release` directory:

   ```bash
   ./target/release/gptsh --help
   ```


---

## Contributing to `gptsh`

Thank you for considering contributing to `gptsh`! We welcome contributions from developers of all experience levels. Please follow the guidelines below to ensure a smooth process.

### Getting Started

1. **Fork the Repository**: Start by forking the repo and cloning it locally.
2. **Create a Branch**: Make a branch for your feature or bug fix (`git checkout -b feature-name`).
3. **Install Dependencies**: Run `cargo build` to set up the project.
4. **Make Changes**: Ensure your changes pass all tests (`cargo test`).
5. **Submit a Pull Request**: Once complete, submit a PR, describing your changes in detail.

### Code of Conduct

This project adheres to the [Code of Conduct](./CODE_OF_CONDUCT.md). 
Please be respectful in all interactions.

## Licensing

By contributing, you agree that your contributions will be licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

## Reporting Issues

If you encounter any issues or have questions, please open a GitHub issue with detailed information.

Copyright 2024 Blake Rhodes

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

The project contains ai generated code and documentation.
