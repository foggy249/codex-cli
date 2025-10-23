# Code Review Bot

An AI-powered code review bot built with the Codex Agent Framework. This is a proof-of-concept application demonstrating how to build practical LLM agents using the framework.

## Features

- **File Review**: Analyze individual files for quality, bugs, and best practices
- **Directory Review**: Comprehensive review of entire codebases
- **Interactive Mode**: Ask questions and get insights about your code
- **Architecture Analysis**: Understand project structure and patterns
- **Actionable Recommendations**: Get prioritized suggestions for improvements

## Installation

```bash
cd code-review-bot
cargo build --release
```

## Usage

Set your OpenAI API key:
```bash
export OPENAI_API_KEY=sk-...
```

### Review a Single File

```bash
cargo run -- file path/to/file.rs
```

### Review an Entire Directory

```bash
cargo run -- dir path/to/project
```

### Interactive Mode

```bash
cargo run -- interactive
```

See the full README for detailed usage instructions and examples.
