# OpenDesktop

A lightweight, minimalist desktop app for viewing and managing Docker containers running inside WSL. Built with [Tauri v2](https://tauri.app/) for a tiny footprint and native performance.

![Windows](https://img.shields.io/badge/platform-Windows-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-orange)

## Features

- **View all containers** - equivalent to `docker ps -a`, but in a clean UI
- **Compose grouping** - containers from the same Docker Compose project are automatically grouped with collapsible headers
- **Start / Stop / Restart / Remove** individual containers
- **Start All / Stop All / Restart All** for entire Compose projects
- **Filter** by running or stopped state
- **Status indicators** - color-coded dots (green = running, red = exited, yellow = partial)
- **Dark theme** - easy on the eyes

## Prerequisites

- **Windows 10/11** with [WSL 2](https://learn.microsoft.com/en-us/windows/wsl/install) installed
- **Docker** installed and running inside WSL (e.g., Docker Engine on Ubuntu)
- For building from source:
  - [Node.js](https://nodejs.org/) (v18+)
  - [Rust](https://rustup.rs/) (stable)

## Installation

### Download Release

Download the latest `.msi` or `.exe` installer from the [Releases](https://github.com/alexjrk/OpenDesktop/releases) page.

### Build from Source

```bash
git clone https://github.com/alexjrk/OpenDesktop.git
cd OpenDesktop
npm install
npm run tauri build
```

The built installer will be in `src-tauri/target/release/bundle/`.

## Development

```bash
npm install
npm run tauri dev
```

This starts the app in development mode with hot-reload for the frontend. The first build compiles all Rust dependencies and takes a few minutes; subsequent builds are fast.

## Project Structure

```
OpenDesktop/
+-- src/                    # Frontend (HTML/CSS/JS)
|   +-- index.html
|   +-- main.js
|   +-- styles.css
+-- src-tauri/              # Rust backend (Tauri)
|   +-- src/
|   |   +-- lib.rs          # Tauri commands (Docker via WSL)
|   |   +-- main.rs         # Entry point
|   +-- Cargo.toml
|   +-- tauri.conf.json
+-- package.json
+-- README.md
```

## How It Works

OpenDesktop calls `wsl docker ps -a` from the Rust backend to list containers, parsing the output into structured data. Actions like start/stop/restart are executed via `wsl docker <action> <container-id>`. Compose operations use `wsl docker compose -p <project> <action>`.

The frontend is vanilla HTML/CSS/JS - no framework overhead. The Tauri webview renders the UI natively, resulting in a ~5MB app with minimal memory usage.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | Tauri v2 |
| Backend | Rust |
| Frontend | Vanilla HTML/CSS/JS |
| Docker access | WSL CLI bridge |

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
