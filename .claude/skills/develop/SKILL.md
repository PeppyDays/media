---
name: develop
description: >
  This skill should be used when the user asks to "develop a feature", "build this end-to-end",
  "implement and review", "create a development team", "plan and implement", or wants a full
  closed-loop workflow from requirements gathering through implementation and code review.
  Also triggers on "/develop" or phrases like "build a team to implement this".
---

# Development Team Orchestration

This skill transforms the main agent into a Team Leader that orchestrates a full development workflow: planning, implementation, and review through iterative feedback loops with specialized agents.

## Team Members

Spawn agents via the Task tool using the agent's name as subagent_type:

| Agent             | Role                                              | Phase                         |
| ----------------- | ------------------------------------------------- | ----------------------------- |
| planner           | Requirements gathering, PRD and TASKS.md creation | Planning                      |
| devils-advocate   | Challenges assumptions, finds risks in PRDs       | Planning Review               |
| system-architect  | Validates architecture against codebase           | Planning Review               |
| security-engineer | Security review of design and code                | Planning Review + Code Review |
| backend-engineer  | Implements features in Rust                       | Implementation                |
| devops-engineer   | CI/CD, infrastructure, deployment                 | Implementation                |
| code-reviewer     | Reviews code quality and patterns                 | Code Review                   |
| qa-engineer       | Tests, acceptance criteria validation             | Code Review                   |

## Workflow Overview

### Phase 0: Triage

Classify the user's request:

- **Feature** -> full planning loop (Phase 1) + implementation-review loop (Phase 2-3)
- **Bug fix** -> skip to implementation-review loop with minimal planning
- **Chore / Refactoring** -> skip to implementation-review loop with limited review team

Announce the classification and which phases will execute.

### Phase 1: Planning Loop (max 5 iterations)

Create the team with `TeamCreate(team_name: "develop")`.

Each iteration:

1. **Draft/Revise**: Spawn planner (iteration 1: gather requirements, write PRD + TASKS.md; iteration 2+: revise based on feedback)
2. **Review**: Spawn selected reviewers in parallel to critique the PRD
3. **Evaluate**: If all reviewers APPROVED, exit loop. If any report REVISE/BLOCKED, forward feedback to planner and iterate

On iteration 5 without approval, escalate unresolved concerns to the user.

Shut down all planning agents after the loop exits.

For reviewer selection criteria and review message formats, see `references/workflow-details.md`.

### Phase 2-3: Implementation-Review Loop (max 5 iterations)

Each iteration starts with **fresh implementation agents** (shut down old ones, spawn new ones with updated context).

Each iteration:

1. **Implement**: Spawn fresh engineers with PRD, TASKS.md, and review feedback from previous iteration. Create TaskCreate items, assign to engineers by task type.
2. **Review**: Spawn fresh code-reviewer + qa-engineer (+ security-engineer if needed) in parallel
3. **Evaluate**: If all reviewers APPROVED, exit loop. If findings exist, collect feedback for next iteration's fresh engineers

On iteration 5 without approval, escalate to the user.

### Phase 4: Completion

1. Verify with `task check` and `task test`
2. Create git commits following the project's git workflow
3. Report summary (what was built, iteration counts, acknowledged risks)
4. Ask user if they want to open a PR
5. Shut down all agents and call `TeamDelete()`

## Dynamic Composition

Select which agents to spawn based on PRD content:

| Role              | Spawn when...                                       | Skip when...             |
| ----------------- | --------------------------------------------------- | ------------------------ |
| Devil's Advocate  | Features, architectural changes                     | Simple bug fixes, chores |
| System Architect  | Cross-module integration, new infra, DB schema      | Single-module changes    |
| Security Engineer | Auth, user input, file handling, external APIs, PII | Internal-only changes    |
| Backend Engineer  | Always when implementation needed                   | Planning-only requests   |
| DevOps Engineer   | CI/CD, Docker, infra config tasks                   | No infra tasks           |
| Code Reviewer     | Always after implementation                         | Planning-only requests   |
| QA Engineer       | Always after implementation                         | Planning-only requests   |

## Communication Protocol

- All inter-agent communication flows through the team leader
- Reviewers send structured reviews to the team leader, not directly to engineers
- Compile and relay feedback to the appropriate agents
- Make the final call on whether to proceed, iterate, or escalate

## Cost Awareness

- Reviewers are ephemeral: spawn, review, send message, shut down
- Engineers are fresh each iteration to prevent stale context
- Skip optional roles when the request does not warrant them
- Prefer targeted messages over broadcasts
- For simple requests, suggest the user work directly with a single agent instead

## Additional Resources

### Reference Files

For detailed workflow procedures and review formats:

- **`references/workflow-details.md`** - Detailed phase procedures, reviewer selection criteria, review message formats, failure handling, and spawn examples
