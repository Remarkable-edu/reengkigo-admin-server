# Development Guide

## Project Structure

```
src/
├── main.rs              # Application entry point
├── config/              # Configuration management
│   └── mod.rs
├── dto/                 # Data Transfer Objects
│   ├── mod.rs
│   └── asset.rs         # Asset-related DTOs
├── handlers/            # HTTP request handlers
│   ├── mod.rs
│   └── admin.rs         # Asset management endpoints
├── models/              # Database models
│   ├── mod.rs
│   └── asset.rs         # Asset data models
├── services/            # Business logic layer
│   ├── mod.rs
│   ├── database.rs      # Database connection
│   └── asset.rs         # Asset service operations
└── utils/               # Utility modules
    ├── mod.rs
    └── logging.rs       # Logging configuration
```

## Getting Started

### Prerequisites

- Rust 1.70+ 
- MongoDB 4.4+

### Environment Setup

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Configure your environment variables:
```bash
# MongoDB connection
MONGO_URI=mongodb://localhost:27017

# Application configuration
APP_DATABASE_URL=mongodb://localhost:27017
APP_DATABASE_NAME=reengkigo_db
APP_SERVER_HOST=127.0.0.1
APP_SERVER_PORT=3000
APP_APP_DEBUG=true
```

### Running the Application

```bash
# Development mode with auto-reload
cargo run

# Release mode
cargo build --release
./target/release/reengkigo-admin-app
```

### Running Examples

```bash
# Run the CRUD example
MONGO_URI=mongodb://localhost:27017 cargo run --example db_crud_sampling
```

## Development Workflow

### Code Organization

- **Handlers**: HTTP request/response handling, input validation, error handling
- **Services**: Business logic, database operations, data transformation
- **Models**: Database schema definitions and data structures
- **DTOs**: API request/response structures
- **Config**: Application configuration management

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_asset_creation
```

### Code Quality

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy

# Check compilation without building
cargo check
```

## Database Schema

### Assets Collection

```json
{
  "_id": ObjectId,
  "book": [
    {
      "book_id": "string",
      "month": "string", 
      "cover_img": "string",
      "video_content": [
        {
          "video_img": "string",
          "youtube_url": "string"
        }
      ]
    }
  ]
}
```

## Configuration

The application uses a hierarchical configuration system:

1. **Default values** (in code)
2. **config.yaml** file (optional)
3. **Environment variables** (prefixed with `APP_`)

Example `config.yaml`:
```yaml
app:
  name: "reengkigo"
  version: "1.0.0"
  debug: true

database:
  url: "mongodb://localhost:27017"
  name: "reengkigo_db"

server:
  host: "0.0.0.0"
  port: 3000
```

## Adding New Features

### 1. Add New Model

Create a new model in `src/models/`:

```rust
// src/models/user.rs
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
}
```

### 2. Add DTOs

Create corresponding DTOs in `src/dto/`:

```rust
// src/dto/user.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
}
```

### 3. Add Service

Create business logic in `src/services/`:

```rust
// src/services/user.rs
use anyhow::Result;
use mongodb::Collection;
use crate::{models::user::User, dto::user::*};

pub struct UserService;

impl UserService {
    pub async fn create_user(db: &Database, request: CreateUserRequest) -> Result<UserResponse> {
        // Implementation
    }
}
```

### 4. Add Handler

Create HTTP handlers in `src/handlers/`:

```rust
// src/handlers/user.rs
use axum::{extract::State, response::Json, http::StatusCode};
use crate::{AppState, dto::user::*, services::user::UserService};

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Implementation
}
```

### 5. Add Routes

Update `src/main.rs` to include new routes:

```rust
fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/api/assets", post(admin::create_asset))
        .route("/api/users", post(user::create_user)); // New route

    Router::new()
        .merge(api_routes)
        .with_state(state)
}
```

## Troubleshooting

### Common Issues

1. **MongoDB Connection Failed**
   - Check that MongoDB is running
   - Verify MONGO_URI environment variable
   - Ensure database permissions are correct

2. **Compilation Errors**
   - Run `cargo clean` and rebuild
   - Check for missing dependencies in Cargo.toml
   - Verify Rust version compatibility

3. **Test Failures**
   - Ensure test database is available
   - Check test data setup
   - Verify async runtime configuration

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

For more detailed logging:
```bash
RUST_LOG=server_test=trace,mongodb=debug cargo run
```