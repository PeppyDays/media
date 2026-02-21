# Development Team Workflow Details

Detailed procedures, review formats, and spawn examples for the development team orchestration skill.

## Phase 1: Planning Loop Details

### Spawning the Planner

Iteration 1:

```
Task(subagent_type: "planner", team_name: "develop", name: "planner",
     prompt: "User request: {user's request}. Gather requirements, analyze the codebase, and produce a PRD and TASKS.md.")
```

Iteration 2+:

```
Task(subagent_type: "planner", team_name: "develop", name: "planner",
     prompt: "Revise the PRD at {prd_path} based on this feedback: {compiled_feedback}. Update both PRD and TASKS.md.")
```

### Reviewer Selection Criteria

Determine which planning reviewers to spawn based on PRD content:

- **Devil's Advocate**: Always for features and architectural changes. Skip for simple bug fixes.
- **System Architect**: When PRD contains cross-module integration, new infrastructure, DB schema changes. Skip for single-module changes.
- **Security Engineer**: When PRD involves auth, user input, file handling, external APIs, PII. Skip for internal-only changes.

### Spawning Reviewers in Parallel

```
Task(subagent_type: "devils-advocate", team_name: "develop", name: "devils-advocate",
     prompt: "Review the PRD at {prd_path}. Send your structured critique to the team leader.")
Task(subagent_type: "system-architect", team_name: "develop", name: "system-architect",
     prompt: "Review the PRD at {prd_path}. Validate architecture against the existing codebase. Send your review to the team leader.")
Task(subagent_type: "security-engineer", team_name: "develop", name: "security-engineer",
     prompt: "Review the PRD at {prd_path} for security concerns. Send your review to the team leader.")
```

### Planning Review Evaluation

After collecting all reviews:

- If ALL reviewers report APPROVED -> exit planning loop
- If any reviewer reports REVISE -> compile feedback, send to planner, iterate
- If any reviewer reports BLOCKED -> compile feedback, include blocker details, send to planner
- If iteration 5 reached -> present remaining concerns to user, ask whether to proceed, adjust, or abandon

## Phase 2-3: Implementation-Review Loop Details

### Fresh Agent Spawning

Before each implementation iteration:

1. Send `SendMessage(type: "shutdown_request")` to existing implementation agents
2. Prepare context: PRD path, TASKS.md path, git diff of current state, review feedback (if iteration 2+)
3. Spawn fresh agents with the updated context

### Task Classification and Assignment

Read TASKS.md and classify each task:

- Rust code, API endpoints, business logic, database -> assign to backend-engineer
- CI/CD, Docker, Taskfile, infrastructure config -> assign to devops-engineer
- Both -> assign to backend-engineer (primary), devops-engineer for infra portions

Create TaskCreate items for each task with dependencies using addBlockedBy for sequential tasks.

### Spawning Implementation Agents

```
Task(subagent_type: "backend-engineer", team_name: "develop", name: "backend-engineer",
     prompt: "Implement your assigned tasks. PRD: {prd_path}. TASKS: {tasks_path}.
              Review feedback to address: {review_feedback_if_any}.
              Check TaskList for your assignments. Run `task check` and `task test` before completing each task.")
Task(subagent_type: "devops-engineer", team_name: "develop", name: "devops-engineer",
     prompt: "Implement your assigned infrastructure tasks. PRD: {prd_path}. TASKS: {tasks_path}.
              Review feedback to address: {review_feedback_if_any}.
              Check TaskList for your assignments.")
```

### Spawning Review Agents

```
Task(subagent_type: "code-reviewer", team_name: "develop", name: "code-reviewer",
     prompt: "Review the implementation. PRD: {prd_path}. TASKS: {tasks_path}. Changed files: {file_list}.
              Send your structured review to the team leader.")
Task(subagent_type: "qa-engineer", team_name: "develop", name: "qa-engineer",
     prompt: "Validate the implementation. PRD: {prd_path}. TASKS: {tasks_path}. Changed files: {file_list}.
              Run tests, check acceptance criteria, write missing tests. Send your review to the team leader.")
```

### Implementation Review Evaluation

After collecting all reviews:

- If ALL reviewers report APPROVED -> exit implementation loop
- If any reviewer reports REVISE -> collect structured feedback, shut down engineers, iterate with fresh agents
- If iteration 5 reached -> present remaining issues to user, ask for guidance

## Failure Handling

### Devil's Advocate Findings

Categorize each finding:

- **Risk**: Potential issue, not blocking. Add to PRD risks section, continue.
- **Alternative**: Viable different approach. Present both options to user for decision. Pause until decided.
- **Blocker**: Fundamental flaw. Pause work, escalate to user immediately.

### Review Findings in Implementation Loop

Collected as structured feedback including:

- File paths and line ranges
- Severity (must-fix / should-fix / suggestion)
- Description of the issue
- Expected fix approach

Provide this feedback to fresh engineers in the next iteration.

### Max Iterations Reached

When 5 iterations are exhausted without full approval:

1. Summarize all unresolved issues
2. Present to user with options: proceed with acknowledged risks, adjust approach, or abandon
3. Wait for user decision before continuing

### Fresh Agent Context

Each new implementation agent receives:

- PRD file path
- TASKS.md file path
- Git diff of current state (what has been implemented so far)
- Specific review feedback to address (structured by file and severity)
- Any user decisions from escalations

## Review Message Formats

### Planning Review Format (all planning reviewers)

```
PLANNING REVIEW: {Agent Name}

{SECTION_NAME}:
- [finding description, or "None"]

{SECTION_NAME}:
- [finding description, or "None"]

VERDICT: [APPROVED / REVISE / BLOCKED]
```

### Code Review Format

```
CODE REVIEW: {Agent Name}

MUST FIX:
- [critical issues, or "None"]

SHOULD FIX:
- [significant improvements, or "None"]

SUGGESTIONS:
- [optional improvements, or "None"]

VERDICT: [APPROVED / REVISE]
```

### QA Review Format

```
QA REVIEW: QA Engineer

TEST SUITE: [PASS / FAIL - details]

ACCEPTANCE CRITERIA:
- Task X.Y: [MET / NOT MET - details]

COVERAGE GAPS:
- [untested scenario, or "None"]

TESTS ADDED:
- [test files added, or "None needed"]

BUGS FOUND:
- [bug with reproduction steps, or "None"]

VERDICT: [APPROVED / REVISE]
```
