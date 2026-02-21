# Testing guidelines

## Overview

Unit tests covering API layer and domain components. Integration, e2e, or benchmark tests will be added later.

## Testing approach

Classicist style, not Mockist. Prefer real implementations to ensure test reliability and catch integration issues early.

- Use real dependencies backed by local infrastructure (PostgreSQL, S3-compatible storage, etc.)
- Only mock external service clients (CloudFront signer, third-party APIs) that require network calls or AWS credentials

## Directory structure

Tests are organized within each package:

```plaintext
packages/
├── api/
│   ├── src/
│   │   └── routes/
│   │       └── upload.rs           # includes #[cfg(test)] mod tests
│   └── tests/                      # API integration tests
│       ├── common/
│       │   └── mod.rs              # Shared test utilities, fixtures
│       └── upload_test.rs
├── foundation/
│   ├── src/
│   │   └── feature/
│   │       ├── upload/
│   │       │   └── command.rs      # includes #[cfg(test)] mod tests
│   │       └── serve/
│   │           └── query.rs        # includes #[cfg(test)] mod tests
│   └── tests/                      # Foundation integration tests
│       └── ...
└── processor/
    ├── src/
    │   └── consumers/
    │       └── transcode.rs        # includes #[cfg(test)] mod tests
    └── tests/                      # Processor integration tests
        └── ...
```

- **Unit tests**: Inside the source file as `#[cfg(test)] mod tests { ... }`
- **Integration tests**: In each package's `tests/` directory, one file per feature

### Running tests

Use the following commands to run tests:

```bash
# Run all tests across the workspace
task test

# Run tests for a specific package
cargo test -p foundation

# Run a specific test
cargo test test_name
```

## Basic conventions

These conventions keep tests readable and maintainable.

### AAA pattern

Structure each test using the Arrange-Act-Assert pattern:

```rust
#[test]
fn sut_returns_signed_url_when_media_is_image() {
    // Arrange
    let media = arrange_media_with(MediaType::Image);

    // Act
    let result = sut.get_access(&media);

    // Assert
    assert!(matches!(result, Ok(MediaAccess::SignedUrl { .. })));
}
```

### File naming

Follow these naming patterns for test files:

- Integration test files: `{feature}_test.rs`
- Unit tests: `#[cfg(test)] mod tests` within the source file

### Function naming

Name test functions using the `sut_{behavior}_when_{condition}` pattern (SUT = System Under Test). Keep it concise but with enough context to understand the scenario.

```rust
// Good: describes behavior and condition
sut_returns_signed_url_when_media_is_image
sut_returns_not_found_error_when_media_does_not_exist
sut_creates_upload_url_with_correct_content_type
sut_returns_cursor_as_none_when_items_less_than_limit
```

### File structure

Within each `#[cfg(test)] mod tests` block:

1. **Imports** — `use super::*` and test dependencies
2. **Fixtures / setup functions** — test helpers and builder creation
3. **Tests** — all `#[test]` functions
4. **Arrange helpers** — `arrange_*` functions

### Helper functions

Extract helper functions to keep tests clean and maintainable.

```rust
fn arrange_media() -> Media {
    Media {
        id: MediaId(rand::random()),
        media_type: random_element(&[MediaType::Image, MediaType::ShortVideo, MediaType::LongVideo]),
        status: random_element(&[MediaStatus::Pending, MediaStatus::Ready]),
        content_type: "image/jpeg".to_string(),
        size_bytes: rand::random_range(1024..10_000_000),
        created_at: Utc::now(),
    }
}

fn arrange_upload_payload() -> UploadPayload {
    UploadPayload {
        content_type: "image/png".to_string(),
        size_bytes: rand::random_range(1024..50_000_000),
    }
}
```

## Test data

These guidelines ensure test data communicates intent clearly.

### No fixed values for irrelevant data

**Never use fixed values when the specific value is irrelevant to the test.** Fixed values without context imply special meaning where none exists. Use random or arbitrary data to make it clear the specific value doesn't matter.

### Arrange pattern

**Avoid constructing domain objects directly in tests.** Use `arrange_*` helper functions that return fully random data. When a test needs specific field values, call the arrange function first, then override only the fields that matter.

**Arrange functions take no parameters.** They return fully random data. Don't add parameters to customize the returned object.

```rust
// Good: arrange function returns fully random data, no parameters
fn arrange_media() -> Media {
    Media {
        id: MediaId(rand::random()),
        media_type: random_element(&[MediaType::Image, MediaType::ShortVideo, MediaType::LongVideo]),
        status: random_element(&[MediaStatus::Pending, MediaStatus::Ready, MediaStatus::Failed]),
        content_type: "image/jpeg".to_string(),
        size_bytes: rand::random_range(1024..10_000_000),
        created_at: Utc::now(),
    }
}

// Good: override only what the test cares about
#[test]
fn sut_returns_signed_cookie_when_media_is_long_video() {
    let mut media = arrange_media();
    media.media_type = MediaType::LongVideo;
    // ...
}

// Bad: arrange function with parameters
fn arrange_media(media_type: MediaType) -> Media { ... }

// Bad: constructing domain objects directly in test body
#[test]
fn sut_returns_signed_cookie_when_media_is_long_video() {
    let media = Media {
        id: MediaId(42),                        // why 42?
        media_type: MediaType::LongVideo,
        status: MediaStatus::Ready,             // does status matter here?
        content_type: "video/mp4".to_string(),
        size_bytes: 1024,
        created_at: Utc::now(),
    };
}
```

### Fixed values for meaningful data

**Use fixed values only when they carry meaning.** For edge cases, boundary conditions, or domain-specific scenarios. Each fixed value must have a comment explaining why that specific value is necessary.

```rust
// Good: comment explains why this specific value matters
#[test]
fn sut_rejects_upload_when_file_exceeds_max_size() {
    // 100MB is the domain-defined upload limit
    let max_size: u64 = 100 * 1024 * 1024;
    let mut media = arrange_media();
    media.size_bytes = max_size + 1;
    // ...
}

// Bad: fixed value with no explanation
#[test]
fn sut_processes_video_correctly() {
    let mut media = arrange_media();
    media.size_bytes = 8_458_796; // why this exact number?
    // ...
}
```

### Parametrized tests

Prefer `rstest` over duplicated tests when testing multiple inputs with the same logic.

```rust
// Good: parametrized with rstest
#[rstest]
#[case(MediaType::Image, AccessMethod::SignedUrl)]
#[case(MediaType::ShortVideo, AccessMethod::SignedUrl)]
#[case(MediaType::LongVideo, AccessMethod::SignedCookie)]
fn sut_selects_correct_access_method(
    #[case] media_type: MediaType,
    #[case] expected: AccessMethod,
) {
    let mut media = arrange_media();
    media.media_type = media_type;
    let result = determine_access_method(&media);
    assert_eq!(result, expected);
}

// Bad: separate tests with identical structure
#[test]
fn sut_selects_signed_url_for_image() { ... }
#[test]
fn sut_selects_signed_url_for_short_video() { ... }
#[test]
fn sut_selects_signed_cookie_for_long_video() { ... }
```

## Feature slice testing

Each feature slice has tests at two layers: API and domain.

### API layer tests

Isolate the API layer by providing mock implementations of command/query executors via traits. This tests:

- Request parsing and validation
- Command/Query construction
- Response formatting
- Error code mapping

```rust
#[tokio::test]
async fn sut_delivers_command_to_executor_correctly() {
    // Arrange
    let (executor, spy) = mock_executor();
    let app = build_test_app(executor);
    let payload = arrange_payload();

    // Act
    let _ = app.post("/api/v1/upload-url").json(&payload).send().await;

    // Assert
    let captured = spy.captured_command();
    assert_eq!(captured.content_type, payload.content_type);
    assert_eq!(captured.size_bytes, payload.size_bytes);
}
```

### Domain layer tests

Use real dependencies. Test command/query executors with actual repositories and infrastructure.

**Don't test internal implementations.** Commands and Queries represent all domain behaviors. Test only at this level.

**Treat the executor as a black box.** Don't assume the order in which the executor calls its dependencies. When testing an error case for one dependency, arrange all other dependencies with valid return values first — then override only the one under test.

```rust
// Good: all dependencies return valid values, only one is overridden
#[tokio::test]
async fn sut_returns_not_found_when_media_does_not_exist() {
    let mut media_repo = mock_repo();
    media_repo.expect_find().returning(|_| Ok(None));  // case under test

    let mut storage = mock_storage();
    storage.expect_sign_url().returning(|_| Ok(signed_url()));  // arranged normally
    // ...
}

// Bad: only sets up media_repo, assumes storage is never called
```

### Dependency override via traits

Each infrastructure dependency is represented by a trait. Tests provide mock implementations:

```rust
// Production: real S3 implementation
struct S3Storage { client: aws_sdk_s3::Client }
impl MediaStorage for S3Storage { ... }

// Test: mock implementation
struct MockStorage { ... }
impl MediaStorage for MockStorage { ... }
```

Use `mockall` or hand-written mocks. Prefer hand-written mocks for simple interfaces, `mockall` for complex ones.
