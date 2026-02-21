# PRD Skill Examples

## Example 1: Interactive Discovery with Numbered Options

**User Request**: "Introduce async processing all over the world, especially for I/O intensive operations"

**Interactive Dialogue**:

**Q1: "What problem does this solve?"**

1. Slow response times due to blocking I/O
2. Poor resource utilization and scalability
3. System unresponsiveness under load
4. High latency for concurrent requests
5. Other (please specify)

_User responds: "1,2"_

**Q2: "Which I/O operations are the primary bottlenecks?"**

1. Database queries
2. External API calls
3. File system operations
4. Network requests
5. Message queue operations
6. Other (please specify)

_User responds: "1,2 - specifically payment gateway and user service calls"_

**Q3: "What async processing approach fits best?"**

1. Message queue (Redis, RabbitMQ, SQS)
2. Background job system (Celery, Bull, Sidekiq)
3. Event-driven architecture
4. Async/await patterns only
5. Combination (please specify)

_User responds: "2 - Bull for Node.js backend"_

**Q4: "How should errors be handled?"**

1. Retry with exponential backoff
2. Dead letter queue for failed jobs
3. Circuit breaker pattern
4. Immediate user notification
5. Combination (please specify)

_User responds: "1,2"_

**Q5: "Should existing API contracts change?"**

1. Yes, make all operations async (breaking change)
2. No, keep synchronous facade with async backend
3. Hybrid - new async endpoints + maintain old ones
4. Other (please specify)

_User responds: "3 - gradual migration strategy"_

**Persona Analysis**:

- Backend: Async patterns, execution models
- Security: Race conditions, data integrity
- Architect: System-wide impact, migration strategy

**Output**: PRD at `docs/proposals/0001_introduce_async_operation/PRD.md`

## Example 2: Interactive Product Discovery with Progressive Refinement

**User Request**: "Job-based translation request with instant HTTP response with job ID and background job processing"

**Interactive Dialogue**:

**Q1: "What translation services will be integrated?"**

1. Google Translate API
2. AWS Translate
3. DeepL API
4. Microsoft Translator
5. Multiple services (please specify)
6. Custom in-house service

_User responds: "5 - Google and DeepL with fallback logic"_

**Q2: "What's the expected job volume?"**

1. Low (<100 jobs/day)
2. Medium (100-1K jobs/day)
3. High (1K-10K jobs/day)
4. Very high (>10K jobs/day)

_User responds: "3 - around 5K jobs per day initially"_

**Q3: "How long should job results be retained?"**

1. 24 hours
2. 7 days
3. 30 days
4. Indefinitely with archiving
5. Custom (please specify)

_User responds: "3"_

**Q4: "How should clients be notified of completion?"**

1. Polling-only (client checks status)
2. Webhooks (callback URL)
3. WebSocket real-time updates
4. Email notification
5. Combination (please specify)

_User responds: "1,2 - polling with optional webhook"_

**Q5: "What retry strategy for failed translations?"**

1. No retry (fail immediately)
2. Exponential backoff (3 attempts)
3. Exponential backoff (5 attempts)
4. Retry with fallback to alternate service
5. Custom (please specify)

_User responds: "4 - try Google, fallback to DeepL"_

**Q6: "Job queue technology preference?"**

1. Redis (Bull/BullMQ)
2. RabbitMQ
3. AWS SQS
4. Kafka
5. No preference (recommend best fit)

_User responds: "5"_

**Persona Analysis**:

- **Architect**: Job queue architecture, worker pools, scaling strategy
- **Backend**: Job management API, status tracking, result retrieval
- **Frontend**: Polling mechanisms, real-time updates, error states
- **Security**: Job ownership, result access control
- **Analyser**: Feasibility, technology choices, trade-offs
- **Project Manager**: Implementation phases, milestones

**Key Decisions**:

- Recommend Bull (Redis) for volume/complexity fit
- API design validated with OpenAPI specifications
- Database schema designed for job tracking
- Scalability assessed for 5K jobs/day with growth potential

**Output**: PRD at `docs/proposals/{next_number}_job_based_translation_system/PRD.md` containing sequence diagrams, API specs, event specs, database schema, and non-functional requirements.
