# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust web server built with Axum framework that provides asset management functionality with MongoDB integration. The application is named "reengkigo" and follows a clean architecture pattern with clear separation of concerns.

## Development Commands

### Building and Running
```bash
# Run the application in development mode
cargo run

# Build for release
cargo cargo build --release

# Run with environment variables
RUST_LOG=debug cargo run

# Run the example CRUD operations
cargo run --bin db_crud_sampling --example db_crud_sampling
```

### Testing and Quality
```bash
# Run tests
cargo test

# Check for compilation errors
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## Architecture

### Core Components

- **main.rs**: Entry point with server initialization, graceful shutdown handling, and router setup
- **config/**: Configuration management using Figment with YAML and environment variable support
- **handlers/**: HTTP request handlers (currently contains admin functionality)
- **services/**: Business logic layer containing database operations and asset management
- **models/**: Data models and structures
- **dto/**: Data Transfer Objects for API communication
- **utils/**: Utility modules including logging and observability

### Key Features

1. **Configuration Management**: Uses Figment for hierarchical configuration (defaults → config.yaml → environment variables)
2. **Database**: MongoDB integration with async operations using the mongodb crate
3. **Observability**: Structured logging with tracing and observability management
4. **Graceful Shutdown**: Handles SIGTERM and Ctrl+C signals for clean shutdown
5. **Asset Management**: CRUD operations for assets containing books and video content

### Application State

The `AppState` struct contains:
- Database connection (`Database`)
- Application configuration (`Arc<AppConfig>`)
- Observability manager (`Arc<ObservabilityManager>`)

### Configuration Structure

Configuration is loaded from:
1. Default values in code
2. `config.yaml` file (if present)
3. Environment variables prefixed with `APP_`

Key configuration sections:
- `app`: Application metadata (name, version, debug mode)
- `database`: MongoDB connection (url, database name)
- `server`: Server settings (host, port)

## Environment Setup

Required environment variables:
- `MONGO_URI`: MongoDB connection string (for examples)
- `APP_*`: Any configuration overrides using the APP_ prefix

The application uses `.env` file loading via dotenv for local development.