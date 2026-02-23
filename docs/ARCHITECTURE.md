# Architecture guidelines

Technical architecture overview for the media service, a media processing service for uploading, transcoding, and serving images and videos.

## Workspace structure

The project uses a **Cargo workspace** to organize code into separate packages:

```plaintext
packages/
├── api/                # HTTP server (axum routes, app setup)
│   └── src/
├── foundation/         # Core business logic (library crate)
│   └── src/
│       ├── common/     # Cross-cutting concerns (tracing)
│       ├── config.rs   # Configuration (env-based, per-package)
│       ├── feature/    # Vertical feature slices
│       └── shared/     # Shared infrastructure
└── processor/          # Async worker for transcoding jobs
    └── src/
```

### Package dependencies

- **foundation**: Core library with no binary dependencies. All domain logic and infrastructure abstractions live here.
- **api**: Depends on `foundation`. Thin HTTP entry point using Axum.
- **processor**: Depends on `foundation`. Thin worker entry point consuming jobs from a queue.

```
┌─────────────┐
│     api     │
└──────┬──────┘
       │ depends on
       ▼
┌─────────────┐
│ foundation  │
└─────────────┘
       ▲
       │ depends on
┌──────┴──────┐
│  processor  │
└─────────────┘
```

Both `api` and `processor` are thin entry points. They compose dependencies defined in `foundation` and wire them together in `container.rs`.

## Architectural style

### Vertical slice architecture

Within the `foundation` package, the system is organized around business capabilities:

```plaintext
packages/foundation/src/
├── common/              # Cross-cutting concerns (tracing)
├── config.rs            # Configuration (env-based, per-package)
├── shared/              # Shared infrastructure (storage, cdn, queue, database)
└── feature/             # Production feature slices
    ├── image/           # Image media type
    │   ├── upload/      # Presigned URL generation and upload validation
    │   └── download/    # Image access via signed URLs
    ├── short_video/     # Short-form video (future)
    └── long_video/      # Long-form video (future)
```

### Feature slice structure

Each feature slice encapsulates domain logic and infrastructure. HTTP handlers live in the `api` package.

Each feature slice is organized into layers:

- **Domain**: Commands, queries, domain models, and trait definitions for infrastructure
- **Infrastructure**: Implementations of domain traits (for example, repositories, external service clients)

```plaintext
packages/foundation/src/feature/{media_type}/{feature}/
├── mod.rs              # pub mod declarations only
├── command.rs          # Domain layer: command DTOs and executors
├── query.rs            # Domain layer: query DTOs and executors
├── model.rs            # Domain layer: domain models and infrastructure traits
├── error.rs            # Feature-specific error types
└── repository.rs       # Infrastructure layer: data access
```

HTTP routes are separate in the `api` package:

```plaintext
packages/api/src/
├── main.rs             # Entry point, starts the server
├── container.rs        # Dependency wiring and AppState construction
└── routes/
    ├── mod.rs
    ├── image/
    │   ├── mod.rs
    │   ├── upload.rs   # Image upload endpoints
    │   └── download.rs # Image download endpoints
    └── health.rs       # Health check endpoints
```

The structure above is simplified. As complexity grows, each layer can be further organized into submodules.

#### API

HTTP handlers or event consumers that orchestrate command and query executors.

Request flow:

1. Request arrives at the API layer (HTTP handler or event consumer)
2. API layer translates the request into a **command** or **query** struct
3. The corresponding **executor** processes the operation
4. Result is returned to the caller

**Keep handlers thin.** Handlers should only orchestrate: parse requests, call executors, format responses. Don't inline complex logic (filtering, sorting, data transformation) inside a handler. If the logic can be expressed as a method on a domain model or value object, put it there instead.

#### Domain

The system uses the CQRS pattern to separate write and read operations:

- **Command + Executor**: Encapsulates a write operation and its executor
- **Query + Executor**: Encapsulates a read operation and its executor

For simpler features, keep commands and queries in single files (`command.rs` and `query.rs`). When complexity grows, organize them into submodules.

#### Infrastructure

Implementations of domain traits (for example, repositories, external service clients).

When the same pattern or utility appears across multiple features, extract it into `common/` or `shared/`.

### Dependency rules

- **Inward dependencies**: Features depend on `common/` and `shared/` within foundation
- **Feature isolation**: Features don't directly reference other features
- **Shared code**: Common utilities are in `shared/` or `common/`
- **Package dependencies**: `api` and `processor` depend on `foundation`, not on each other

## Dependency injection

Rust doesn't need a DI framework. The type system enforces dependencies at compile time through traits and constructor injection. This gives you the same benefits (testability, loose coupling) without runtime overhead.

### Traits as interfaces

Define infrastructure boundaries as traits in the domain layer. Implementations live in the infrastructure layer.

```rust
// Domain layer: packages/foundation/src/feature/upload/model.rs
#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn find(&self, id: MediaId) -> Result<Option<Media>, RepositoryError>;
    async fn save(&self, media: &Media) -> Result<(), RepositoryError>;
}

// Infrastructure layer: packages/foundation/src/feature/upload/repository.rs
pub struct PostgresMediaRepository {
    pool: PgPool,
}

impl MediaRepository for PostgresMediaRepository {
    // ...
}
```

### Constructor injection

Executors receive their dependencies as trait objects through their constructor. Use `Arc<dyn Trait>` for shared ownership across async tasks.

```rust
// packages/foundation/src/feature/upload/command.rs
pub struct CreateUploadUrlCommandExecutor {
    media_repository: Arc<dyn MediaRepository>,
    storage: Arc<dyn MediaStorage>,
}

impl CreateUploadUrlCommandExecutor {
    pub fn new(
        media_repository: Arc<dyn MediaRepository>,
        storage: Arc<dyn MediaStorage>,
    ) -> Self {
        Self { media_repository, storage }
    }

    pub async fn execute(&self, command: CreateUploadUrlCommand) -> Result<UploadUrl, UploadError> {
        // ...
    }
}
```

### Container module

Each binary crate (`api`, `processor`) has a `container.rs` that composes the dependency graph. This is the only place where concrete types are chosen. The `main.rs` calls the container to build the app state and starts the server.

```rust
// packages/api/src/container.rs
#[derive(Clone)]
pub struct AppState {
    pub create_upload_url: Arc<CreateUploadUrlCommandExecutor>,
    pub get_media: Arc<GetMediaQueryExecutor>,
    // ...
}

impl AppState {
    pub async fn build(config: &Config) -> Self {
        let pool = PgPool::connect(&config.database.connection_url()).await.unwrap();

        // Wire infrastructure
        let media_repository = Arc::new(PostgresMediaRepository::new(pool.clone()));
        let storage = Arc::new(S3Storage::new(config));

        // Wire executors
        let create_upload_url = Arc::new(CreateUploadUrlCommandExecutor::new(
            media_repository.clone(),
            storage.clone(),
        ));

        Self { create_upload_url, /* ... */ }
    }
}
```

```rust
// packages/api/src/main.rs
mod container;
mod routes;

#[tokio::main]
async fn main() {
    let config = Config::load();
    let state = container::AppState::build(&config).await;

    let app = Router::new()
        .route("/api/v1/upload-url", post(routes::upload::create))
        .with_state(state);

    // ...
}
```

Handlers extract state via axum's `State` extractor:

```rust
// packages/api/src/routes/upload.rs
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateUploadUrlRequest>,
) -> Result<Json<CreateUploadUrlResponse>, ApiError> {
    let command = CreateUploadUrlCommand {
        content_type: payload.content_type,
        size_bytes: payload.size_bytes,
    };
    let result = state.create_upload_url.execute(command).await?;
    Ok(Json(result.into()))
}
```

### Testing with trait objects

In tests, provide mock or stub implementations of the same traits:

```rust
#[cfg(test)]
mod tests {
    struct StubMediaRepository {
        result: Option<Media>,
    }

    impl MediaRepository for StubMediaRepository {
        async fn find(&self, id: MediaId) -> Result<Option<Media>, RepositoryError> {
            Ok(self.result.clone())
        }
        // ...
    }
}
```

Use hand-written stubs or mocks. Don't use libraries that require modifying production code with test-only annotations.
