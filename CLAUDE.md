# Media Service guidelines

## Quick reference

Rules are automatically loaded from `.claude/rules/`. Reference them based on your task.

Before writing code, read the relevant documentation:

1. [Development guidelines](docs/DEVELOPMENT.md) - design principles, naming conventions, lint rules
2. [Testing guidelines](docs/TESTING.md) - test patterns, arrange helpers, feature slice testing
3. [Ubiquitous languages](docs/UBIQUITOUS_LANGUAGES.md) - domain terminology
4. [Git workflow](docs/GIT_WORKFLOW.md) - branching, commits, PR process

## Development workflow

When asked to develop a feature or fix, **always create a new branch first** before making any code changes. Follow the naming convention in [Git workflow](docs/GIT_WORKFLOW.md).

### Starting a new feature

1. **Create a branch first** - `{feature-description}` (lowercase, hyphen-separated)
2. Read [ubiquitous languages](.docs/UBIQUITOUS_LANGUAGES.md) - Understand domain terms
3. Read [architecture](.docs/ARCHITECTURE.md) - Know where code belongs
4. Read [development](.docs/DEVELOPMENT.md) - Follow naming conventions

### Writing tests

1. Read [testing](.docs/TESTING.md) - Testing philosophy & patterns
2. Read [architecture](.docs/ARCHITECTURE.md) - Layer-specific testing strategies

### Committing and pull requests

1. Read [git workflow](docs/GIT_WORKFLOW.md) - commit guidelines, PR process
2. Branch: `{feature-description}` in lowercase
3. Commits: Single subject, up to 200 lines (excluding docs/tests)
4. PRs: Up to 3 commits, amend and force push for post-review changes

## Key principles

- **Workspace architecture**: Code organized into separate packages (`api`, `foundation`, `processor`)
- **Vertical slice architecture**: Features are self-contained within `foundation`
- **Classicist testing**: Use real implementations, mock only external services
- **Self-documenting code**: Minimize docstrings, use clear naming
- **CQRS pattern**: Commands for writes, Queries for reads
