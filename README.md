# gptsh

A command-line tool written in Rust that translates user prompts into bash commands using OpenAI's GPT model. It confirms the command with the user before execution to ensure safety and accuracy.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
    - [1. Install Rust](#1-install-rust)
    - [2. Clone the Repository](#2-clone-the-repository)
    - [3. Install OpenSSL Development Libraries](#3-install-openssl-development-libraries)
    - [4. Set Up Environment Variables](#4-set-up-environment-variables)
    - [5. Build the Project](#5-build-the-project)
    - [6. Run the Program](#6-run-the-program)
- [Usage](#usage)
    - [Single Command Mode](#single-command-mode)
    - [Continuous Shell Mode](#continuous-shell-mode)
    - [Help](#help)
- [Examples](#examples)
    - [Example 1: Single Command Mode](#example-1-single-command-mode)
    - [Example 2: Continuous Shell Mode](#example-2-continuous-shell-mode)
- [Troubleshooting](#troubleshooting)
    - [Build Error: Could not find directory of OpenSSL installation](#build-error-could-not-find-directory-of-openssl-installation)
    - [Error: OPENAI_API_KEY not set in environment](#error-openai_api_key-not-set-in-environment)
    - [Error communicating with OpenAI API](#error-communicating-with-openai-api)
    - [Command exited with non-zero status](#command-exited-with-non-zero-status)
    - [OpenAI API Rate Limits](#openai-api-rate-limits)
- [Notes](#notes)
- [License](#license)

## Features

- **Natural Language to Bash**: Translates natural language prompts into bash commands.
- **User Confirmation**: Displays the generated command and asks for user confirmation before execution.
- **Continuous Shell Mode**: Interactive shell mode for multiple commands.
- **Robust Help Feature**: Provides usage instructions and options.

## Prerequisites

- **Linux Operating System**: The program is designed to work on Linux systems.
- **Rust**: Ensure Rust is installed on your system. [Install Rust](https://www.rust-lang.org/tools/install)
- **OpenSSL Development Libraries**: Required for building the project (see [Installation](#installation)).
- **OpenAI API Key**: An API key from OpenAI set as the environment variable `OPENAI_API_KEY`.
- **Internet Connection**: Required to communicate with the OpenAI API.

## Installation

### 1. Install Rust

If Rust is not already installed on your system, install it using the following command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation. After installation, ensure that Cargo (Rust's package manager) is in your system's `PATH`.

Verify the installation:

```bash
rustc --version
cargo --version
```

### 2. Clone the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/yourusername/gptsh.git
cd gptsh
```

> **Note**: Replace `yourusername` with your GitHub username if you have forked the repository.

### 3. Install OpenSSL Development Libraries

The OpenSSL development libraries are required for building the `openssl-sys` crate, which is a dependency of the `reqwest` crate used in the project.

If you encounter an error during the build process similar to the following:

```
Could not find directory of OpenSSL installation, and this `-sys` crate cannot proceed without this knowledge. ...
```

This means that the OpenSSL development libraries are not installed or not found.

#### For Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install libssl-dev pkg-config
```

#### For Fedora/CentOS/RHEL

```bash
sudo dnf install openssl-devel pkg-config
```

#### For Arch Linux

```bash
sudo pacman -S openssl pkgconf
```

### 4. Set Up Environment Variables

Set your OpenAI API key in the terminal session:

```bash
export OPENAI_API_KEY=your_api_key_here
```

Alternatively, you can use a `.env` file in the root directory of the project:

```bash
echo OPENAI_API_KEY=your_api_key_here > .env
```

> **Warning**: Ensure that the `.env` file is included in `.gitignore` to prevent accidentally committing your API key to version control.

### 5. Build the Project

Use Cargo to build the project in release mode:

```bash
cargo build --release
```

**Possible Build Error**:

If you encounter an error like the following:

```
Could not find directory of OpenSSL installation, and this `-sys` crate cannot proceed without this knowledge. ...
```

Refer to [Step 3](#3-install-openssl-development-libraries) and the [Troubleshooting](#build-error-could-not-find-directory-of-openssl-installation) section for detailed steps to resolve this issue.

### 6. Run the Program

You can run the program directly from the `target/release` directory:

```bash
./target/release/gptsh --help
```

Alternatively, you can install the program to your system's binary directory:

```bash
cargo install --path .
```

This will compile the program and place the executable in Cargo's binary directory (usually `~/.cargo/bin`), which should be in your `PATH`.

Verify the installation:

```bash
gptsh --help
```

## Usage

### Single Command Mode

Run the program with a prompt as an argument:

```bash
gptsh "your prompt here"
```

**Example**:

```bash
gptsh "List all files in the current directory including hidden ones"
```

The program will:

1. Translate your prompt into a bash command.
2. Display the generated command.
3. Ask for confirmation before executing it.

### Continuous Shell Mode

Start the program in continuous shell mode using the `--shell` option:

```bash
gptsh --shell
```

**Features**:

- Acts as an interactive shell.
- Type `exit` to leave the shell mode.
- Each prompt will go through the same process of translation, display, and confirmation.

### Help

Display the help message using `--help` or `-h`:

```bash
gptsh --help
```

## Examples

### Example 1: Single Command Mode

```bash
$ gptsh "Show the disk usage of the current directory"
```

**Output**:

```
Generated Command:
du -sh .

Do you want to execute this command? (y/n)
```

- **If you type `y`**: The command `du -sh .` will be executed, showing the disk usage.
- **If you type `n`**: The command will not be executed.

### Example 2: Continuous Shell Mode

```bash
$ gptsh --shell
Entering continuous shell mode. Type 'exit' to quit.
gptsh> Find all `.txt` files in the current directory
```

**Output**:

```
Generated Command:
find . -name "*.txt"

Do you want to execute this command? (y/n)
```

- **Continue**: After execution or cancellation, the prompt will reappear for the next command.
- **Exit**: Type `exit` to leave the shell mode.

## Troubleshooting

### Build Error: Could not find directory of OpenSSL installation

**Error Message Example**:

```
Could not find directory of OpenSSL installation, and this `-sys` crate cannot proceed without this knowledge. ...

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
warning: build failed, waiting for other jobs to finish...
```

**Cause**:

The Rust compiler cannot find the OpenSSL development libraries required by the `openssl-sys` crate.

**Solution**:

Install the OpenSSL development libraries for your system as described in [Step 3](#3-install-openssl-development-libraries). After installing, you may need to:

- **Clean the Build Cache**:

  ```bash
  cargo clean
  ```

- **Rebuild the Project**:

  ```bash
  cargo build --release
  ```

### Error: OPENAI_API_KEY not set in environment

**Error Message**:

```
Error: OPENAI_API_KEY not set in environment.
```

**Cause**:

The `OPENAI_API_KEY` environment variable is not set or cannot be found by the program.

**Solution**:

Set the `OPENAI_API_KEY` environment variable as described in [Step 4](#4-set-up-environment-variables).

**Example**:

```bash
export OPENAI_API_KEY=sk-...
```

### Error communicating with OpenAI API

**Error Message**:

```
Error communicating with OpenAI API: ...
```

**Possible Causes**:

- **Invalid API Key**: The API key is incorrect or has been revoked.
- **Network Issues**: No internet connection or firewall blocking the request.
- **API Endpoint Changed**: The OpenAI API endpoint has been updated.
- **Rate Limits**: You have exceeded the API rate limits.

**Solution**:

- **Check API Key**: Ensure that your `OPENAI_API_KEY` is correct and active.
- **Verify Internet Connection**: Make sure you have an active internet connection.
- **Check OpenAI Status**: Visit [OpenAI's status page](https://status.openai.com/) to see if there are any outages.
- **Inspect Firewall Settings**: Ensure that your firewall or proxy is not blocking outbound HTTPS requests.

### Command exited with non-zero status

**Error Message**:

```
Command exited with non-zero status.
```

**Cause**:

The executed command returned a non-zero exit code, indicating an error.

**Solution**:

- **Review the Command**: Check the generated command for correctness.
- **Run Manually**: Try running the command manually in your terminal to see the detailed error.
- **Modify Prompt**: Rephrase your prompt for clarity to generate a correct command.

### OpenAI API Rate Limits

**Issue**:

Receiving errors related to rate limits or too many requests, such as HTTP 429 errors.

**Cause**:

You have exceeded the rate limits set by OpenAI for API usage.

**Solution**:

- **Wait and Retry**: Wait for a few minutes before making new requests.
- **Optimize Usage**: Reduce the frequency of requests or batch them if possible.
- **Upgrade Plan**: If you frequently hit rate limits, consider upgrading your OpenAI plan.

## Notes

- **Review Commands**: Always read the generated command carefully before confirming execution.
- **Security**: Be cautious of commands that could alter or delete data.
- **API Usage**: Using the OpenAI API may incur costs. Monitor your usage to avoid unexpected charges.
- **Error Handling**: The program includes basic error handling, but you may need to troubleshoot issues based on specific error messages.
- **Compatibility**: The program is designed to work on Linux systems.

## License

This project is licensed under the [MIT License](LICENSE).
