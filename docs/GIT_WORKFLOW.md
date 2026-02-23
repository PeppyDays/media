# Git workflow

Guidelines for managing branches, commits, and pull requests.

## Branching

Always create a new branch from `main` before starting work. If the current branch has already been merged, switch back to `main` and create a fresh branch.

```bash
git switch main
git pull
git switch -c <branch-name>
```

Branch names must be lowercase and describe the change (for example, `add-image-upload`, `fix-presigned-url-expiry`).

## Commit guidelines

Each commit captures a single, focused change that keeps the codebase in a working state.

### Size and focus

Each commit must represent a single, focused change. This makes reviews easier and history more navigable.

- **One subject per commit**: Address only one logical change
- **Up to 200 lines changed**: Excluding documentation and tests
- **Self-contained**: Each commit must leave the codebase in a working state

### Message format

Use the following format for commit messages:

```
<type>: <summary>

- <bullet points describing the changes>
```

- **type**: Category of change (see table below)
- **summary**: Brief description in imperative mood

### Commit types

The following table lists the recognized commit types:

| Type       | Description                                   |
| ---------- | --------------------------------------------- |
| `feat`     | New feature or functionality                  |
| `fix`      | Bug fix                                       |
| `refactor` | Code restructuring without changing behavior  |
| `chore`    | Maintenance tasks (dependencies, configs, CI) |
| `docs`     | Documentation changes                         |
| `test`     | Adding or updating tests                      |
| `perf`     | Performance improvements                      |
| `style`    | Code style changes (formatting, naming)       |

**Body**: Briefly describe what changed and why. Focus on the intent, not the implementation details visible in the diff.

**Example:**

```
feat: add retry logic for image downloads

- Add exponential backoff for transient network failures
- Limit retries to 3 attempts with 1s, 2s, 4s delays
- Log retry attempts for observability
```

### Pre-commit checks

Before committing, ensure all linting and tests pass:

```bash
# Run linting
task check

# Run tests
task test
```

Both must pass before creating a commit. Fix any issues before proceeding.

### Co-authoring

Credit collaborators or AI assistance using the `Co-Authored-By` trailer at the end of the commit message:

```
feat: add retry logic for image downloads

- Add exponential backoff for transient network failures

Co-Authored-By: Name <email@example.com>
Co-Authored-By: Claude <noreply@anthropic.com>
```

## Pull request guidelines

These guidelines keep pull requests focused and easy to review.

### Size

Keep pull requests small and focused:

- **Up to 3 commits per PR**: Keeps reviews focused and manageable
- If a feature requires more commits, consider splitting into multiple PRs

### Title format

PR titles follow the same convention as commit messages:

```
<type>: <summary>
```

- **type**: Same types as commits (`feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `perf`, `style`)
- **summary**: Imperative mood, lowercase, no trailing period

When a PR contains multiple commit types, use the type that best represents the overall change. If a PR includes both a feature and its documentation, use `feat`.

**Examples:**

```
feat: add presigned URL generation for image uploads
fix: handle expired tokens in upload validation
docs: add testing guidelines and conventions
chore: add CI pipeline and tooling config
refactor: extract shared storage client into foundation
```

### Description format

The PR description summarizes the changes for reviewers:

```markdown
## Summary

- <bullet points summarizing all changes in the PR>

## Todo (optional)

- <manual steps required after deployment>
```

Focus on what changed and why. The diff shows the how.

### Updating after review

When changes are requested after opening a PR:

1. Make the necessary modifications
2. **Amend the relevant commit** rather than adding new commits
3. **Force push** to the origin branch

```bash
# Stage your changes
git add <files>

# Amend the relevant commit (if it's the most recent)
git commit --amend --no-edit

# Force push to update the PR
git push --force
```

For amending older commits, use interactive rebase:

```bash
# Start interactive rebase
git rebase -i HEAD~n  # where n is the number of commits to review

# Mark the commit to edit as 'edit', make changes, then:
git add <files>
git commit --amend --no-edit
git rebase --continue
git push --force
```

## Branch lifecycle

Branches are automatically deleted when their pull request is merged. No manual cleanup required.

## Workflow summary

The following steps outline the complete workflow from branch creation to merge:

```
1. Branch           →  Create new branch from main
2. Develop          →  Make focused changes
3. Verify           →  Run `task check` and `task test`
4. Commit           →  Single subject, ≤200 lines, clear message
5. Open PR          →  ≤3 commits, summary + todo
6. Address feedback →  Amend relevant commits, force push
7. Merge            →  Branch auto-deleted
```
