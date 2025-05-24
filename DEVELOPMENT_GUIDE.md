# MCP Security Gateway Development Guide

This document provides guidelines for development work on the MCP Security Gateway project.

## Table of Contents

1. [Git Workflow](#git-workflow)
2. [Branch Strategy](#branch-strategy)
3. [Commit Message Guidelines](#commit-message-guidelines)
4. [Pull Request Process](#pull-request-process)
5. [Release Process](#release-process)
6. [Code Quality](#code-quality)

## Git Workflow

In this project, we use a GitFlow-like workflow.

### Initial Setup

To clone the remote repository:

```bash
# Clone the repository
git clone https://github.com/username/mcp-security-gateway.git
cd mcp-security-gateway

# Install dependencies
cargo build
```

To add a remote repository:

```bash
# Add a remote to an existing local repository
git remote add origin https://github.com/username/mcp-security-gateway.git

# Pull changes from the remote
git pull origin main
```

## Branch Strategy

We use the following branches:

- **main**: Stable code for production environment
- **develop**: Development code for the next release
- **feature/X**: Development of individual features (`feature/milestone7-beta-release` etc.)
- **bugfix/X**: Bug fixes
- **release/X.Y.Z**: Release preparation (`release/0.2.0` etc.)
- **hotfix/X**: Emergency fixes

### For New Feature Development

```bash
# Get the latest changes from develop
git checkout develop
git pull origin develop

# Create a feature branch
git checkout -b feature/new-feature-name

# (Development work)

# Commit changes
git add .
git commit -m "Feature addition: 〇〇 feature implementation"

# Before merging into develop, get the latest changes
git checkout develop
git pull origin develop
git checkout feature/new-feature-name
git rebase develop

# Merge into develop
git checkout develop
git merge --no-ff feature/new-feature-name
git push origin develop

# Delete feature branch (optional)
git branch -d feature/new-feature-name
```

## Commit Message Guidelines

Commit messages should follow this format:

```
[Type]: Summary (50 characters or less)

Detailed description (optional, 72 characters per line)
```

Example types:
- **Feature**: New feature
- **Fix**: Bug fix
- **Improvement**: Improvement of existing feature
- **Refactor**: Code change without feature change
- **Documentation**: Only change in documentation
- **Test**: Addition or correction of test
- **Config**: Change in CI/CD, build configuration

Example:
```
Feature: OPA policy engine integration

- Added dynamic loading of Rego policy
- Implemented policy evaluation caching mechanism
- Improved error handling
```

## Pull Request Process

1. Complete development in a feature branch
2. Run tests and check code style
   ```bash
   cargo test
   cargo fmt --all -- --check
   cargo clippy
   ```
3. Create a PR explaining the changes
4. Receive code review
5. Ensure CI pipeline succeeds
6. Get merge approval

## Release Process

1. Create a release branch from develop
   ```bash
   git checkout develop
   git checkout -b release/0.2.0
   ```

2. Update version number
   - `Cargo.toml` version
   - CHANGELOG update

3. Final testing and QA

4. Merge into main
   ```bash
   git checkout main
   git merge --no-ff release/0.2.0
   git tag -a v0.2.0 -m "MCP Security Gateway v0.2.0"
   git push origin main --tags
   ```

5. Merge into develop
   ```bash
   git checkout develop
   git merge --no-ff release/0.2.0
   git push origin develop
   ```

## Code Quality

- Run the following before committing
  ```bash
  cargo test
  cargo fmt
  cargo clippy
  ```

- Check code coverage
  ```bash
  cargo tarpaulin
  ```

- Security check
  ```bash
  cargo audit
  ```

## CI/CD

GitHub Actions workflow automates the following:

- Compile test
- Unit test execution
- Code style check
- Static analysis
- Security scan
- Code coverage report
- Document generation 