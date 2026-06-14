# Ferroway

**A spec-driven sensor data pipeline platform for embedded engineers.**

Ferroway bridges the gap between embedded device output and cloud data pipelines — giving embedded engineers a structured, repeatable path from edge telemetry to validated, deployed data systems.

---

## Platform Components

| Component | Directory | Language | Description |
|---|---|---|---|
| **Ferroway Forge** | `forge/` | Python | CLI and spec authoring tool — define pipeline specs, retrieve requirements context, generate and validate Trace pipelines |
| **Ferroway Trace** | `trace/` | Python | Pipeline runtime — ingests telemetry, routes through Waypoints and Gates, writes to output sinks |
| **Ferroway Cast** | `cast/` | Rust | Edge device simulator — emits configurable, realistic sensor telemetry from Manifests |

## Key Concepts

- **Manifest** — declarative device profile describing sensor channels, correlations, and fault patterns
- **Waypoints** — typed calculation nodes data passes through (unit conversion, signal filtering, anomaly scoring)
- **Gates** — assertion checkpoints between Waypoints that validate invariants before the next stage fires
- **Pipeline Spec** — the source of truth; code is a build artifact

---

## Repository Structure

```
ferroway/
  cast/               # Ferroway Cast — Rust edge device simulator
  trace/              # Ferroway Trace — Python pipeline runtime
  forge/              # Ferroway Forge — Python CLI and spec authoring tool
  profiles/           # YAML Manifests — device profile definitions
  signal-processing/  # YAML derived signal definitions
  scenarios/          # YAML fault injection scenarios
  proto/              # Protobuf message contracts (shared)
  infra/              # Docker Compose, later Terraform
  docs/               # Architecture, schema, ADRs, canonical schema
```

---

## Getting Started (Phase 1 — Local Development)

### Prerequisites

- Rust (stable) — `rustup install stable`
- Python 3.11+ with `uv` or `pip`
- Docker Desktop
- Protocol Buffer compiler — `protoc`

### Start the local infrastructure

```bash
cd infra
docker compose up -d
```

This starts:
- **Kafka** (KRaft mode) on `localhost:9092`
- **PostgreSQL** (with pgvector) on `localhost:5432`
- **MinIO** (S3-compatible) on `localhost:9000`

### Run the simulator

```bash
cd cast
cargo run -- \
  --profile ../profiles/heavy-equipment-engine-v1.yaml \
  --derived ../signal-processing/heavy-equipment-engine-v1-derived.yaml \
  --scenario ../scenarios/gradual_system_degradation.yaml \
  --transport kafka \
  --transport file --file-path ../output/session-001.jsonl
```

### Use Ferroway Forge

```bash
cd forge
pip install -e .

ferroway context add ../docs/requirements/pressure_monitoring.md
ferroway generate --output ../trace/generated/pressure_monitor.py
ferroway validate ../trace/generated/pressure_monitor.py
```

---

## Documentation

Full architecture documentation lives in `docs/` and is mirrored in Confluence.

- [System Architecture](docs/architecture/system-architecture.md)
- [Manifest Schema](docs/architecture/manifest-schema.md)
- [Canonical Message Schema](docs/architecture/canonical-schema.md)
- [Phase 1 Build Sequence](docs/architecture/phase1-build-sequence.md)

---

## Development Workflow

Branch naming: `feature/FERRO-{n}-short-description`
Commit format: `FERRO-{n}: Description of change`

Jira project: **FERRO** (implementation) | **FERRODOC** (documentation)
GitHub org: [Ferroway-Dev](https://github.com/Ferroway-Dev)

---

*Ferroway — Phase 1 in active development.*
