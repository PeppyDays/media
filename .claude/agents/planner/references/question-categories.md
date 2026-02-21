# PRD Question Categories

Detailed question templates with numbered options for each discovery category. Use these as a guide during the Explore phase.

## Problem and User Context

"What problem does this solve for users?"

1. Slow or degraded performance
2. Missing functionality users need
3. Manual process that should be automated
4. Data quality or consistency issues
5. Other (please specify)

## Target Users and Workflows

"Who are the primary users?"

1. Internal team members
2. External customers/clients
3. API consumers/developers
4. System administrators
5. Other (please specify)

## Success Criteria

"What defines success for this feature?"

1. Performance improvement (specify metrics)
2. User experience enhancement
3. Cost reduction
4. Scalability improvement
5. Security enhancement
6. Other (please specify)

## Technical Approach

Present domain-specific numbered options based on the feature type. Example for async processing:

1. Message queue (Redis, RabbitMQ, SQS)
2. Background job system (Celery, Bull, Sidekiq)
3. Event-driven architecture
4. Async/await patterns only
5. Other (please specify)

## Integration and Dependencies

"How should this integrate with existing systems?"

1. REST API integration
2. Event-based integration
3. Direct database access
4. Shared library/module
5. Other (please specify)

## Performance and Scale

"What are the expected performance requirements?"

1. Low latency (<100ms)
2. High throughput (>1000 req/s)
3. Large data volumes (>1TB)
4. Real-time processing
5. Batch processing acceptable
6. Other (please specify)

## Progressive Questioning Strategy

- Start with high-level questions (problem, users, goals)
- Progress to technical details (approach, integration, performance)
- Adapt follow-up questions based on previous answers
- Ask clarifying questions only when responses are ambiguous
- Build on responses to uncover edge cases and constraints
