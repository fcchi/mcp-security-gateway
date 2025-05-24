# Contributing to MCP Security Gateway

Thank you for your interest in contributing to the MCP Security Gateway project. This document explains the contribution process.

## Code of Conduct

This project follows a [Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project, you agree to abide by its terms.

## Setting up the Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/mcp-security-gateway.git
   cd mcp-security-gateway
   ```

2. Install dependencies:
   - **Linux (Ubuntu/Debian):**
     ```bash
     sudo apt-get update
     sudo apt-get install -y protobuf-compiler bubblewrap libseccomp-dev
     ```
   - **Fedora/CentOS:**
     ```bash
     sudo dnf install -y protobuf-compiler bubblewrap libseccomp-devel
     ```
   - **macOS:**
     ```bash
     brew install protobuf
     ```
   - **Windows:**
     1. Download the latest Windows release (e.g., `protoc-25.2-win64.zip`) from the [Protocol Buffers release page](https://github.com/protocolbuffers/protobuf/releases)
     2. Extract the downloaded ZIP file to any folder
     3. Add the `bin` directory of the extracted folder (e.g., `C:\protoc\bin`) to the PATH environment variable
     4. Restart Command Prompt or PowerShell
     5. Verify the installation with the `protoc --version` command

3. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Contribution Process

1. Check existing issues or create a new one on [GitHub Issues](https://github.com/your-username/mcp-security-gateway/issues).
2. Fork the repository and clone it locally.
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes and ensure tests pass.
5. Check code style: `cargo fmt -- --check` and `cargo clippy -- -D warnings`
6. Commit your changes and push: `git push origin feature/your-feature-name`
7. Create a pull request.

## Coding Conventions

- Follow Rust standard style (use `cargo fmt`)
- Resolve all warnings (check with `cargo clippy -- -D warnings`)
- Add unit tests for new features (coverage target ≥ 80%)
- Add documentation comments (required for public APIs/functions)

## Commit Message Conventions

Follow this format for commit messages:
```
[type]: Short description (under 50 chars)

More detailed description if needed. Wrap at 72 characters.

Reference related issues: fixes #123
```

Examples of types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes only
- `test`: Test changes only
- `refactor`: Refactoring (no functional changes)
- `style`: Code style changes (whitespace, formatting, etc.)
- `chore`: Changes to build process, etc.

## Pull Request Review Process

1. CI pass: Ensure GitHub Actions pass
2. Review approval: At least one maintainer review approval is required
3. Merge: Will be merged once all requirements are met

## Release Process

Releases follow [Semantic Versioning](https://semver.org/):
- Patch releases (1.0.x): Bug fixes only
- Minor releases (1.x.0): Backward-compatible new features
- Major releases (x.0.0): Breaking changes

## Questions and Help

If you have questions, create an issue or post in the [Discussions](https://github.com/your-username/mcp-security-gateway/discussions) section.

Thank you for your contributions! 