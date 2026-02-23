# Development guidelines

## Project structure

The project is organized as a Cargo workspace with packages in `packages/`:

```plaintext
packages/
├── api/                # HTTP server (axum routes, app setup)
│   └── src/
├── foundation/         # Core business logic (library crate)
│   └── src/
│       ├── common/     # Cross-cutting concerns (config, tracing)
│       ├── feature/    # Vertical feature slices
│       └── shared/     # Shared infrastructure
└── processor/          # Async worker for transcoding jobs
    └── src/
```

Both `api` and `processor` depend on `foundation`. Feature code and domain logic live in `foundation`; the binary crates are thin entry points.

Other top-level directories:

```plaintext
docs/                   # Development docs, proposals, and templates
```

## Cargo workspace

Common commands:

```bash
# Build all packages
cargo build

# Run linting and formatting checks
task check

# Run tests
task test

# Run api and processor concurrently
task run

# Run a single package
task run:api
task run:processor
```

### Adding dependencies

Dependencies are split between the workspace root and individual packages based on scope.

**Workspace-level dependencies** (root `Cargo.toml` under `[workspace.dependencies]`):

Only declare dependencies here when they are shared across two or more packages. Each package references them with `.workspace = true`. This centralizes version management for common crates and ensures consistency across the workspace.

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

```toml
# packages/api/Cargo.toml
[dependencies]
tokio.workspace = true
serde.workspace = true
```

**Package-level dependencies** (each package's `Cargo.toml`):

Dependencies used by only one package are declared directly in that package's `Cargo.toml` with an explicit version. This keeps the workspace root lean and makes each package's unique requirements visible at a glance.

```toml
# packages/api/Cargo.toml — axum is only used by the api package
[dependencies]
axum = "0.8"
```

**Dependency consistency:**

Prefer using the same crate for similar functionality across the codebase. Before adding a new dependency, check what existing packages use.

| Purpose             | Preferred Crate                  |
| ------------------- | -------------------------------- |
| Async runtime       | `tokio`                          |
| HTTP framework      | `axum`                           |
| Serialization       | `serde` + `serde_json`           |
| Database (Postgres) | `sqlx`                           |
| AWS SDK             | `aws-sdk-*`                      |
| Error handling      | `thiserror` and `anyhow`         |
| HTTP client         | `reqwest`                        |
| Observability       | `tracing` + `tracing-subscriber` |

## Documentation

**Default: No doc comments.** Code should be self-documenting through:

- Clear, descriptive function/variable names
- Small, single-purpose functions
- Type signatures for all parameters and return values

**Exceptions (doc comments allowed):**

1. **Public API types and endpoints** — Brief description of what it does
2. **Complex algorithms** — When the "why" isn't obvious from the code
3. **Non-obvious side effects** — External calls, panics, error conditions
4. **Domain-specific terminology** — When business context is needed

**When writing doc comments:**

- One-line summary only (no `# Arguments` / `# Returns` sections — use type signatures)
- Explain "why", not "what"
- Keep it under 120 chars when possible

## Lint

**Don't modify Clippy lint rules in `Cargo.toml` or create a `clippy.toml`.** When encountering lint warnings:

1. Fix the code to comply with existing rules
2. Use `#[allow(clippy::xxx)]` attributes for legitimate exceptions with a comment explaining why
3. Refactor code to reduce complexity

Common inline suppressions:

- `#[allow(clippy::too_many_arguments)]` — Justified in trait definitions or builders
- `#[allow(dead_code)]` — Only during active development, remove before merging
- `#[allow(clippy::module_name_repetitions)]` — When the repetition aids clarity

## Production code hygiene

**Don't modify production code only for testing.** Production code must not contain test-only annotations, attributes, or conditional compilation flags. If a testing library requires adding attributes (for example, `#[automock]`) to production traits or structs, don't use that library. Use hand-written mocks instead.

## Design principles

These principles guide how you structure types and handle errors in the codebase.

### Struct definition

**Group related fields logically.** Use composition over large flat structs. Always derive common traits at the top.

```rust
// Good: well-organized, domain types, common derives
#[derive(Debug, Clone)]
struct TranscodeJob {
    id: TranscodeJobId,
    media_id: MediaId,
    status: TranscodeStatus,
    profile: TranscodeProfile,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

// Bad: primitives everywhere, no structure
struct TranscodeJob {
    id: i64,
    media_id: i64,
    status: String,
    width: u32,
    height: u32,
    bitrate: u32,
    created_at: String,
}
```

### Newtype pattern for domain IDs

**Wrap primitive IDs in newtypes.** This prevents accidentally mixing different ID types and makes function signatures self-documenting.

```rust
// Good: distinct types prevent mix-ups
struct MediaId(i64);
struct TranscodeJobId(i64);

fn get_media(id: MediaId) -> Result<Media, Error> { ... }

// Bad: raw primitives allow silent bugs
fn get_media(id: i64) -> Result<Media, Error> { ... }
```

### Error handling

**Use typed errors, not string errors.** Define domain-specific error types with `thiserror`.

```rust
// Good: typed errors with context
#[derive(Debug, thiserror::Error)]
enum UploadError {
    #[error("media not found: {0}")]
    NotFound(MediaId),

    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),

    #[error("file size exceeds limit: {size} > {limit}")]
    FileTooLarge { size: u64, limit: u64 },
}

// Bad: string errors
fn upload(file: File) -> Result<(), String> { ... }
```

### Avoid sentinel values

Don't make a field `Option` solely to represent a different state. Handle distinct states explicitly with enums.

```rust
// Good: explicit state modeling
enum MediaAccess {
    SignedUrl { url: String, expires_at: DateTime<Utc> },
    SignedCookie { cookie: String, domain: String, expires_at: DateTime<Utc> },
}

// Bad: nullable fields as sentinels
struct MediaAccess {
    url: Option<String>,    // None means "use cookie instead"
    cookie: Option<String>, // None means "use url instead"
}
```

## Code organization

These conventions keep the codebase navigable as it grows.

### Module organization

**Keep `mod.rs` minimal.** Use it only for `pub mod` declarations and no re-exports. Don't put implementation logic in `mod.rs`.

```rust
// Good: mod.rs only declares submodules
// src/feature/upload/mod.rs
pub mod command;
pub mod handler;

// Import from the source module
use crate::feature::upload::command::CreateUploadUrlCommand;
```

### Naming conventions

Follow these naming patterns for CQRS and event-driven components.

#### CQRS pattern

Use verb-first naming for commands and queries, with matching executor types.

##### Command components

- **Command DTO**: Start with a verb, end with `Command`, for example, `CreateUploadUrlCommand`, `CompleteUploadCommand`
- **Command executor**: Same name with `Executor` suffix, for example, `CreateUploadUrlCommandExecutor`

##### Query components

- **Query DTO**: Start with a verb, end with `Query`, for example, `GetMediaQuery`, `ListMediaQuery`
- **Query executor**: Same name with `Executor` suffix, for example, `GetMediaQueryExecutor`

#### Event-driven pattern

Use noun-first, past-tense naming for events with matching handler types.

- **Event DTO**: Start with a noun, use past-tense verb, end with `Event`, for example, `MediaUploadedEvent`, `TranscodeCompletedEvent`
- **Event handler**: Same name with `Handler` suffix, for example, `MediaUploadedEventHandler`

### API path conventions

Each media type owns its own API surface under a dedicated path prefix. The path structure separates the media type, version, and resource:

```
/api/{media-type}/v1/{resource}/{action}
```

- `{media-type}`: Singular form of the media type (for example, `image`, `short-video`, `long-video`).
- `v1`: API version, scoped per media type. Each media type can version independently.
- `{resource}`: The domain resource or capability (for example, `upload`, `download`, `images`).
- `{action}`: The specific operation (for example, `create-presigned-url`, `get-signed-url`).

For example:

```
POST /api/image/v1/upload/create-presigned-url
GET  /api/image/v1/download/get-signed-url/{image_id}
POST /api/image/v1/download/get-signed-urls
```

Infrastructure endpoints like health checks live at the root level without versioning:

```
GET /api/health/liveness
```

## Database migrations

**For local development and testing only.** These aren't automatic migrations — apply manually to local databases.

### File naming

Use the following pattern for migration file names:

```
{sequence}_{feature_or_module}.sql
```

- `{sequence}`: 4-digit number (for example, `0001`, `0002`)
- `{feature_or_module}`: feature name or shared module name

For example:

```
0001_media.sql         # media entity tables
0002_transcode.sql     # transcode job tables
```
