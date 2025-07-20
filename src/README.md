# Explanation of folder structure
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
├── fixtures/                       # Test data (used by integration tests)
├── .gitignore
├── Cargo.toml                      # Project configuration
└── README.md
```
---
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
>`main.rs` is the controller, `cli.rs` analyzes user commands, `commands/` executes actions, and `core/` contains the real technical building blocks. <br>
Everything is modular, testable and easy to evolve.
