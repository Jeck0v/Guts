# 🧬 `nix` Support for `guts`

> Guts is a Git-like version control system written in Rust.  
> With Nix, you can build, run, install, and develop `guts` easily  on Linux, macOS, and even WSL.

---

## 📦 Install via Nix

### 🔁 Prerequisites

Install [Nix](https://nixos.org/download) with:

```bash
curl -L https://nixos.org/nix/install | sh
```

Enable **flakes** support (if not already enabled):

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

---

## 🚀 Quick Install & Usage

### 🔧 Build locally:

```bash
nix build
./result/bin/guts --help
```

### 🚀 Run directly (without installing):

```bash
nix run github:Jeck0v/guts
```

### 🖥️ Install globally into your system profile:

```bash
nix profile install github:Jeck0v/guts
```

You can then run:

```bash
guts init
guts hash-object
guts help
...
```

---

## ⚙️ Development Mode

Enter a development shell with Rust, Cargo, OpenSSL... pre-installed:

```bash
nix develop
```

This gives you an isolated environment to build, test, and contribute without polluting your system.

---

## 🛠️ Flake Structure Overview

The `flake.nix` file provides:

| Component            | Command         | Description                                         |
|----------------------|------------------|-----------------------------------------------------|
| `packages.default`   | `nix build`      | Builds the `guts` binary                            |
| `apps.default`       | `nix run`        | Runs the main CLI binary                            |
| `devShells.default`  | `nix develop`    | Full Rust development environment                   |

---

## ✅ Examples

```bash
# Build
nix build
./result/bin/guts init

# Run directly
nix run github:Jeck0v/guts -- init

# Install permanently
nix profile install github:Jeck0v/guts
guts init

# Enter dev environment
nix develop
cargo build
```

---

## 📚 Resources

- [Official Nix documentation](https://nixos.org/manual/nix/stable/)
- [Nix flakes overview](https://nixos.wiki/wiki/Flakes)
- [Guts GitHub repository](https://github.com/Jeck0v/guts)
