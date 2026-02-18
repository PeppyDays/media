# Git Workflow

Guidelines for managing branches, commits, and pull requests.

## Branching

Always create a new branch from `main` before starting work. If the current branch has already been merged, switch back to `main` and create a fresh branch.

```bash
git switch main
git pull
git switch -c <branch-name>
```

Branch names should be lowercase and describe the change (e.g. `add-image-upload`, `fix-presigned-url-expiry`).

## Commit Guidelines

### Size and Focus

Each commit should represent a single, focused change. This makes reviews easier and history more navigable.

- **One subject per commit**: Address only one logical change
- **Up to 200 lines changed**: Excluding documentation and tests
- **Self-contained**: Each commit should leave the codebase in a working state

### Message Format

```
<type>: <summary>

- <bullet points describing the changes>
```

- **type**: Category of change (see table below)
- **summary**: Brief description in imperative mood

### Commit Types

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

### Pre-Commit Checks

Before committing, ensure all linting and tests pass:

```bash
# Run linting
task check

# Run tests
task test
```

Both must pass before creating a commit. Fix any issues before proceeding.

### Co-Authoring

Credit collaborators or AI assistance using the `Co-Authored-By` trailer at the end of the commit message:

```
feat: add retry logic for image downloads

- Add exponential backoff for transient network failures

Co-Authored-By: Name <email@example.com>
Co-Authored-By: Claude <noreply@anthropic.com>
```

## Pull Request Guidelines

### Size

- **Up to 3 commits per PR**: Keeps reviews focused and manageable
- If a feature requires more commits, consider splitting into multiple PRs

### Description Format

Follow the same format as commit messages:

```markdown
## Summary

<bullet points summarizing all changes in the PR>

## Todo (optional)

<manual steps required after deployment>
```

### Updating After Review

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

## Branch Lifecycle

Branches are automatically deleted when their pull request is merged. No manual cleanup required.

## Workflow Summary

```
1. Branch           →  Create new branch from main
2. Develop          →  Make focused changes
3. Verify           →  Run `task check` and `task test`
4. Commit           →  Single subject, ≤200 lines, clear message
5. Open PR          →  ≤3 commits, summary + todo
6. Address feedback →  Amend relevant commits, force push
7. Merge            →  Branch auto-deleted
```
