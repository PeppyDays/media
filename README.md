# Media Service

A unified backend for uploading, transcoding, resizing, and securely serving platform media — images, short-form videos, and long-form videos.

We upload photos, scroll through short-form clips, and stream hours of video every day — but rarely think about what happens behind the scenes. How presigned URLs work, why short-form skips HLS, how adaptive bitrate streaming switches quality mid-play, or why every piece of media is served through signed URLs. This service explores all of that by building the full pipeline from scratch.

Given a media ID, the service resolves the optimal access URL for the underlying media type.

## Purpose

A personal project to learn media engineering by building a production-shaped service from scratch.

Beyond the media domain itself, this repo serves as a testbed for AI-assisted development — crafting prompts and closed-loop workflows that drive the full lifecycle: spec, planning, implementation, review, testing, merge, and deployment.

## Project structure

```plaintext
media/
├── packages/                       # Application packages
│   ├── api/                        # HTTP server (axum)
│   ├── foundation/                 # Core business logic
│   │   └── src/
│   │       ├── common/             # Config, tracing, shared utilities
│   │       ├── feature/            # Vertical feature slices
│   │       │   ├── upload/         # Presigned URL, validation
│   │       │   ├── serve/          # Signed URL/cookie, metadata
│   │       │   ├── transcode/      # FFmpeg, status management
│   │       │   └── manage/         # CRUD, search, deletion
│   │       └── shared/             # Shared infrastructure
│   │           ├── storage/        # S3 client
│   │           ├── cdn/            # CloudFront signing
│   │           ├── queue/          # SQS producer/consumer
│   │           └── database/       # PostgreSQL repository
│   └── processor/                  # Async transcoding worker
├── docs/                           # Development docs and proposals
├── resources/                      # Database migrations
├── Cargo.toml                      # Workspace root
└── Taskfile.yaml                   # Task runner
```

## Documentation

- [Development guidelines](docs/DEVELOPMENT.md) — design principles, naming conventions, lint rules
- [Testing guidelines](docs/TESTING.md) — test patterns, arrange helpers, feature slice testing
- [Ubiquitous languages](docs/UBIQUITOUS_LANGUAGES.md) — domain terminology
- [Git workflow](docs/GIT_WORKFLOW.md) — branching, commits, PR process
