  # fiber-diagnostics

  [![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](#tech-stack)
  [![Axum](https://img.shields.io/badge/API-Axum-000000.svg)](#api-overview)
  [![SQLite](https://img.shields.io/badge/DB-SQLite-07405E.svg)](#tech-stack)
  ![Status](https://img.shields.io/badge/status-functional-success.svg)

  A diagnostics backend for the **CKB Fiber Network (FNN)** that polls live
  Fiber nodes, classifies failures across **8 diagnostic categories**, and
  exposes the results through a clean **JSON API** for dashboards, CLIs, and
  operator tooling.

  ---

  ## Submission Category

  **Category 2 — Node, Routing, Cross-Chain, and Diagnostics Infrastructure**

  ---
  
  ## Demo

  - **Video walkthrough:**
  https://www.loom.com/share/9c01c9fb878942dd81f53a83721fb2b8
  - **Hosted deployment:** Not yet available
  - **Testing guide:** `docs/TESTING.md`

  ---

  ## Table of Contents

  - [Overview](#overview)
  - [Why This Project Exists](#why-this-project-exists)
  - [What Makes This Valuable](#what-makes-this-valuable)
  - [Key Features](#key-features)
  - [How It Works](#how-it-works)
  - [Architecture](#architecture)
  - [Diagnostic Categories](#diagnostic-categories)
  - [Validation and Testing](#validation-and-testing)
  - [Tech Stack](#tech-stack)
  - [Quick Start](#quick-start)
  - [API Overview](#api-overview)
  - [Current Limitations](#current-limitations)
  - [Roadmap](#roadmap)
  - [Frontend](#frontend)
  - [Why This Submission Matters](#why-this-submission-matters)
  - [Status](#status)

  ---

  ## Overview

  `fiber-diagnostics` is an infrastructure tool for **Fiber node operators** and
  **developers building on top of Fiber**.

  Its purpose is simple:

  > turn Fiber’s low-level, fragmented operational signals into actionable
  diagnostics.

  Today, diagnosing failures in FNN often requires manually correlating:

  - `node_info`
  - `list_peers`
  - `list_channels`
  - `get_payment`
  - `get_invoice`
  - gossip graph data
  - raw logs

  That process is slow, error-prone, and difficult to automate.

  `fiber-diagnostics` solves this by continuously polling one or more live Fiber
  nodes over JSON-RPC, storing their current operational state in SQLite, and
  running an ordered rule engine that converts raw state into plain-English
  diagnostic issues. Those issues are then exposed through a REST API that a
  dashboard, CLI, or other observability tooling can consume.

  ---

  ## Why This Project Exists

  One of the biggest practical problems in the current FNN operator experience
  is that payment failures are not returned as structured error codes.

  FNN’s `get_payment` RPC reports failures as:

  - `failed_error: Option<String>`

  That means the system exposes a free-text error message rather than a stable,
  machine-readable enum or code. So if an operator asks:

  > Why did this payment fail?

  there is no single RPC response that gives a structured answer.

  That creates three major problems:

  1. **Operators must manually investigate incidents**
  2. **Tooling cannot rely on stable error-code lookups**
  3. **Every dashboard or automation layer must reinvent its own diagnostic 
  logic**

  `fiber-diagnostics` closes that gap by acting as a reusable diagnostics
  backend for the Fiber ecosystem.

  ---

  ## What Makes This Valuable

  This project is valuable because it does not just display raw node state — it
  **interprets** that state.

  ### Instead of raw infrastructure signals, it provides:
  - categorized issues
  - plain-English explanations
  - API-ready diagnostic outputs
  - a foundation for dashboards and CLIs
  - a reusable backend for future Fiber observability tooling

  ### In other words:
  It turns Fiber from a system that is **inspectable only by experts** into one
  that is **operable through tooling**.

  ---

  ## Key Features

  - Polls one or more **live FNN nodes** over JSON-RPC
  - Persists current state to **SQLite**
  - Tracks:
    - node reachability
    - peer connectivity
    - channel state
    - payments
    - invoices
  - Classifies failures into **8 diagnostic categories**
  - Exposes results through a clean **Axum REST API**
  - Supports **payment origination and auto-tracking**
  - Designed for a **dashboard**, **CLI**, or external operator tools
  - Uses a **rule engine**, not a fragile one-shot lookup table
  - Handles Fiber-specific serialization edge cases such as:
    - hex-encoded amounts
    - Molecule-encoded invoice UDT attributes

  ---

  ## How It Works

  At runtime, `fiber-diagnostics` continuously collects state from monitored
  Fiber nodes, stores the latest snapshot in SQLite, and evaluates it through an
  ordered diagnostics engine.
  
  ### Polling cadence

  - **Fast loop (every 5s)**
    Polls:
    - `node_info`
    - `list_peers`
    - `list_channels`

  - **Slow loop (every 30s)**
    Polls:
    - `graph_nodes`
    - `graph_channels`

  - **Payment tracker (every 10s)**
    Polls tracked payments and invoices via:
    - `get_payment`
    - `get_invoice`

  ### Diagnostics lifecycle

  1. Poll live Fiber state
  2. Persist current-state records to SQLite
  3. Run ordered diagnostic rules
  4. Produce categorized issues
  5. Store latest issues in an in-memory cache
  6. Serve them over a JSON API

  Issues are **not persisted historically** in the shipped version. The API
  always reflects the most recent computed diagnostic view. If the underlying
  condition clears, the issue disappears on the next poll cycle.
  
  ---

  ## Architecture

  ### High-level flow

  ```text
  Fiber Nodes (node1, node2, ...)
          |
          | JSON-RPC
          v
  +-------------------------------+
  |       FiberRpcClient          |
  +-------------------------------+
          |
          +-------------------------------+
          |                               |
          v                               v
   Fast Poll Loop (5s)               Slow Poll Loop (30s)
   - node_info                       - graph_nodes
   - list_peers                      - graph_channels
   - list_channels
          |                               |
          +---------------+---------------+
                          |
                          v
               SQLite current-state tables
          +---------------+---------------+
          |                               |
          v                               v
   payment_tracker (10s)           Diagnostics engine
   - get_payment                   - ordered rules
   - get_invoice                   - 8 categories
          |                               |
          +---------------+---------------+
                          |
                          v
                 In-memory issue cache
                          |
                          v
                   Axum REST API
                          |
                          v
                 Dashboard / CLI / UI
                 
  Shared storage contract

  The project is conceptually split into two areas, with SQLite acting as the
  contract between them:

  ┌───────────────┬─────────────────────────────────────────────────────────┐
  │     Area      │                     Responsibility                      │
  ├───────────────┼─────────────────────────────────────────────────────────┤
  │ Data          │ RPC client, monitored node registry, polling loops,     │
  │ collection    │ per-node failure isolation, poll run logging            │
  ├───────────────┼─────────────────────────────────────────────────────────┤
  │ Diagnostics & │ Rule engine, payment/invoice tracking, issue            │
  │  API          │ classification, REST API contract                       │
  └───────────────┴─────────────────────────────────────────────────────────┘

  Important design decisions

  1. Rule engine, not lookup table

  Because Fiber returns unstructured failure strings, diagnostics cannot rely on
  a static enum or code-based switch statement.

  The engine therefore uses:

  - deterministic checks first
  - topology/state checks second
  - substring matching last

  This makes the classifier more robust and more aligned with how FNN behaves
  today.

  2. Per-node failure isolation

  If one monitored node goes down, it does not crash the entire poll cycle.

  That failure is treated as signal, not as a fatal collector error.

  This is important operationally: a dead node is itself a diagnostic condition.

  3. Current-state issue model

  The shipped version intentionally uses an in-memory latest-issues cache rather
  than a historical issue-events model.

  This was chosen to prioritize shipping a working diagnostics backend quickly
  and cleanly. A persisted issue-history model was prototyped during development
  but deliberately descoped from the submission version.
  
  4. Explicit handling of Fiber serialization details

  Two important Fiber-specific implementation details had to be discovered and
  handled correctly:

  - numeric amounts are serialized as hex strings
  - invoice UDT information is embedded as Molecule-encoded binary, not plain
  JSON

  These are not cosmetic details — they directly affect diagnostic correctness.

  ---
  Diagnostic Categories
  
  The engine currently classifies issues into the following 8 categories:

  ┌─────────────────────────────────┬───────────────────────────────────────┐
  │            Category             │                Meaning                │
  ├─────────────────────────────────┼───────────────────────────────────────┤
  │ Node Failure (node-down)        │ The monitored node is unreachable or  │
  │                                 │ unavailable                           │
  ├─────────────────────────────────┼───────────────────────────────────────┤
  │ Peer Connectivity              │ A required peer is disconnected or     │
  │ (peer-offline)                 │ offline                                │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Channel Readiness              │ A channel exists but is not in a       │
  │ (channel-not-ready)            │ usable/ready state                     │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Liquidity Failure              │ A payment cannot be sent because       │
  │ (insufficient-balance)         │ usable balance is insufficient         │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Invoice Failure                │ The invoice is expired or otherwise    │
  │ (invoice-expired)              │ invalid for execution                  │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Routing Failure (no-route)     │ A payment cannot find a valid route    │
  │                                │ through the network                    │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Fee / Policy Failure           │ Payment parameters violate fee or      │
  │ (fee-too-low)                  │ policy constraints                     │
  ├────────────────────────────────┼────────────────────────────────────────┤
  │ Asset Mismatch                 │ The invoice asset does not match the   │
  │ (asset-mismatch)               │ asset available on the channel         │
  └────────────────────────────────┴────────────────────────────────────────┘

  Why these categories matter

  Together, these 8 categories cover the most important operator-facing failure
  domains:

  - node health
  - peer health
  - channel usability
  - liquidity sufficiency
  - invoice validity
  - route availability
  - fee policy compliance
  - asset compatibility

  That makes the API useful not only for developers, but also for operational
  dashboards and support tooling.

  ---
  Validation and Testing
  
  This project was validated based on actual implementation and testing, not
  just intended design.

  Current validation status

  ┌──────────────────────────────────┬──────────────────────────────────────┐
  │             Category             │                Status                │
  ├──────────────────────────────────┼──────────────────────────────────────┤
  │ Node Failure (node-down)         │ Confirmed against real unreachable   │
  │                                  │ nodes                                │
  │ Channel Readiness            │ Confirmed against real channel state     │
  │ (channel-not-ready)          │                                          │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Liquidity Failure            │ Confirmed against real balances          │
  │ (insufficient-balance)       │                                          │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Invoice Failure              │ Confirmed via real and synthetic         │
  │ (invoice-expired)            │ invoices                                 │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Peer Connectivity            │ Confirmed in real time                   │
  │ (peer-offline)               │                                          │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Policy/Fee Failure           │ Rule logic confirmed with synthetic      │
  │ (fee-too-low)                │ failed-payment data                      │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Routing Failure (no-route)   │ Rule logic confirmed with synthetic data │
  ├──────────────────────────────┼──────────────────────────────────────────┤
  │ Asset Mismatch               │ Verified using a real captured invoice   │
  │ (asset-mismatch)             │ response and unit-tested Molecule        │
  │                              │ decoding                                 │
  └──────────────────────────────┴──────────────────────────────────────────┘

  Bugs discovered and fixed during development

  The validation process surfaced and fixed real implementation bugs, including:

  - wrong constant usage in channel-readiness logic
  - hex-string amount parsing bug in liquidity checks
  - invoice expiry timestamp parsing bug
  - previous always-connected peer assumption
  - incorrect asset-mismatch assumptions before Molecule decoding was introduced

  This matters because it shows the project is not just conceptually sound — it
  has already survived practical edge-case debugging.

  Documentation

  For the full testing runbook, see:

  - docs/TESTING.md

  ---
  Tech Stack
  
  - Rust 2021
  - Axum 0.8 for the HTTP API
  - Tokio for async runtime
  - sqlx 0.8 + SQLite for persistence
  - reqwest for the JSON-RPC client
  - tower-http for CORS
  - chrono, uuid, anyhow, thiserror, tracing

  RPC coverage

  The hand-rolled FiberRpcClient supports the Fiber RPC methods required by the
  diagnostics backend, including:

  - node_info
  - list_peers
  - list_channels
  - graph_nodes
  - graph_channels
  - send_payment
  - get_payment
  - parse_invoice
  - get_invoice
  - new_invoice
  - connect_peer
  - disconnect_peer
  - open_channel

  This project intentionally does not depend on an external Fiber SDK.

  ---
  Quick Start
  
  Prerequisites

  You need:

  - Rust installed
  - SQLite available locally
  - one or more running fnn nodes

  Run locally

  export DATABASE_URL="sqlite://fiber-diagnostics.db"
  cargo run

  On startup:

  - migrations are applied automatically via sqlx::migrate!
  - the API listens on http://127.0.0.1:3000

  Register a monitored node

  After startup, register a Fiber node in the database:

  sqlite3 fiber-diagnostics.db \
  "INSERT INTO monitored_nodes (id, name, rpc_url, created_at, updated_at)
   VALUES ('node1', 'node1', 'http://127.0.0.1:8227', datetime('now'), 
  datetime('now'));"

  To monitor more nodes, add additional records to monitored_nodes. No code
  changes are required.

  ---
  API Overview
  
  The backend exposes a CORS-enabled JSON API for consumption by dashboards,
  frontends, and external tooling.

  Endpoints
  
  GET /issues

  Returns the latest computed issue set.

  Supports filters such as:

  - kind
  - severity

  GET /issues/{kind}

  Returns issues for a specific diagnostic kind.

  POST /payments

  Originate a real payment and automatically register it for tracking.

  This removes the need for a manual DB registration step after dispatch.

  Response shape

  Every response includes:

  - generated_at
  - count

  The API also returns structured JSON error responses for failure cases.

  For the full request/response contract, see:

  - docs/API.md

  ---
  Current Limitations
  
  This submission is functional, but intentionally honest about current
  boundaries.

  1. Pre-flight routing failure gap

  There is a known gap around synchronous send_payment routing rejection.

  Current behavior:
  - post-dispatch routing failures are handled
  - synchronous pre-flight pathfinding failures are not yet fully captured
  end-to-end if no payment_hash is created
  
  2. Channel-side UDT assumption

  asset-mismatch currently assumes that list_channels returns
  funding_udt_type_script as a standard JSON object.

  Current status:
  - confirmed for native CKB channels
  - not yet confirmed against a real UDT-funded channel
  
  3. No persisted issue history in shipped version

  The current version serves only the latest computed issues.

  It does not yet include:
  - historical issue timelines
  - acknowledged state
  - muted state
  - resolved state
  
  Those were considered and prototyped, but intentionally descoped for the
  submission version to keep the implementation focused.

  ---
  Roadmap
  
  Planned next steps include:

  - persisted issue history with:
    - open
    - acknowledged
    - muted
    - resolved
  - full support for synchronous pre-flight routing failure capture
  - broader max_fee_amount support on POST /payments
  - real-node validation for fee-too-low and no-route
  - a CLI interface, for example:
    - fiber-diagnose watch
    - fiber-diagnose check
    - fiber-diagnose payment
    - fiber-diagnose channel
    - fiber-diagnose peers
    - fiber-diagnose node
  - a completed dashboard frontend

  ---
  Frontend
  
  A dashboard frontend is being built with Next.js.

  Local frontend setup

  cd ui
  npm install
  npm run dev

  Note

  Do not run the frontend on the same port as:

  - the backend API
  - any active Fiber node services

  Frontend status

  Frontend development is in progress.

  ---
  Why This Submission Matters
  
  fiber-diagnostics is not just a monitoring tool — it is a missing piece of
  Fiber operator infrastructure.

  It matters because it:

  - reduces operator debugging time
  - translates unstructured Fiber failures into structured diagnostics
  - provides a reusable backend for dashboards and CLIs
  - improves observability without requiring changes to FNN itself
  - creates a practical foundation for future operational tooling in the Fiber
  ecosystem

  In short:
  
  ▎ this project turns difficult-to-interpret Fiber behavior into actionable 
  ▎ infrastructure intelligence.

  That is valuable for both individual node operators and the broader ecosystem.

  ---
  Status
  
  fiber-diagnostics is currently a functional backend diagnostics engine and API
  with:

  - all 8 diagnostic categories implemented
  - a working live polling architecture
  - SQLite-backed current-state storage
  - real-node and synthetic validation across core rule paths
  - a stable API surface for a dashboard or CLI to build on

  It is already useful today, while also leaving a clear path for deeper
  observability features in future iterations.

  ---
  Documentation
  
  - docs/TESTING.md — rule-by-rule testing guide

  ---
  License
  
   MIT 
