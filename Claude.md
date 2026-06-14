# Ferroway — Claude Code Project Context

This file provides persistent context for Claude Code when working in this repository.
Read this before making any changes to understand the project architecture,
naming conventions, and coding standards.

---

## What Ferroway Is

Ferroway is a **spec-driven sensor data pipeline platform for embedded engineers**.
It bridges the gap between embedded device output and cloud data pipelines — giving
embedded engineers a structured, repeatable path from edge telemetry to validated,
deployed data systems.

**GitHub org:** github.com/Ferroway-Dev
**Primary branch:** `develop`
**Work branches:** `feature/FERRO-{n}-description`
**Commit format:** `FERRO-{n}: Description`

---

## Platform Components

| Component | Directory | Language | Role |
|---|---|---|---|
| **Ferroway Cast** | `cast/` | Rust | Edge device simulator |
| **Ferroway Trace** | `trace/` | Python | Pipeline runtime |
| **Ferroway Forge** | `forge/` | Python | CLI and spec authoring tool |

### Key Vocabulary

- **Manifest** — device profile YAML (`profiles/*.yaml`)
- **Waypoints** — calculation nodes in Ferroway Trace (`trace/`)
- **Gates** — assertion checkpoints between Waypoints
- **Derived signals** — pre-computed signals (`signal-processing/*.yaml`)
- **Scenarios** — fault injection scripts (`scenarios/*.yaml`)

---

## Repository Structure

```
ferroway/
  cast/                 # Ferroway Cast — Rust
    profile/            # Manifest parser library crate
      src/
        lib.rs          # Public API re-exports
        manifest.rs     # Structs, enums, parse + validate logic
        error.rs        # ManifestError enum (thiserror)
      tests/
        parse_valid.rs  # Tests against real profile YAML
        parse_invalid.rs# Tests for error cases
    src/
      main.rs           # Cast binary entry point
    Cargo.toml          # Cast binary crate
  trace/                # Ferroway Trace — Python
  forge/                # Ferroway Forge — Python
  profiles/             # Manifest YAML files (device profiles)
  signal-processing/    # Derived signal YAML files
  scenarios/            # Fault injection scenario YAML files
  proto/                # Protobuf telemetry message contract
  infra/                # Docker Compose, smoke test
  docs/                 # Architecture, schema, ADRs
```

---

## Three-Tier Computation Model

Signal computation is organized into three tiers with explicit separation:

**Tier 1 — Manifest (`profiles/*.yaml`):**
Raw sensor channels only. Correlation drivers must be Tier-1 channel IDs.
Changes only when hardware changes.

**Tier 2 — Derived signals (`signal-processing/*.yaml`):**
Pre-computed signals mirroring ECU broadcasts.
Supported expressions: arithmetic, `delta(ch)`, `rolling_mean(ch, n=N)`.
`sample_rate_hz` must match referenced source channel rates (rate alignment rule).

**Tier 3 — Waypoints (Ferroway Trace `trace/`):**
Complex domain logic, statistical models, ML inference.

**Tier separation rule:** Tier-1 correlation `driver` fields must reference
Tier-1 raw channel IDs only. Derived signal IDs are NOT valid correlation drivers.
Enforced at parse time in `Manifest::validate()`.

---

## Canonical Message Schema

The telemetry message contract is defined in `proto/telemetry.proto`.
JSON encoding is used in Phase 1. Avro (Phase 2) is the production encoding.

**Key fields:**
- `data_type: String` — always present, carries declared channel precision
- `oneof value` — `value_f32 | value_f64 | value_i32 | value_i64 | value_u32`
- `timestamp_nanos: i64` — UTC epoch nanoseconds, never strings
- `quality: Quality` — `good | degraded | fault | dropout`
- `source: Source` — `raw | derived`

**Kafka topic pattern:** `ferroway.{domain}.{profile_id}.{channel_id}`

---

## Rust Coding Standards

### Non-negotiable rules
- **No `unwrap()` in production code.** Use `?` or explicit error handling.
  `unwrap()` / `expect()` are acceptable only in tests.
- **No `unsafe` code.** The workspace lint forbids it.
- **Document all public items** with `///` doc comments. `missing_docs` is warned.
- **All errors use `thiserror`** for libraries, `anyhow` for binary entry points.
- **`#[async_trait]`** from the async-trait crate for all async trait definitions.

### Error handling pattern
```rust
// Libraries (profile crate, future crates): typed errors with thiserror
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Failed to read '{path}': {source}")]
    FileRead { path: String, #[source] source: std::io::Error },
}

// Binary entry points (main.rs): anyhow for ergonomic propagation
fn main() -> anyhow::Result<()> { ... }
```

### Serde conventions
- Use `#[serde(rename_all = "kebab-case")]` for enums whose YAML values are kebab-case
- Use `#[serde(rename_all = "snake_case")]` for enums whose YAML values are snake_case
- Use `#[serde(rename_all = "lowercase")]` for simple lowercase enum values
- Use `#[serde(tag = "profile", rename_all = "lowercase")]` for `NoiseConfig`
  (the YAML `profile:` field discriminates the variant)
- Use `#[serde(rename = "type")]` for fields named `type` in YAML
  (reserved keyword in Rust — store as `onset_type`, `recovery_type`)

### Formatting and linting
```powershell
cargo fmt --check                                          # CI check
cargo clippy --workspace --all-targets -- -D warnings     # CI check
cargo test --workspace                                     # run tests
cargo doc --workspace --no-deps                           # verify docs
```

`rustfmt.toml` at repo root configures formatting. `max_width = 100`.

---

## Python Coding Standards

### Non-negotiable rules
- **All function signatures must be typed** — arguments and return types.
- **No implicit `Any`** — mypy runs in strict mode.
- **Waypoints** must declare `input_schema`, `output_schema`, typed `process()`,
  and a docstring describing their contract.
- **Gates** must test both passing and failing conditions in unit tests.

### Tooling
```powershell
ruff check .          # lint
ruff format .         # format
mypy .                # type check
pytest                # tests
pre-commit run --all-files  # all hooks
```

`ruff` select rules: `["E", "F", "I", "UP", "N", "ANN", "B", "SIM", "RUF"]`
`mypy` runs in strict mode. See `trace/pyproject.toml` and `forge/pyproject.toml`.

---

## Infrastructure

Local development runs via Docker Compose:

```powershell
docker compose -f infra/docker-compose.yml up -d
python infra/smoke_test.py   # verify all three services healthy
```

Services:
- **Kafka** (KRaft mode) — `localhost:9092`
- **PostgreSQL** (with pgvector) — `localhost:5432` db: ferroway user: ferroway pw: ferroway_local
- **MinIO** (S3-compatible) — `localhost:9000`

---

## Current Development State

**Active sprint:** FERRO Sprint 2
**Active story:** FERRO-11 — Manifest YAML parser
**Active branch:** `feature/FERRO-11-manifest-yaml-parser`

**Phase 1 build sequence:**
- ✓ Steps 1-4: Infrastructure, canonical schema, proto file
- 🔄 Step 4 (FERRO-11): Manifest YAML parser — IN PROGRESS
  - FERRO-28: Define Rust structs — IN PROGRESS
  - FERRO-29: Deserialize YAML into structs
  - FERRO-30: Validate tier separation rule
  - FERRO-31: Clear error messages on malformed Manifests
- ⬜ Step 5 (FERRO-12): Simulator channel engine
- ⬜ Step 6 (FERRO-13): File and Kafka transport layer
- ⬜ Steps 7a/7b: Fault injection + correlation engine
- ⬜ Step 8: Derived signals engine
- ⬜ Steps 9-11: Ferroway Trace (ingestion, Waypoints, Gates)
- ⬜ Steps 12a-12c: Ferroway Forge CLI

---

## Key Design Decisions (ADRs)

| ADR | Decision |
|---|---|
| ADR-001 | Kafka over NATS — log model, replay, fan-out |
| ADR-002 | pgvector over ChromaDB — reuses existing PostgreSQL instance |
| ADR-003 | Three-tier computation model — Manifest / derived signals / Waypoints |
| ADR-004 | GitHub over Bitbucket — ecosystem, portfolio visibility |
| ADR-005 | Ferroway naming and component vocabulary |
| ADR-006 | JSON (Phase 1) → Avro (Phase 2) message encoding |

Full ADRs in Confluence: Ferroway space → Design Decisions.

---

## Profile Files (Phase 1 Demo)

```
profiles/heavy-equipment-engine-v1.yaml           # Tier 1 — raw channels
signal-processing/heavy-equipment-engine-v1-derived.yaml  # Tier 2 — derived signals
scenarios/gradual_system_degradation.yaml         # Demo scenario
```

The Manifest parser (FERRO-11) must successfully parse
`profiles/heavy-equipment-engine-v1.yaml` and validate tier separation.

---

## Jira / Traceability

- **FERRO** — implementation work items
- **FERRODOC** — documentation and ADR work items
- Every commit: `FERRO-{n}: Description`
- Every branch: `feature/FERRO-{n}-description`
- Confluence pages linked to Jira tickets via Jira Issue macro
