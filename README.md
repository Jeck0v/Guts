# <p align="center">Git from Scratch in Rust</p>
## Project Scope

The project implements a subset of Git commands, both low-level **(plumbing)** and user-facing **(porcelain)**. The goal was to ensure that they behave similarly to real Git, within clearly defined constraints.

### TUI installation for windows:
Go to `windows-installer` and just take the file `Guts_Installer.exe` and execute it. <br>
You now have a shortcut + the application in the PATH.
> The installer was made using Inno Setup Script (ISS) and PowerShell. For the sake of transparency, we've left the ISS file freely accessible, so you can read it and better understand what you're running. If you wish to modify the executable, you'll need to copy and add your own values to the config.iss file (which serves as the .env file for ISS)

### MacOs Installation:
[Homebrew Tap](https://github.com/Oomaxime/homebrew-guts) <br>
Now you can use the CLI app or the TUI app as you want.

## How to use the TUI app:
You need to enter:
``` bash
guts
```
### Folder Structure
```
gust/
├── src/
│   ├── main.rs                     # Entry point that redirects to the appropriate command
│   ├── lib.rs                      # Global setup & re-exports
│   ├── core/                       # Business logic (independent of CLI)
│   │   ├── blob.rs
│   │   ├── tree.rs
│   │   ├── hash.rs
│   │   ├── object.rs
│   │   └── ...
│   ├── commands/                   # Porcelain & plumbing commands
│   │   ├── init.rs
│   │   ├── hash_object.rs
│   │   └── ...
│   ├── terminal/                   # TUI Ratatui
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── run_app.rs
│   │   └── ui.rs
│   └── cli.rs                      # CLI argument parsing using `clap`
├── tests/                          # Integration tests
│   ├── test_init.rs
│   └── ...
├── .gitignore
├── Cargo.toml                      # Project configuration
└── README.md
```
---
<details>
        <summary>Explanation of folder structure</summary>

### main.rs
It launches the program <br>
It calls the CLI parser `cli.rs`, then sends to the right commands (commands/...)

### cli.rs
**It reads the arguments that the user types into the terminal (gust init, gust commit...)** <br>
Informs `main.rs` which command was invoked.
> 🧠 Think of it as the interpreter between the user and the code.
---

### The commands folder
It contains all the actions the user will perform, all the commands the user will use, and is where the functions created in the `core folder` will be used <br>
Each file corresponds to a command: init, add, commit...
> 🧠 It's like buttons on a machine: each button triggers a specific behavior

### The core folder
Contains generic, reusable logic: create Git objects, calculate hashes, manage indexes...<br>
**Never talk to the terminal! Just business functions**
> 🧠 It's like the machine's internal engine

### The terminal folder
It contains everything about **the TUI - Ratatui** <br>
This is where you call up and configure the commands you created earlier.
> 🧠 This is the graphical part of the project
---
### Summary
> `main.rs` is the controller, `cli.rs` analyzes user commands, `commands/` executes actions, and `core/` contains the real technical building blocks. And `terminal/` is just the grafical part of the project.
Everything is modular, testable and easy to evolve.
<br>

</details>

### Our Team:
- [Arnaud Fischer - Jeck0v ](https://github.com/Jeck0v)
- [Maxime Bidan - Max ](https://github.com/Oomaxime)
- [Louis Dondey - Kae ](https://github.com/Kae134)
- [Alexis Gontier - Algont ](https://github.com/Alexis-Gontier)

We organized ourselves using GitHub features, meaning we had a Project to which we added previously created issues.
### Bonus features: 
- `Retro-compatibility` management
- `.gutsignore` management
- Installation via `package managers` (HomeBrew / Chocolatery)
- Installation for Windows with a executable, with **creation of a shortcut and automatic addition to the PATH**
<details>
        <summary>TUI with Rataui</summary>
    The TUI uses widget features linked to Ratatui (formerly tui-rs), including some cool features:

- Display of `the scroll bar` when there is no more space to display items (can be used with Ctrl + up/down arrow)
- Use of all `system` commands in the TUI
- Use of all `guts` commands in the TUI
- Use of `nano (and vim)` in the TUI
</details>
