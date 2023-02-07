# pogodoro — A poggers-as-hell terminal UI pomodoro timer

<img width="1058" alt="pomo_tasks" src="https://user-images.githubusercontent.com/80245312/217387835-768fc441-9a58-4a24-bd30-391a58c6f7a7.png">
<img width="231" alt="pomo_work" src="https://user-images.githubusercontent.com/80245312/217387842-86462338-ce3b-4ed7-a474-7d24670ac6a6.png">

## Features

- Built-in task manager
- Persistent pomodoro sessions
- Streamlined UI experience
- Not a whole lot else, really

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

See below for a brief video going through the main features of the UI:

https://user-images.githubusercontent.com/80245312/217387755-ddd6638f-c5ff-4f6d-809c-62b9e0ab755b.mp4

If you're confused because that went too quickly, or prefer to learn by reading, each page of the app has a built-in help page which is toggled by pressing '?'.

The app can also be started without selecting a task, if you'd prefer a more minimal experience:

```pogodoro start <work_dur> <short_break_dur> <long_break_dur>```

Just for fun, I like to add `alias pog='pogodoro'` to my .zshrc :)

Big shoutout to [orhun](https://github.com/orhun/)'s [tui-rs template](https://github.com/orhun/rust-tui-template), which I am using as a base for this project.

