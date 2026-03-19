# cornerback

🚧 Under Construction 🚧

A backend-focused developer tool for intercepting, inspecting, replaying, and routing webhook events across multiple systems. Designed to simulate real-world integration workflows, debug failures, and handle schema evolution in event-driven architectures.

## Overview

cornerback acts as a universal webhook ingestion and debugging platform. It allows developers to:

- Receive webhooks from any external service
- Persist and inspect event payloads
- Replay events to arbitrary endpoints
- Route events to multiple downstream systems
- Validate and react to schema changes

The system is built to mirror real-world backend challenges such as integration reliability, schema drift, and event replayability.

## Core Features

### 1. Webhook Ingestion

- Dynamic endpoints: `/webhook/:id`
- Accepts arbitrary JSON payloads and headers
- Stores:
  - Headers
  - Parsed body
  - (Optionally) raw body for signature validation
  - Timestamp

### 2. Event Storage & Inspection

- Events stored in Postgres (JSONB)
- Query by webhook ID
- Inspect payloads for debugging and development

### 3. Replay System

Replay any stored event to a target endpoint: `POST /events/:eventId/replay`

**Capabilities:**

- Resend original payload and headers
- Override destination endpoint
- Capture response status and body
- Log replay attempts for observability

**Use cases:**

- Debug failed integrations
- Test new handlers with real data
- Reproduce production issues locally

### 4. Pathway Splitting (Fan-out Routing)

Forward a single incoming webhook to multiple downstream endpoints.

```
Incoming Event
     │
     ├──→ Production API
     ├──→ Debugging Endpoint
     └──→ Local Dev Server
```

**Key ideas:**

- Configurable routing rules per webhook ID
- Parallel or async delivery
- Independent success/failure handling per destination

This models real-world systems where events are consumed by multiple services simultaneously.

### 5. Schema Validation & Event Processing

Supports validating incoming events against expected schemas and reacting to changes.

**Motivation:** External APIs evolve. This system detects and handles breaking changes, additive changes, and unknown payloads.

| Scenario               | Action               |
| ---------------------- | -------------------- |
| Breaking schema change | Send to DLQ          |
| New field detected     | Trigger notification |
| Valid schema           | Process normally     |

### 6. Adapter-Based Processing

Inspired by the Adapter pattern, each integration can define its own schema, validation logic, transformation rules, and routing behavior.

```ts
interface EventAdapter {
  validate(event): ValidationResult;
  transform(event): NormalizedEvent;
  route(event): Destination[];
}
```

This allows the system to:

- Normalize disparate webhook formats
- Handle provider-specific quirks
- Evolve safely as schemas change

### 7. Dead Letter Queue (DLQ)

Events that fail validation or processing are routed to a DLQ.

**Use cases:**

- Debugging schema mismatches
- Recovering failed events
- Preventing data loss

### 8. Observability

- Replay attempt logging
- Delivery success/failure tracking
- Event history and audit trail

Future extensions: metrics (latency, success rate), alerting hooks.

## Architecture

**High-level flow:**

```
Incoming Webhook
       │
       ▼
  Ingestion API
       │
       ▼
   Event Store (Postgres)
       │
       ├──→ Schema Validation (Adapter)
       │         ├──→ Valid → Routing
       │         └──→ Invalid → DLQ
       │
       ▼
   Routing Layer (Fan-out)
       ├──→ Endpoint A
       ├──→ Endpoint B
       └──→ Endpoint C
```

**Replay flow:**

```
Stored Event → Replay API → Target Endpoint → Response Logged
```

## Tech Stack

- **Rust**
- **PostgreSQL** (JSONB)

## Future Improvements

- Async job queue for replay and routing
- Retry logic with exponential backoff
- Rate limiting and circuit breakers
- Web UI for event inspection
- Schema versioning support
- Auth & multi-tenant support
