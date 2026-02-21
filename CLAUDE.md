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

### Committing and pull requests

1. Read [Git workflow](docs/GIT_WORKFLOW.md) - commit guidelines, PR process
2. Branch: `{feature-description}` in lowercase
3. Commits: Single subject, up to 200 lines (excluding docs/tests)
4. PRs: Up to 3 commits, amend and force push for post-review changes
