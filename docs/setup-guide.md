# Ferroway Project Infrastructure Setup Guide

*One-time setup. Follow in order.*

---

## Step 0 — Copy Project Files Into Place

Before initializing Git, copy the generated files into your local project directory.

Your local project root is:
```
C:\Users\dldur\SoftwareProjects\Ferroway_Dev\Ferroway\
```

Copy the following files from wherever Claude delivered them, maintaining the directory structure exactly:

```
Ferroway\
  README.md
  Cargo.toml
  pyproject.toml
  .gitignore
  proto\
    telemetry.proto
  infra\
    docker-compose.yml
    smoke_test.py
    docker\
      postgres-init.sql
  docs\
    canonical-schema.md
    setup-guide.md        ← this file
  cast\                   ← empty directory (create it)
  trace\                  ← empty directory (create it)
  forge\                  ← empty directory (create it)
  profiles\               ← empty directory (create it)
  signal-processing\      ← empty directory (create it)
  scenarios\              ← empty directory (create it)
  output\                 ← empty directory (create it)
```

Create the empty directories in PowerShell:

```powershell
cd C:\Users\dldur\SoftwareProjects\Ferroway_Dev\Ferroway
New-Item -ItemType Directory -Force -Path cast, trace, forge, profiles, signal-processing, scenarios, output
```

Verify the structure looks right before continuing:

```powershell
Get-ChildItem -Recurse -Depth 2
```

---

## Step 1 — GitHub Repository

### 1a. Create the GitHub repo (if not already done)

Go to [github.com/Ferroway-Dev](https://github.com/Ferroway-Dev) → New repository:
- Name: `ferroway`
- Visibility: Public (or Private — your choice)
- **Do NOT initialize with README, .gitignore, or license** — these files already exist locally and adding them here causes a conflict on first push

### 1b. Initialize the local repo

```powershell
cd C:\Users\dldur\SoftwareProjects\Ferroway_Dev\Ferroway
git init
git add .
git commit -m "FERRO-1: Initial repository structure and infrastructure files"
```

### 1c. Push to GitHub org

```powershell
git remote add origin https://github.com/Ferroway-Dev/ferroway.git
git branch -M develop
git push -u origin develop
```

If GitHub created the repo with any default files and you get a conflict error, run:

```powershell
git pull origin develop --allow-unrelated-histories
git push -u origin develop
```

### 1d. Branch protection (do after first push)

In GitHub → your repo → Settings → Branches → Add branch protection rule:

- **Branch name pattern:** `develop`
- Require pull request before merging
- Require 1 approval (can be yourself for now — good habit)
- Require status checks to pass (add once CI exists in Phase 2)

### Branching strategy

```
main       — production-ready releases only (protected, merge from develop)
develop    — primary integration branch (protected, all feature work merges here)
feature/*  — individual work branches, named with Jira ticket key
```

All day-to-day development branches from `develop` and merges back to `develop`. When a phase milestone is complete and stable, `develop` is merged to `main`. Feature branch naming convention:

```
feature/FERRO-4-manifest-parser
feature/FERRO-5-channel-engine
bugfix/FERRO-12-correlation-lag-precision
```

### 1e. Connect GitHub to Jira

The connection is made from the **Jira side**, not GitHub.

1. Go to your Jira site
2. Click **Apps** in the top navigation bar
3. Select **Explore more apps** (or go to the Atlassian Marketplace)
4. Search for **GitHub for Jira** (by Atlassian — free)
5. Click **Get app** → **Get it now**
6. Once installed, go to **Apps → GitHub for Jira**
7. Click **Connect GitHub organization**
8. Choose **GitHub Cloud**
9. Authorize Jira when prompted — this redirects to GitHub
10. In GitHub, under **Repository access**, select the `ferroway` repository
11. Click **Save** → **Connect**

Once connected, any commit or PR that includes `FERRO-{n}` in the message will automatically appear in the corresponding Jira ticket's development panel.

---

## Step 2 — Atlassian Setup

### 2a. Create Confluence space

1. Go to your Atlassian Confluence instance
2. Create Space → Blank space
3. Name: **Ferroway**
4. Key: **FERRY** (short key used in page URLs)
5. Set as private (change to public later if desired)

### 2b. Create Confluence space structure

Create these pages under the Ferroway space:

```
Ferroway (Space Home)
  ├── Architecture
  │     ├── System Architecture
  │     ├── Manifest Schema
  │     ├── Canonical Message Schema
  │     └── Phase 1 Build Sequence
  ├── Design Decisions (ADRs)
  │     ├── ADR Template
  │     ├── ADR-001: Kafka over NATS
  │     ├── ADR-002: pgvector for embeddings
  │     ├── ADR-003: Three-tier computation model
  │     ├── ADR-004: GitHub over Bitbucket
  │     ├── ADR-005: Ferroway naming and component vocabulary
  │     └── ADR-006: JSON Phase 1 → Avro Phase 2 encoding
  ├── Requirements
  │     └── Pressure Monitoring Requirements (Phase 1 demo)
  └── Demos
        └── Phase 1 End-to-End Demo
```

Upload the four Ferroway documents to the Architecture section:
- `Ferroway_SystemArchitecture_Draft.md`
- `Ferroway_Manifest_Schema_Draft.md`
- `Ferroway_Phase1_BuildSequence.md`
- `docs/canonical-schema.md`

### 2c. ADR Template

Create a page called **ADR Template** under Design Decisions with this content:

```
# ADR-{N}: {Title}

**Status:** Proposed | Accepted | Superseded
**Date:** {date}

## Context
What situation or problem prompted this decision?

## Decision
What was decided?

## Alternatives Considered
What other options were evaluated and why were they rejected?

## Consequences
What are the positive and negative results of this decision?
What does this make easier? What does this make harder?

## Related
Links to related ADRs, Jira tickets, or Confluence pages.
```

---

## Step 3 — Jira Projects

### 3a. Create FERRO project (implementation work)

1. Jira → Create Project → Scrum
2. Name: **Ferroway**
3. Key: **FERRO**
4. Connect to GitHub repo when prompted

### 3b. Create FERRODOC project (documentation work)

1. Jira → Create Project → Kanban (simpler for doc tasks)
2. Name: **Ferroway Docs**
3. Key: **FERRODOC**

### 3c. Create FERRO epics

Create these epics in the FERRO project:

| Epic Name | Key | Description |
|---|---|---|
| Infrastructure | FERRO-E1 | Repo structure, Docker Compose, canonical schema doc |
| Ferroway Cast | FERRO-E2 | Manifest parser, channel engine, transports |
| Fault and Correlation Engine | FERRO-E3 | Fault injection, correlations, derived signals |
| Ferroway Trace | FERRO-E4 | Ingestion, Waypoints, Gates |
| Ferroway Forge | FERRO-E5 | CLI: context, generate, validate |
| End-to-End Demo | FERRO-E6 | Demo recording and Confluence publish |

### 3d. Create first stories from Phase 1 Build Sequence

Create these stories under FERRO-E1:

| Story | Title |
|---|---|
| FERRO-1 | Initialize monorepo structure |
| FERRO-2 | Stand up Docker Compose infrastructure (Kafka, PostgreSQL, MinIO) |
| FERRO-2.5 | Write and freeze canonical schema decision document |
| FERRO-3 | Write proto/telemetry.proto and generate Rust + Python bindings |

---

## Step 4 — Verify Infrastructure

### 4a. Install Python dependencies for smoke test

```powershell
pip install confluent-kafka psycopg2-binary
```

### 4b. Start Docker Compose

```powershell
cd C:\Users\dldur\SoftwareProjects\Ferroway_Dev\Ferroway
docker compose -f infra/docker-compose.yml up -d
```

Wait 30 seconds for Kafka to initialize (KRaft startup is slower than ZooKeeper mode).

### 4c. Run smoke test

```powershell
python infra/smoke_test.py
```

Expected output:
```
Ferroway Infrastructure Smoke Test
========================================

Checking Kafka (KRaft)...
  ✓ Kafka — connected (KRaft mode, 0 topics)

Checking PostgreSQL + pgvector...
  ✓ PostgreSQL — connected (pgvector extension active)

Checking MinIO...
  ✓ MinIO — healthy (S3-compatible object storage)

========================================
✓ All 3 services healthy — Ferroway infrastructure ready
```

---

## Step 5 — Commit Infrastructure Setup

```powershell
git add infra/
git commit -m "FERRO-2: Docker Compose infrastructure — Kafka KRaft, PostgreSQL pgvector, MinIO"
git push
```

Verify the commit appears in the FERRO-2 Jira ticket's development panel (may take a minute for the GitHub integration to sync).

---

## What's Ready After This Setup

- ✓ GitHub org: `github.com/Ferroway-Dev`
- ✓ Local repo initialized at `C:\Users\dldur\SoftwareProjects\Ferroway_Dev\Ferroway`
- ✓ Monorepo structure in place
- ✓ Confluence space with architecture documentation
- ✓ Jira projects FERRO and FERRODOC with Phase 1 epics
- ✓ GitHub → Jira traceability chain connected
- ✓ Local infrastructure running (Kafka, PostgreSQL, MinIO)
- ✓ Canonical schema document frozen
- ✓ Proto file written

**Next step: FERRO-4 — Manifest parser in Rust (Step 4 of the Phase 1 build sequence)**

---

## Branch Naming Convention

```
feature/FERRO-4-manifest-parser
feature/FERRO-5-channel-engine
bugfix/FERRO-12-correlation-lag-precision
```

All branches cut from `develop`. PRs merge back to `develop`. `main` receives merges from `develop` at phase milestones only.

## Commit Message Convention

```
FERRO-4: Add Manifest YAML parser and JSON Schema validation

Parses profiles/heavy-equipment-engine-v1.yaml into typed Rust structs.
Validates tier separation rule at parse time — Tier-1 correlations
may only reference Tier-1 raw channel IDs.
```
