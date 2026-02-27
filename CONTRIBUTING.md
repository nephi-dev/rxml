# Contributing to rxml

Thanks for your interest in contributing to rxml! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Python 3.10+
- A stable [Rust toolchain](https://rustup.rs/)
- [Maturin](https://www.maturin.rs/) (`pip install maturin`)

### Development Setup

1. Fork and clone the repository:

   ```bash
   git clone https://github.com/<your-username>/rxml.git
   cd rxml
   ```

2. Create and activate a virtual environment:

   ```bash
   python -m venv .venv
   source .venv/bin/activate   # Linux/macOS
   .venv\Scripts\activate      # Windows
   ```

3. Install development dependencies:

   ```bash
   pip install maturin
   ```

4. Build the project in development mode:

   ```bash
   maturin develop
   ```

5. Run the tests:

   ```bash
   cargo test
   ```

## Making Changes

1. Create a new branch for your feature or fix:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes. If you're modifying Rust code, rebuild with `maturin develop` before testing.

3. Ensure all tests pass:

   ```bash
   cargo test
   ```

4. Format your code:

   ```bash
   cargo fmt
   ```

5. Run the Rust linter:

   ```bash
   cargo clippy
   ```

## Submitting a Pull Request

1. Push your branch to your fork.
2. Open a pull request against the `main` branch.
3. Provide a clear description of your changes and the motivation behind them.
4. Ensure CI checks pass.

## Reporting Issues

If you encounter a bug or have a feature request, please [open an issue](https://github.com/nephi-dev/rxml/issues/new) with as much detail as possible, including:

- Your Python and Rust versions
- Your operating system
- A minimal reproducible example (if applicable)

## Code of Conduct

Please be respectful and constructive in all interactions. We are committed to providing a welcoming and inclusive experience for everyone.
