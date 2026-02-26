# Image management tasks

This document breaks the image management feature into implementation tasks organized across five phases: foundation setup, upload, serve, post-upload validation, and health check middleware.

## Phase 1: Foundation setup and shared infrastructure

This phase establishes the workspace dependencies, configuration loading, database connectivity, shared infrastructure clients, and image domain types that all features depend on.

### Task 1.1: Add workspace dependencies to `Cargo.toml` files

Add all required crates to the workspace root `Cargo.toml` under `[workspace.dependencies]` and reference them in each package's `Cargo.toml`. This includes `tokio`, `axum`, `serde`, `sqlx`, `aws-sdk-s3`, `aws-sdk-sqs`, `aws-config`, `cloudfront_sign`, `async-trait`, `thiserror`, `tracing`, `tracing-subscriber`, `chrono`, `ulid`, `tower-http`, `regex`, and `rstest` (dev).

- Add workspace-level dependency declarations in the root `Cargo.toml`.
- Reference workspace dependencies in `packages/foundation/Cargo.toml`, `packages/api/Cargo.toml`, and `packages/processor/Cargo.toml`.
- Note: Don't include `aws-sdk-cloudfront`. The `cloudfront_sign` crate handles URL signing. The `aws-sdk-cloudfront` crate is a management API client without signing support.
- Note: The workspace uses Rust edition 2024, which supports async functions in traits natively. Verify during this task whether `async fn` in `dyn`-dispatched traits (for example, `Arc<dyn ImageRepository>`) works without the `async-trait` crate. If native support works, remove `async-trait` from the dependency list. If not, keep it.
- Verify the project compiles with `cargo build`.

The task is complete when the following criteria are met.

- All packages compile without errors.
- Dependencies are declared at the workspace level and referenced with `.workspace = true`.

### Task 1.2: Implement configuration loading in foundation common

Create `packages/foundation/src/common/config.rs` to load configuration from environment variables. Define a `Config` struct with fields for database URL, AWS region, S3 bucket, SQS queue URL, SQS DLQ URL, CloudFront domain, CloudFront key pair ID, CloudFront private key PEM, and expiry durations with defaults.

- Create the `Config` struct with all fields listed in the PRD configuration section.
- Implement a `Config::load()` method that reads from environment variables.
- Use default values for expiry durations (300s for upload URLs, 600s for signed URLs) and max upload size (10MB).
- The max upload size config is used by the post-upload validation processor, not by the API upload endpoint.
- Export from `packages/foundation/src/common/mod.rs`.

The task is complete when the following criteria are met.

- `Config::load()` reads all required environment variables.
- Missing required variables (`DATABASE_URL`, `AWS_REGION`, etc.) cause a clear error message.
- Optional variables with defaults are handled correctly.

### Task 1.3: Set up tracing initialization

Create `packages/foundation/src/common/tracing.rs` with a function to initialize a tracing subscriber. Both `api` and `processor` will call this function during startup.

- Initialize `tracing_subscriber` with a JSON or pretty format based on an environment flag.
- Set default log level from an environment variable (default: `info`).
- Export from `packages/foundation/src/common/mod.rs`.

The task is complete when the following criteria are met.

- Tracing is initialized with a configurable log level.
- Both api and processor can call the init function.

### Task 1.4: Set up database connection pool in foundation shared

Create `packages/foundation/src/shared/database.rs` to establish a SQLx PostgreSQL connection pool from the config.

- Create a function `create_pool(database_url: &str) -> PgPool` that initializes a connection pool.
- Configure pool settings (max connections, idle timeout).
- Export from `packages/foundation/src/shared/mod.rs`.

The task is complete when the following criteria are met.

- Pool is created from a database URL string.
- Pool settings are reasonable defaults for a service.

### Task 1.5: Create the images database migration

Create `migrations/0001_images.sql` with the image_records table schema as defined in the PRD.

- Create the `image_records` table with columns: id (TEXT PK), status, content_type, file_name, size_bytes (BIGINT, nullable), object_key, created_at, updated_at.
- The `size_bytes` column is nullable (BIGINT without NOT NULL) because the actual file size is unknown at upload time and is populated during post-upload validation.
- There is no `type` column. The table itself represents the image type. Short-form and long-form video will get their own tables (`short_videos`, `long_videos`) when implemented.
- Document that this migration is applied manually for local development.
- Note: There is no database trigger for `updated_at`. The application code is responsible for setting `updated_at` in all UPDATE queries.

The task is complete when the following criteria are met.

- SQL file is valid and creates the table with all columns and indexes.
- Column types and defaults match the PRD specification.
- The `size_bytes` column is BIGINT and nullable (no NOT NULL constraint).
- There is no `type` column in the schema.

### Task 1.6: Set up AWS SDK clients and CloudFront signer in foundation shared

Create `packages/foundation/src/shared/storage.rs` for the S3 client and `packages/foundation/src/shared/cdn.rs` for the CloudFront signer setup.

- Create a function to build an S3 client from the AWS config.
- Create a CloudFront signer wrapper using the `cloudfront_sign` crate. The wrapper takes a key pair ID, private key PEM, and domain, and provides a method to generate signed URLs using `cloudfront_sign::get_signed_url`.
- Don't use `aws-sdk-cloudfront`. That crate is a management API client, not a URL signing library.
- Export from `packages/foundation/src/shared/mod.rs`.

The task is complete when the following criteria are met.

- S3 client is constructed from shared AWS config.
- CloudFront signer wrapper is constructed from key pair ID, PEM key, and domain.
- Signed URLs can be generated with the correct domain, path, and expiry.
- Both are exported and ready for use by feature slices.

### Task 1.7: Define image domain types and repository

Create `packages/foundation/src/shared/image/` with image domain types and the repository trait used by multiple features (upload, serve, validation).

- Create `packages/foundation/src/shared/image/mod.rs` with module declarations.
- Create `packages/foundation/src/shared/image/model.rs` with:
  - `ImageId` newtype wrapping a String (ULID).
  - `ImageRecord` struct with all fields from the database schema. The `size_bytes` field is `Option<i64>` because it is nullable (unknown at upload time, populated during validation).
  - `ImageStatus` enum with variants: Pending, Ready, Failed.
  - `ImageRepository` trait with methods: `save`, `find_by_id`, `find_by_ids`, `update_status`. The `update_status` method must also accept an optional `size_bytes` parameter so validation can write the actual file size when transitioning to "ready."
- Create `packages/foundation/src/shared/image/repository.rs` with `PostgresImageRepository` implementing `ImageRepository`.
  - Implement `save`: INSERT the image record into the `image_records` table. The `size_bytes` column is inserted as NULL.
  - Implement `find_by_id`: SELECT an image record by ID.
  - Implement `find_by_ids`: SELECT image records by multiple IDs using `WHERE id = ANY($1)`.
  - Implement `update_status`: UPDATE status, updated_at, and optionally size_bytes for a given image ID. All UPDATE queries must include `updated_at = now()` since there is no database trigger.
  - Handle mapping between database rows and domain types (`ImageStatus`).
- Export from `packages/foundation/src/shared/mod.rs`.

The task is complete when the following criteria are met.

- Domain types are defined with appropriate derives (Debug, Clone, etc.).
- `ImageRepository` trait defines `save`, `find_by_id`, `find_by_ids`, and `update_status` methods.
- The `ImageRecord` struct has `size_bytes` as `Option<i64>`.
- The `save` method inserts NULL for `size_bytes`.
- The `update_status` method can optionally update `size_bytes`.
- `PostgresImageRepository` correctly inserts, retrieves, batch-retrieves, and updates image records.
- Status enum maps correctly to and from database text values.
- All features (upload, serve, validation) can import these types from `shared::record::image_record`.

## Phase 2: Upload feature

This phase implements the presigned URL generation endpoint, from the domain layer through the API handler.

### Task 2.1: Define upload domain models and traits

Create the image upload feature slice structure under `packages/foundation/src/feature/image/upload/`.

- Create `packages/foundation/src/feature/image/upload/mod.rs` with module declarations.
- Create `packages/foundation/src/feature/image/upload/model.rs` with:
  - `ImageStorage` trait with methods: `generate_presigned_upload_url` (signs content type into the presigned URL so S3 rejects mismatched `Content-Type` headers), `get_object_metadata` (returns object size for validation), and `delete_object` (removes S3 objects on validation failure).
- Create `packages/foundation/src/feature/image/upload/error.rs` with `UploadError` enum.
- Import shared types (`ImageRecord`, `ImageId`, `ImageStatus`, `ImageRepository`) from `shared::record::image_record`.
- Export from `packages/foundation/src/feature/image/mod.rs`.

The task is complete when the following criteria are met.

- `ImageStorage` trait defines the infrastructure boundary for presigned URL generation, object metadata retrieval, and object deletion.
- Shared types are imported from `shared::record::image_record`, not redefined.
- Error types use `thiserror` for ergonomic error handling.

### Task 2.2: Implement the `CreateImageUploadUrl` command and executor

Create `packages/foundation/src/feature/image/upload/command.rs` with the command DTO and executor.

- Define `CreateImageUploadPresignedUrlCommand` with fields: content_type (String), file_name (String). No `size_bytes` field.
- Define `CreateImageUploadPresignedUrlCommandExecutor` with dependencies: `Arc<dyn ImageRepository>`, `Arc<dyn ImageStorage>`, and config values (bucket name, expiry).
- Implement `execute` method:
  1. Validate content_type is an exact match against the allowlist: `image/jpeg`, `image/png`, `image/webp`, `image/avif`, `image/gif`. Reject pattern matches (for example, `image/*`), parameterized types (for example, `image/jpeg;text/html`), and any value not in the list.
  2. Validate file_name doesn't exceed 255 characters, is non-empty, and contains no control characters (ASCII 0-31).
  3. Generate a ULID for the image ID.
  4. Construct the object key as `uploads/{image_id}` (no filename in the key).
  5. Create an `ImageRecord` with status "pending" and `size_bytes` as None, then save it to the repository.
  6. Generate a presigned PUT URL via the storage trait, with content type signed into the URL.
  7. Return `PresignedUrl { image_id, upload_url, expires_at }`.
- Write unit tests using hand-written mock implementations of `ImageRepository` and `ImageStorage`.

The task is complete when the following criteria are met.

- Command validates content type as an exact match against the allowlist.
- Parameterized MIME types (for example, `image/jpeg;text/html`) are rejected.
- No size validation is performed (size is enforced only during post-upload validation).
- Object key format is `uploads/{image_id}` with no user-supplied filename.
- Image record is persisted with `size_bytes` as NULL before returning the presigned URL.
- Presigned URL is generated with the content type signed into the URL.
- Tests cover: successful presigned URL creation, invalid content type rejection, parameterized MIME type rejection.

### Task 2.3: Implement `S3ImageStorage` for presigned URL generation

Create the `ImageStorage` trait implementation using the AWS S3 SDK.

- Implement `generate_presigned_upload_url` using the S3 presigned request builder.
- Configure the presigned URL with: PUT method, content type (signed into the URL), and expiry duration.
- **Verify that the generated presigned URL includes `content-type` in `X-Amz-SignedHeaders`.** Some AWS SDKs silently exclude `Content-Type` from signed headers, which means S3 would accept any Content-Type on upload. If the Rust SDK does not include it, document the limitation and rely on post-upload validation as the authoritative enforcement point.
- Note: S3 presigned PUT URLs cannot enforce `content-length-range`. File size enforcement happens during post-upload validation.
- Return the URL string and expiry timestamp.

The task is complete when the following criteria are met.

- Presigned URL is generated with PUT method, correct content type, and expiry.
- The generated URL's `X-Amz-SignedHeaders` includes `content-type`. If the SDK does not support this, document the limitation.

### Task 2.4: Wire the upload endpoint in the API package

Set up the axum server in `packages/api/` and wire the upload endpoint.

- Create `packages/api/src/container.rs` with `AppState` that holds the `CreateImageUploadPresignedUrlCommandExecutor`.
- Implement `AppState::build(config: &Config)` that wires all dependencies.
- Create `packages/api/src/routes/image/mod.rs` and `packages/api/src/routes/image/upload.rs`.
- Implement the `POST /api/image/v1/upload/create-presigned-url` handler:
  1. Parse JSON request body with `content_type` and `file_name` fields only. No `size_bytes`.
  2. Construct `CreateImageUploadPresignedUrlCommand`.
  3. Call the executor.
  4. Return 201 Created with the response body.
  5. Map domain errors to appropriate HTTP status codes (400 for invalid content type).
- Update `packages/api/src/main.rs` to initialize tracing, load config, build state, and start the axum server.

The task is complete when the following criteria are met.

- API server starts and listens on a configured port.
- POST /api/image/v1/upload/create-presigned-url accepts a valid request and returns 201 with image_id, upload_url, and expires_at.
- Invalid content types return 400.
- No 413 response exists (no size validation at the API level).
- Handler is thin and delegates to the command executor.

## Phase 3: Serve feature

This phase implements the image serving endpoints that generate CloudFront signed URLs.

### Task 3.1: Define serve domain models and traits

Create the serve feature slice under `packages/foundation/src/feature/image/download/`.

- Create `packages/foundation/src/feature/image/download/mod.rs` with module declarations.
- Create `packages/foundation/src/feature/image/download/model.rs` with:
  - `ImageAccess` struct with fields: image_id, download_url, expires_at.
  - `ImageCdn` trait with a `generate_signed_url` method that takes an object key and returns a signed URL.
- Create `packages/foundation/src/feature/image/download/error.rs` with `ServeError` enum (NotFound, CdnSigningFailed, etc.).
- Import `ImageRepository` and shared types from `shared::record::image_record`.
- Export from `packages/foundation/src/feature/image/mod.rs`.

The task is complete when the following criteria are met.

- Domain types and traits are defined.
- Shared types are imported from `shared::record::image_record`, not redefined.
- Error types cover not-found and CDN signing failure cases.

### Task 3.2: Implement `GetImage` query and executor

Create `packages/foundation/src/feature/image/download/query.rs` with the query DTOs and executors.

- Define `GetImageDownloadSignedUrlQuery` with field: image_id (String).
- Define `GetImageDownloadSignedUrlQueryExecutor` with dependencies: `Arc<dyn ImageRepository>`, `Arc<dyn ImageCdn>`, and config values (signed URL expiry).
- Implement `execute` method:
  1. Look up the image record by ID.
  2. Return not-found error if the record doesn't exist or status isn't "ready."
  3. Generate a CloudFront signed URL using the CDN trait.
  4. Return `ImageAccess`.
- Define `GetImageDownloadSignedUrlsQuery` with field: image_ids (Vec of String).
- Define `GetImageDownloadSignedUrlsQueryExecutor` with the same dependencies.
- Implement batch `execute` method:
  1. Validate the list isn't empty and doesn't exceed 50 items.
  2. Deduplicate input IDs before querying. Look up all image records by unique IDs (single query via `find_by_ids`).
  3. For each "ready" record, generate a signed URL.
  4. Return results split into found items and not-found IDs.
- Write unit tests for both executors.

The task is complete when the following criteria are met.

- Single query returns a signed URL for a "ready" image record.
- Single query returns not-found for missing or non-ready records.
- Batch query returns signed URLs for all "ready" records and lists not-found IDs.
- Batch query rejects requests exceeding 50 items.
- Tests cover: successful retrieval, not-found, batch with mixed results, batch limit exceeded.

### Task 3.3: Implement `CloudFrontImageCdn` for signed URL generation

Implement the `ImageCdn` trait using the `cloudfront_sign` crate.

- Create a `CloudFrontImageCdn` struct that holds the CloudFront domain, key pair ID, and private key PEM.
- Implement `generate_signed_url` using `cloudfront_sign::get_signed_url` with the configured domain, key pair ID, and private key.
- The signed URL format: `https://{cloudfront_domain}/{object_key}?Signature=...&Key-Pair-Id=...&Expires=...`
- Set expiry to the configured duration (default 10 minutes).
- Don't use `aws-sdk-cloudfront`. That crate is a management API client, not a URL signing library.

The task is complete when the following criteria are met.

- Signed URLs are generated with the correct domain, path, and expiry.
- Signing uses the `cloudfront_sign` crate with the RSA private key and key pair ID from configuration.

### Task 3.4: Wire the serve endpoints in the API package

Add the serve routes to the axum router and wire dependencies.

- Create `packages/api/src/routes/image/download.rs` with two handlers:
  1. `GET /api/image/v1/download/get-signed-url/{image_id}` handler for single retrieval.
  2. `POST /api/image/v1/download/get-signed-urls` handler for batch retrieval.
- Add `GetImageDownloadSignedUrlQueryExecutor` and `GetImageDownloadSignedUrlsQueryExecutor` to `AppState`.
- Wire `CloudFrontImageCdn` in `container.rs`.
- Map domain errors to HTTP status codes (404 for not found, 400 for bad request).

The task is complete when the following criteria are met.

- GET /api/image/v1/download/get-signed-url/{image_id} returns 200 with a signed URL for ready images.
- GET /api/image/v1/download/get-signed-url/{image_id} returns 404 for missing or non-ready images.
- POST /api/image/v1/download/get-signed-urls returns 200 with items and not_found arrays.
- POST /api/image/v1/download/get-signed-urls returns 400 for empty or oversized requests.

## Phase 4: Post-upload validation via SQS

This phase implements the processor worker that consumes S3 event notifications from SQS, validates uploaded images, and updates image status.

### Task 4.1: Implement SQS client in foundation shared

Create `packages/foundation/src/shared/queue.rs` for SQS message consumption.

- Create a function to build an SQS client from AWS config.
- Define a `QueueConsumer` trait with methods: `receive_messages`, `delete_message`.
- Implement `SqsQueueConsumer` using the AWS SQS SDK.
- Configure long polling (20 seconds wait time) and max number of messages per receive.

The task is complete when the following criteria are met.

- SQS client is constructed from shared AWS config.
- Long polling is configured for efficient message retrieval.
- Messages can be received and deleted.

### Task 4.2: Implement S3 event parsing with object key validation

Create a module to parse S3 event notification messages from SQS, with strict object key validation.

- Define structs for the S3 event notification JSON format (Records, S3 object, bucket).
- Implement parsing to extract the object key from each record.
- Validate the object key format using a regex pattern (`^uploads/[0-7][0-9A-HJKMNP-TV-Z]{25}$` for ULID characters) before extracting the image ID. Reject keys that don't match the expected format.
- Handle the SQS message wrapper (the S3 event JSON is nested inside the SQS message body).

The task is complete when the following criteria are met.

- S3 event notifications are correctly parsed from SQS message bodies.
- Object keys are validated against the expected format regex before processing.
- Image IDs are extracted only from validated object keys.
- Malformed messages and invalid object key formats are logged and skipped.

### Task 4.3: Define validation command and executor

Create a validation command under the image upload feature (validation is part of the upload lifecycle).

- Define `ValidateImageCommand` with fields: image_id (ImageId), object_key (String).
- Define `ValidateImageCommandExecutor` with dependencies: `Arc<dyn ImageRepository>`, `Arc<dyn ImageStorage>`, and config values (max upload size).
- Implement `execute` method:
  1. Look up the image record by ID.
  2. Return an error if the image record isn't found.
  3. Check that the status is "pending". If not, skip processing (idempotency guard for SQS at-least-once delivery) and return success.
  4. Get the object metadata from S3 (via the storage trait) to read the actual file size.
  5. If the file size exceeds the configured max upload size (10 MB), update status to "failed" and return.
  6. Run validation (stub: always succeeds). Mark this stub clearly with a comment: "MUST replace with magic-bytes MIME type detection before production use." The stub must log a warning on every invocation to ensure it isn't silently left in production.
  7. On success: update image status to "ready" and write the actual `size_bytes` value via `update_status`.
  8. On failure (oversized or invalid): update image status to "failed" and delete the S3 object via the storage trait to prevent storage accumulation.
- Write unit tests.

The task is complete when the following criteria are met.

- Image ID and object key are provided directly (already extracted and validated by the event parser).
- The executor reads actual file size from S3 object metadata.
- Files exceeding 10 MB are marked as "failed."
- On success, image status is updated to "ready" and `size_bytes` is populated with the actual file size.
- Records not in "pending" status are skipped (idempotency).
- Returns an error if the image record isn't found.
- The stub validator has a clear comment indicating it must be replaced before production.
- Failed image records have their S3 objects deleted.
- Tests cover: successful validation, file size exceeded (with S3 object deletion), image not found, idempotent skip of already-processed records.

### Task 4.4: Wire the processor worker

Set up the processor binary to consume SQS messages and run validation.

- Create `packages/processor/src/container.rs` with dependency wiring.
- Update `packages/processor/src/main.rs` to:
  1. Initialize tracing.
  2. Load config.
  3. Build dependencies.
  4. Run a polling loop: receive messages, parse S3 events, validate object key format (regex), extract image ID, execute validation, delete messages.
- Handle graceful shutdown on SIGTERM/SIGINT.

The task is complete when the following criteria are met.

- Processor starts and polls the SQS queue.
- Object keys are validated against the regex before extraction.
- S3 event messages are parsed and validation is executed for each valid record.
- Successfully processed messages are deleted from the queue.
- Failed messages (transient errors) are left in the queue for retry.
- Validation failures (bad image or oversized file) delete the S3 object, update status to "failed," and delete the SQS message.
- Messages with invalid object key formats are logged and deleted.
- Processor shuts down gracefully on termination signals.

## Phase 5: Health check and API middleware

This phase adds a health check endpoint and basic HTTP middleware.

### Task 5.1: Add health check endpoint

Create `packages/api/src/routes/health.rs` with a basic health check.

- Implement `GET /api/health/lieveness` that returns 200 OK.

The task is complete when the following criteria are met.

- GET /api/health/liveness returns 200 when the service is running.
- Response includes a simple status indicator.

### Task 5.2: Add HTTP middleware

Add `tower-http` middleware to the axum router.

- Add request tracing middleware (log method, path, status code, duration).
- Add CORS middleware with sensible defaults for a backend API.
- Configure in `packages/api/src/main.rs` or a dedicated middleware module.

The task is complete when the following criteria are met.

- All requests are traced with method, path, status, and duration.
- CORS headers are set for API consumption.
