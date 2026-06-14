# Ferroway Canonical Message Schema
*Step 2.5 — frozen before proto bindings are generated*

---

## Source of Truth

`proto/telemetry.proto` is the single canonical source of truth for the Ferroway message contract.

The JSON wire encoding is a **defined serialization of the proto struct** — not an independent schema. Any conflict between this document and the proto file should be resolved in favour of the proto file.

---

## Field Definitions

| Field | Proto type | JSON field | Notes |
|---|---|---|---|
| `device_id` | `string` | `device_id` | Unique identifier for this device instance |
| `profile_id` | `string` | `profile_id` | Manifest profile_id this device is running |
| `channel_id` | `string` | `channel_id` | Channel identifier from the Manifest |
| `timestamp_nanos` | `int64` | `timestamp_nanos` | UTC epoch nanoseconds — no string parsing in hot path |
| `unit` | `string` | `unit` | Unit of measure — matches Manifest channel declaration |
| `quality` | `Quality` enum | `quality` (lowercase string) | Signal quality flag |
| `source` | `Source` enum | `source` (lowercase string) | Raw sensor or Cast-computed derived signal |
| `data_type` | `string` | `data_type` | Declared channel data_type from Manifest — always present |
| `value` | `oneof` | `value_f32` / `value_f64` / `value_i32` / `value_i64` / `value_u32` | Only one present per message |

---

## data_type → value_* Mapping

| Manifest `data_type` | Proto oneof field | JSON field name |
|---|---|---|
| `float32` | `float value_f32` | `value_f32` |
| `float64` | `double value_f64` | `value_f64` |
| `int16` | `int32 value_i32` | `value_i32` |
| `int32` | `int32 value_i32` | `value_i32` |
| `int64` | `int64 value_i64` | `value_i64` |
| `uint16` | `uint32 value_u32` | `value_u32` |
| `uint32` | `uint32 value_u32` | `value_u32` |

The `data_type` field is **always present** on every message. Consumers recover the declared precision without profile lookup. The Phase 2 Avro schema generator reads `data_type` directly.

---

## Enum Casing Rules

| Enum | Proto values | JSON values |
|---|---|---|
| `Quality` | `GOOD`, `DEGRADED`, `FAULT`, `DROPOUT` | `good`, `degraded`, `fault`, `dropout` |
| `Source` | `RAW`, `DERIVED` | `raw`, `derived` |

Proto values are uppercase per Protobuf convention. JSON serialization uses lowercase strings. The mapping is explicit and tested in the Ferroway Trace ingestion layer.

---

## Timestamp

- **Wire format:** `int64` epoch nanoseconds (UTC)
- **ISO8601 formatting:** at output sinks only — never on the wire
- **Rationale:** Avoids string parsing in the ingestion hot path. Nanoseconds are directly comparable, subtractable, and usable in rolling window calculations.

---

## Ingestion Normalization Rule

The Ferroway Trace ingestion layer reads the populated `value_*` field and normalizes to **Python `float` (float64)** before passing to Waypoints. Type conversion happens once at ingestion. Waypoints are type-agnostic.

The deserializer must handle **all five** `value_*` variants and both `source` values from the first line of deserialization code — not incrementally as new message types are introduced.

---

## JSON Wire Example (Phase 1 encoding)

```json
{
  "device_id": "cast-sim-001",
  "profile_id": "heavy-equipment-engine-v1",
  "channel_id": "engine_rpm",
  "timestamp_nanos": 1749823201000000000,
  "unit": "rpm",
  "quality": "good",
  "source": "raw",
  "data_type": "float32",
  "value_f32": 1450.0
}
```

```json
{
  "device_id": "cast-sim-001",
  "profile_id": "heavy-equipment-engine-v1",
  "channel_id": "engine_fault_code",
  "timestamp_nanos": 1749823201000000000,
  "unit": "none",
  "quality": "good",
  "source": "raw",
  "data_type": "uint16",
  "value_u32": 0
}
```

```json
{
  "device_id": "cast-sim-001",
  "profile_id": "heavy-equipment-engine-v1",
  "channel_id": "engine_load",
  "timestamp_nanos": 1749823201500000000,
  "unit": "percent",
  "quality": "good",
  "source": "derived",
  "data_type": "float32",
  "value_f32": 56.8
}
```

---

## Kafka Topic Pattern

```
ferroway.{domain}.{profile_id}.{channel_id}
```

Example: `ferroway.vehicular.heavy-equipment-engine-v1.engine_rpm`

Consumers use wildcard subscriptions:
- All vehicular channels: `ferroway.vehicular.#`
- All channels for a profile: `ferroway.vehicular.heavy-equipment-engine-v1.#`
- Specific channel across profiles: `ferroway.vehicular.*.engine_rpm`

---

## Phase 2 Migration

Phase 1 encoding is JSON. Phase 2 migrates to **Avro with AWS Glue Schema Registry**.

The migration is contained to:
1. Ferroway Cast (Rust simulator) — swap JSON serializer for Avro
2. Ferroway Trace ingestion layer — swap JSON deserializer for Avro

No changes to Waypoints, Gates, or downstream components.

---

*This document is frozen. Changes require updating proto/telemetry.proto first, then this document.*
