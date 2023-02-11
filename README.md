# pogodoro — a poggers-as-hell terminal UI pomodoro timer

<img width="908" alt="pomo_tasks" src="https://user-images.githubusercontent.com/80245312/218248183-88150b48-c1ba-4721-87ac-ba80124d051c.png">
<img width="231" alt="pomo_work" src="https://user-images.githubusercontent.com/80245312/217387842-86462338-ce3b-4ed7-a474-7d24670ac6a6.png">

## Features

- Built-in task manager
- Persistent pomodoro sessions
- Streamlined UI experience
- Notifications with each cycle
- Support for macOS and Linux

## Installation

This is a Rust package, and I haven't prebuilt any binaries, so will need to be compiled.
`rustup` (the Rust version manager) can be installed [here](https://www.rust-lang.org/learn/get-started).

I'm working on a cleaner way of doing this, but for the time being, installation requires `sqlx-cli`, and the creation of a folder in `~/.config`:

```bash
cargo install sqlx-cli
mkdir ~/.config/pogodoro
```

From here we can proceed with actual installation:

```bash
git clone https://github.com/joshcbrown/pogodoro.git
cd pogodoro
sqlx db setup # important!
cargo install --path .
```

## Usage

See below for a brief video going through the main features of the UI, which can be accessed by running just `pogodoro`:

https://user-images.githubusercontent.com/80245312/218248194-1d4f2c2e-7845-491a-bcd7-a32ea1c76b17.mp4

If you're confused because that went too quickly, or prefer to learn by reading, each page of the app has a built-in help page which is toggled by pressing '?'.

All the functionality in the demo video can be replicated with command-line commands:

```
A poggers-as-hell terminal UI pomodoro timer

Usage: pogodoro [COMMAND]

Commands:
  list      Lists incomplete tasks
  add       Adds task to DB
  complete  Completes a task with given ID
  work-on   Start a pomodoro session working on task with given ID
  start     Starts a (non-default) pomo session
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

All commands have dedicated help pages which can be accessed with `pogodoro <COMMAND> -h`

Just for fun, I like to add `alias pog='pogodoro'` to my .zshrc :)

Big shoutout to [orhun](https://github.com/orhun/) for his [tui-rs template](https://github.com/orhun/rust-tui-template), which I am using as a base for this project.
