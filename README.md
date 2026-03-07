# seher-cr

A wrapper for the [CodeRabbit](https://coderabbit.ai/) CLI that automatically retries when rate limit errors are encountered.

## Overview

When CodeRabbit hits a rate limit, it returns an error message like:

```
Rate limit exceeded. Try after 2 minutes and 7 seconds.
```

`scr` parses this message, waits the specified duration, and retries automatically — so you don't have to.

## Installation

### Homebrew

```sh
brew install smartcrabai/tap/scr
```

### Cargo

```sh
cargo install seher-cr
```

## Usage

`scr` passes all arguments through to `coderabbit --prompt-only`:

```sh
scr [args...]
```

This is equivalent to running `coderabbit --prompt-only [args...]`, but with automatic rate limit handling.

## How It Works

1. Runs `coderabbit --prompt-only` with the provided arguments
2. If the output contains a rate limit message (`Try after X minutes and Y seconds`), waits the specified duration and retries
3. On success, prints the output and exits with code 0
4. On other errors, prints the error and exits with the original exit code

## License

MIT
