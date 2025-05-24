# MCP Security Gateway

MCP Security Gateway is a security adapter for enabling secure communication between AI/ML models and external systems. It provides sandboxed command execution, file access control, and security policy enforcement.

## Key Features

- **Secure Command Execution**: Sandboxing using bubblewrap, seccomp
- **Policy-based Control**: Fine-grained security policies using OPA (Rego)
- **Audit and Tracing**: Complete audit logs and observability
- **High Availability**: Supports Active-Active / Active-Passive topology
- **Multiple Interfaces**: REST API / gRPC / Client Libraries

## Quick Start

### Requirements

- Docker and Docker Compose
- Rust (for development only, version 1.73 or higher recommended)
- Protocol Buffers Compiler (protoc)

### Installing Protocol Buffers Compiler

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler
```

**macOS:**
```bash
brew install protobuf
```

**Windows:**
1. Download the latest Windows release (e.g., `protoc-25.2-win64.zip`) from the [Protocol Buffers release page](https://github.com/protocolbuffers/protobuf/releases)
2. Extract the downloaded ZIP file to any folder
3. Add the `bin` directory of the extracted folder (e.g., `C:\protoc\bin`) to the PATH environment variable
4. Restart Command Prompt or PowerShell
5. Verify the installation with the `protoc --version` command

**Verification:**
You can verify the installation with the following command:
```bash
protoc --version
```

### Quick Start with Docker

You can clone the repository and start with Docker Compose:

```bash
# Clone the repository
git clone https://github.com/fcchi/mcp-security-gateway.git
cd mcp-security-gateway

# Start with Docker Compose
docker-compose up -d
```

Once the service is running, you can check its health at:

```
http://localhost:8081/health
```

### Using Secure Distroless Images

For production environments, we recommend using distroless images for enhanced security:

```bash
# Docker build with distroless image
./scripts/build-distroless.sh

# Or run directly
docker run -d --name mcp-gateway \
  --cap-add SYS_ADMIN \
  --security-opt seccomp=unconfined \
  -p 8081:8081 \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/policies:/app/policies \
  -v $(pwd)/workspace:/workspace \
  ghcr.io/fcchi/mcp-security-gateway:latest
```

Distroless images significantly reduce security risks by minimizing the attack surface and excluding shells and debugging tools. For more details, refer to the [Deployment Guide](docs/operations/DEPLOY_GUIDE.md#3.2-using-distroless-images).

### Setting Up Local Development Environment

```bash
# Clone the repository
git clone https://github.com/fcchi/mcp-security-gateway.git
cd mcp-security-gateway

# Install dependencies
# Linux (Ubuntu/Debian):
sudo apt-get install -y protobuf-compiler bubblewrap libseccomp-dev

# macOS:
brew install protobuf

# Windows:
# Follow the Protocol Buffers compiler installation instructions

# Build
cargo build

# Run unit tests
cargo test

# Run
cargo run -- serve --host 127.0.0.1 --port 8081
```

### Basic Usage

#### Command Execution API Example

```bash
# Example using gRPCurl (requires installation)
grpcurl -plaintext -d '{
  "command": "echo",
  "args": ["hello world"],
  "timeout": 30
}' localhost:8081 mcp.McpService/ExecuteCommand

# Check task status
grpcurl -plaintext -d '{"task_id": "task-xxxxx"}' localhost:8081 mcp.McpService/GetTaskStatus
```

## Documentation

### Architecture

- [Architecture Overview](docs/architecture/ARCHITECTURE_OVERVIEW.md) - Overall system architecture and key points
- [User Guide](docs/architecture/USER_GUIDE.md) - Basic usage and operations

### API

- [MCP Protocol Specification](docs/api/MCP_PROTOCOL.md) - Protocol details
- [API Reference](docs/api/API_REFERENCE.md) - API endpoints and features

### Security

- [Security Features](docs/security/SECURITY_FEATURES.md) - Security model and features
- [Threat Model](docs/security/THREAT_MODEL.md) - Security risks and countermeasures

### Operations

- [Deployment Guide](docs/operations/DEPLOY_GUIDE.md) - Installation and configuration
- [Operations Runbook](docs/operations/OPERATIONS_RUNBOOK.md) - Operations and maintenance
- [Performance SLO](docs/operations/PERFORMANCE_SLO.md) - Service level objectives

### Quality

- [Error Handling](docs/quality/ERROR_HANDLING.md) - Error classification and handling
- [Test Strategy](docs/quality/TEST_STRATEGY.md) - Testing methods and coverage

### Observability

- [Observability](docs/observability/OBSERVABILITY.md) - Logs, metrics, and traces

### Meta Information

- [Glossary](docs/meta/GLOSSARY.md) - Key terms and definitions
- [Changelog](docs/meta/CHANGELOG.md) - Version change history
- [Versioning and Upgrade](docs/meta/VERSIONING_UPGRADE.md) - Version management and upgrade procedures
- [Implementation Roadmap](docs/meta/ROADMAP.md) - Implementation plan and milestones

## Cross Reference

| Category | Related Documents | Related Metrics/Alerts |
|---------|-------------------|---------------------|
| Error Codes | [MCP_PROTOCOL.md](docs/api/MCP_PROTOCOL.md), [ERROR_HANDLING.md](docs/quality/ERROR_HANDLING.md) | MCPHighErrorRate |
| Performance | [PERFORMANCE_SLO.md](docs/operations/PERFORMANCE_SLO.md), [TEST_STRATEGY.md](docs/quality/TEST_STRATEGY.md) | MCPLatencyP99, MCPThroughputAPI |
| Security | [SECURITY_FEATURES.md](docs/security/SECURITY_FEATURES.md), [DEPLOY_GUIDE.md](docs/operations/DEPLOY_GUIDE.md) | MCPHighPolicyViolationRate |
| Availability | [ARCHITECTURE_OVERVIEW.md](docs/architecture/ARCHITECTURE_OVERVIEW.md), [OPERATIONS_RUNBOOK.md](docs/operations/OPERATIONS_RUNBOOK.md) | MCPAvailabilityProd |

## Contributing

If you'd like to contribute, please refer to [CONTRIBUTING.md](CONTRIBUTING.md). All contributors are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

## Development Status

Current development status:

- ✅ Milestone 0 (Project Template): Completed basic Rust workspace configuration
  - ✅ Repository structure and workspace setup
  - ✅ Basic gRPC server implementation
  - ✅ GitHub Actions CI setup
  - ✅ Code quality gates (rustfmt & clippy)
  - ✅ CONTRIBUTING & ISSUE_TEMPLATE
  - ✅ Review and final adjustments
- ✅ Milestone 1 (Core MVP α): Completed basic functionality implementation
  - ✅ Basic gRPC server implementation
  - ✅ Task execution protocol definition and stub implementation
  - ✅ Command execution adapter
  - ✅ Structured log output
  - ✅ Unit tests
  - ✅ Quick start configuration
- ✅ Milestone 2 (Policy & Security): Completed
  - ✅ Basic implementation of OPA integration (Rego policies)
  - ✅ Implementation of bubblewrap sandbox profiles
  - ✅ Mapping of error codes and gRPC status
  - ✅ Implementation of common Result<T, McpError> helper
  - ✅ Update of security documentation
  - ✅ Implementation of global error handler
  - ✅ Review and final adjustments
- ✅ Milestone 3 (Observability Stack): Completed
  - ✅ Exposed as Prometheus histograms (task_latency_ms)
  - ✅ Added OTLP trace exporter
  - ✅ Verification of ls flow using e2e scripts (pexpect)
  - ✅ Grafana dashboard setup (overview, performance)
  - ✅ mkdocs build and link check for documentation (CI)
  - ✅ Final review and adjustments for Milestone 3
- ✅ Milestone 4 (CI Gate & Error Strategy): Completed
  - ✅ Added Trivy & cargo-audit SCA step (T040)
  - ✅ Added coverage gate (T041)
  - ✅ McpError ↔ gRPC status mapping matrix (T042)
  - ✅ ERROR_HANDLING link from API_REFERENCE (T043)
  - ✅ action: Release Drafter configuration (T044)
  - ✅ Review & bug fixes (M4) (T045)
- ✅ Milestone 5 (Packaging & Deployment): Completed
  - ✅ create distroless Dockerfile + ko build (T050)
  - ✅ build .deb & .rpm via fpm (T051)
  - ✅ create chart mcp-gateway (T052)
  - ✅ DEPLOY_GUIDE update (helm + deb) (T053)
  - ✅ action: MkDocs-deploy (GitHub Pages) (T054)
  - ✅ Review & bug fixes (M5) (T055)
- ✅ Milestone 6 (Performance & SLO Validation): Completed
  - ✅ locust script 100 RPS × 5 min baseline (T060)
  - ✅ Prometheus alert rule SLO_violation (T061)
  - ✅ PERFORMANCE_SLO link to alert rule (T062)
  - ✅ action: nightly dependency-update w/ Renovate (T063)
  - ✅ Review & bug fixes (M6) (T064)

For detailed progress, refer to the [Implementation Roadmap](docs/meta/ROADMAP.md) and [Changelog](docs/meta/CHANGELOG.md).

## Next Steps

Priority tasks for the future:

1. Start implementation of Milestone 7 (Release β & Docs Polish)
   - bump to v0.2.0 + CHANGELOG entry (T070)
   - README badges (CI, Coverage, Go-Report) (T071)
   - USER_GUIDE (formerly ARCH_OVERVIEW.md) final polish (T072)
   - GitHub Release draft w/ binaries & checksums (T073)

## Recent Updates

- **2025-05-24**: Completed translation of all Japanese comments to English in the codebase (excluding docs directory).

## License

Apache License 2.0 - See the [LICENSE](LICENSE) file for details. 