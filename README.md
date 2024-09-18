<div align="center">
<img src="media/logo.png" height=100>

A shell built with AI at the core.

</div>

> This shell is still in the early stages of development and is **not ready** for production use.

## Features


With `aish`, you can directly interact with LLMs from your command line.

**Prompt it directly**

```
> "Hello, how are you?"
Hello, how can I assist you today?
```

**Utilise LLMs within pipelines**

```
> echo "Hello" | "translate to French"
Bonjour
```

**Pipe files into LLMs**

```
> cat passwords.txt | "summarize"
This is a file with a list of passwords.
```

## Getting Started

Make sure you have the latest version of [Rust](https://www.rust-lang.org) installed.

Then, clone this repository and run `cargo run` to start the shell.

```
git clone https://github.com/siliconlad/aish && cd aish && cargo run
```

### OpenAI Key

To use the llm, you need to provide an OpenAI API key.

Then create a file at `~/.aishrc` with the following:

```
export OPENAI_API_KEY=<your key>
```
