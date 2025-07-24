# Git from Scratch – Student Project

This project involves reimplementing core Git functionality in Rust. It will help you understand Git's internal architecture and make you comfortable with both its plumbing and porcelain commands.

##### Sub-repo:
[Homebrew Tap](https://github.com/Oomaxime/homebrew-guts)

#### Instalation:
https://github.com/Jeck0v/Guts/wiki/Installation-with-package-manager

## 🎯 Project Scope

You will implement a subset of Git commands, both low-level (plumbing) and user-facing (porcelain). Your goal is to ensure they behave similarly to real Git, within clearly defined constraints.

## How to use the TUI app:
You need to enter:
``` bash
guts
```
## TUI installation for windows:
Go to `windows-installer` and just take the file `Guts_Installer.exe` and execute it. <br>
You now have a shortcut + the application in the PATH.
> The installer was made using Inno Setup Script (ISS) and PowerShell. For the sake of transparency, we've left the ISS file freely accessible, so you can read it and better understand what you're running. If you wish to modify the executable, you'll need to copy and add your own values to the config.iss file (which serves as the .env file for ISS)
