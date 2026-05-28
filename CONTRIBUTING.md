# Contributing to Sticky

First off, thanks for taking the time to contribute!

## Local Development Setup

1. **Install Prerequisites (Ubuntu/Debian)**
   ```bash
   sudo apt install libgtk-4-dev libadwaita-1-dev libsqlite3-dev speech-dispatcher alsa-utils
   ```
2. **Clone and Build**
   ```bash
   git clone https://github.com/Mrudula-itsjuzme/Sticky.git
   cd Sticky
   cargo build
   ```
3. **Run locally**
   ```bash
   cargo run
   ```

*Note: Running `cargo run` will use local paths and will not overwrite your system's desktop launcher.*

## Pull Request Process

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes: `cargo test --all`.
5. Run the formatter: `cargo fmt --all`.
6. Run the linter: `cargo clippy --all-targets --all-features -- -D warnings`.
7. Issue that pull request!

