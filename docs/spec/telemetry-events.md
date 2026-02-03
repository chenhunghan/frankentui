# Telemetry Event Schema + Redaction Policy

This spec defines the event schema for FrankenTUI telemetry and the redaction
policy for sensitive data. It complements `telemetry.md` (env var contract).

---

## 1) Goals

- Define a stable, explicit event schema for OTEL spans/events.
- Establish conservative default redaction of sensitive data.
- Enable consumers to build dashboards and alerts.
- Provide clear semantics for user-supplied fields.

## 2) Non-Goals

- Performance-oriented micro-tracing (covered by separate profiling).
- Complete reconstruction of UI state from telemetry.
- Real-time streaming without batching.

---

## 3) Event Categories

### 3.1 Runtime Phase Events

High-level spans for the Elm/Bubbletea runtime loop.

| Span Name | Description | Fields |
|-----------|-------------|--------|
| `ftui.program.init` | Model initialization | `model_type`, `cmd_count` |
| `ftui.program.update` | Single update cycle | `msg_type`, `duration_us`, `cmd_type` |
| `ftui.program.view` | View rendering | `duration_us`, `widget_count` |
| `ftui.program.subscriptions` | Subscription management | `active_count`, `started`, `stopped` |

### 3.2 Render Pipeline Events

Spans for the render kernel (buffer, diff, presenter).

| Span Name | Description | Fields |
|-----------|-------------|--------|
| `ftui.render.frame` | Complete frame cycle | `width`, `height`, `duration_us` |
| `ftui.render.diff` | Buffer diff computation | `changes_count`, `rows_skipped`, `duration_us` |
| `ftui.render.present` | ANSI emission | `bytes_written`, `runs_count`, `duration_us` |
| `ftui.render.flush` | Output flush | `duration_us`, `sync_mode` |
| `ftui.reflow.apply` | Resize application outcome | `width`, `height`, `debounce_ms`, `latency_ms`, `rate_hz` |
| `ftui.reflow.placeholder` | Resize placeholder shown | `width`, `height`, `rate_hz` |

### 3.3 Decision Events

Point-in-time events for auditable decisions.

| Event Name | Description | Fields |
|------------|-------------|--------|
| `ftui.decision.degradation` | Degradation level change | `level`, `reason`, `budget_remaining` |
| `ftui.decision.fallback` | Capability fallback | `capability`, `fallback_to`, `reason` |
| `ftui.decision.resize` | Resize handling decision | `strategy`, `debounce_active`, `coalesced`, `same_size`, `width`, `height`, `rate_hz` |
| `ftui.decision.screen_mode` | Screen mode selection | `mode`, `ui_height`, `anchor` |

### 3.4 Input Events

Spans for input processing (redacted by default).

| Span Name | Description | Fields |
|-----------|-------------|--------|
| `ftui.input.event` | Input event processing | `event_type` (no content!) |
| `ftui.input.macro` | Macro playback | `macro_id`, `event_count` |

---

## 4) Field Schema

### 4.1 Common Fields (All Spans)

These fields are attached to every span:

```
service.name      string   - From OTEL_SERVICE_NAME or "ftui-runtime"
service.version   string   - FrankenTUI version
telemetry.sdk     string   - "ftui-telemetry"
host.arch         string   - Target architecture
process.pid       int      - Process ID
```

### 4.2 Duration Fields

All duration fields use microseconds (us) as the unit for precision:

```
duration_us       u64      - Elapsed time in microseconds
```

### 4.3 Decision Evidence Fields

Decision events include structured evidence:

```
decision.rule      string   - Rule/heuristic applied
decision.inputs    string   - JSON-serialized input state (redacted)
decision.action    string   - Chosen action
decision.confidence f32     - Confidence score (0.0-1.0) if applicable
```

---

## 5) Redaction Policy

### 5.1 Principles

1. **Conservative by default**: Err on the side of not emitting.
2. **No PII**: Never emit user input content, file paths, or secrets.
3. **Structural only**: Emit types and counts, not values.
4. **Opt-in detail**: Verbose fields require explicit configuration.

### 5.2 Never Emit (Hard Redaction)

The following MUST never appear in telemetry:

| Category | Examples |
|----------|----------|
| **User input content** | Key characters, text buffer contents, passwords |
| **File paths** | Log files, config paths, temp files |
| **Environment variables** | Beyond OTEL_* and FTUI_* prefixes |
| **Memory addresses** | Pointer values, buffer addresses |
| **Process arguments** | Command-line arguments |
| **User identifiers** | Usernames, home directories |

### 5.3 Conditionally Emit (Soft Redaction)

These are omitted by default but can be enabled via `FTUI_TELEMETRY_VERBOSE=true`:

| Category | When Enabled |
|----------|--------------|
| **Widget types** | Full widget type names |
| **Message types** | Model::Message enum variants |
| **Command types** | Cmd enum variants |
| **Capability details** | Full terminal capability report |

### 5.4 Always Emit (No Redaction)

These are considered safe for all environments:

| Category | Examples |
|----------|----------|
| **Counts** | Widget count, change count, event count |
| **Durations** | All timing measurements |
| **Dimensions** | Buffer width/height, UI height |
| **Enum variants** | Screen mode, degradation level |
| **Boolean flags** | Mouse enabled, sync available |

---

## 6) User-Supplied Field Handling

### 6.1 Custom Span Attributes

Applications may attach custom attributes via tracing:

```rust
tracing::info_span!("my_operation", custom.field = "value");
```

**Policy:**
- Prefix requirement: Custom fields MUST use a namespace prefix (e.g., `app.`, `custom.`)
- No automatic redaction: Application is responsible for not emitting sensitive data
- Pass-through: Custom fields are passed to the OTEL exporter unchanged

### 6.2 Custom Events

Applications may emit custom events:

```rust
tracing::info!(target: "app.audit", action = "user_action");
```

**Policy:**
- Filtered by target: Only targets matching `app.*` or `custom.*` are exported
- Rate limiting: Custom events are subject to the same batching as built-in events
- Documentation: Applications should document their custom event schemas

---

## 7) Schema Versioning

### 7.1 Version Field

All telemetry includes a schema version:

```
ftui.schema_version   string   - Semantic version (e.g., "1.0.0")
```

### 7.2 Compatibility Rules

- **Patch versions** (1.0.x): Additive only, no breaking changes
- **Minor versions** (1.x.0): New fields, deprecated fields still emitted
- **Major versions** (x.0.0): Breaking changes, old fields may be removed

### 7.3 Current Schema Version

**Version: 1.0.0** (Initial stable schema)

---

## 8) Invariants (Alien Artifact)

1. **Redaction completeness**: No user input content escapes to telemetry.
2. **Schema stability**: Breaking changes require major version bump.
3. **Duration precision**: All durations use microseconds.
4. **Deterministic field set**: Same operation produces same field names.
5. **Bounded cardinality**: Enum-typed fields have known cardinality.

### Failure Modes

| Scenario | Behavior |
|----------|----------|
| Serialization error | Log warning, omit event |
| Field value overflow | Saturate to max value |
| Unknown field type | Stringify with `.to_string()` |
| Custom field collision | Prefix with `app.` |

---

## 9) Evidence Ledger Fields

For decision events, include:

```rust
pub struct DecisionEvidence {
    /// Rule or heuristic that triggered the decision
    pub rule: String,
    /// Inputs to the decision (redacted as per policy)
    pub inputs_summary: String,
    /// Chosen action
    pub action: String,
    /// Confidence (0.0-1.0) if probabilistic
    pub confidence: Option<f32>,
    /// Alternative actions considered
    pub alternatives: Vec<String>,
    /// Brief explanation for humans
    pub explanation: String,
}
```

### 9.1 Evidence Ledger JSONL (v1)

Decision evidence is also emitted as **local JSONL** for deterministic
replay and E2E verification. This ledger is distinct from OTEL spans
but uses aligned field names for easy correlation.

Each JSONL line is a single object with **shared fields** plus
**component-specific payloads**. All fields are snake_case.

**Shared fields (required for all components):**

- `schema_version`: string, **must** be `"v1"`
- `run_id`: string (UUID v4; stable per run)
- `event_idx`: u64 (monotonic per run)
- `timestamp`: ISO-8601 string (optional but recommended)
- `screen_mode`: `"inline" | "alt"`
- `buffer_cols`: u16
- `buffer_rows`: u16
- `component`: `"diff_strategy" | "resize_coalesce" | "budget_risk"`
- `strategy`: string (selected action/strategy)
- `costs`: object (expected cost/latency values)
- `thresholds`: object (budget/decision thresholds)
- `params`: object (model parameters / config)
- `terms`: object (equation terms / intermediate values)
- `rationale`: string (plain-English justification)

#### Component: `diff_strategy`

**Required fields:**

- `strategy`: `"Full" | "DirtyRows" | "FullRedraw"`
- `costs.full`, `costs.dirty`, `costs.redraw` (expected cost units)
- `params.prior_alpha`, `params.prior_beta`
- `terms.alpha`, `terms.beta`, `terms.posterior_mean`, `terms.posterior_variance`
- `terms.dirty_rows`, `terms.total_rows`, `terms.total_cells`

**Example:**

```json
{"schema_version":"v1","run_id":"1d2b2e2d-4b3f-4b75-a6ad-7b0b1b3a5b7e","event_idx":42,"screen_mode":"inline","buffer_cols":120,"buffer_rows":40,"component":"diff_strategy","strategy":"DirtyRows","costs":{"full":4800.0,"dirty":1200.0,"redraw":0.0},"thresholds":{"dirty_rows_enabled":true},"params":{"prior_alpha":1.0,"prior_beta":1.0},"terms":{"alpha":3.5,"beta":92.5,"posterior_mean":0.036,"posterior_variance":0.00034,"dirty_rows":10,"total_rows":40,"total_cells":4800},"rationale":"Dirty rows minimize expected diff cost at the observed change rate."}
```

#### Component: `resize_coalesce`

**Required fields:**

- `strategy`: `"apply" | "apply_forced" | "coalesce" | "skip_same_size"`
- `costs.time_since_render_ms`, `costs.coalesce_ms`
- `thresholds.steady_delay_ms`, `thresholds.burst_delay_ms`, `thresholds.hard_deadline_ms`
- `params.burst_enter_rate`, `params.burst_exit_rate`, `params.rate_window_size`, `params.cooldown_frames`
- `terms.dt_ms`, `terms.event_rate`, `terms.regime`
- `terms.log_bayes_factor`, `terms.regime_contribution`, `terms.timing_contribution`, `terms.rate_contribution`
- `terms.pending_w`, `terms.pending_h`, `terms.applied_w`, `terms.applied_h`, `terms.forced`

**Example:**

```json
{"schema_version":"v1","run_id":"1d2b2e2d-4b3f-4b75-a6ad-7b0b1b3a5b7e","event_idx":57,"screen_mode":"alt","buffer_cols":80,"buffer_rows":24,"component":"resize_coalesce","strategy":"coalesce","costs":{"time_since_render_ms":12.4,"coalesce_ms":24.1},"thresholds":{"steady_delay_ms":16,"burst_delay_ms":40,"hard_deadline_ms":100},"params":{"burst_enter_rate":10.0,"burst_exit_rate":5.0,"rate_window_size":8,"cooldown_frames":3},"terms":{"dt_ms":7.2,"event_rate":18.5,"regime":"burst","log_bayes_factor":-1.42,"regime_contribution":1.0,"timing_contribution":1.25,"rate_contribution":0.50,"pending_w":100,"pending_h":40,"applied_w":80,"applied_h":24,"forced":false},"rationale":"Burst regime + high event rate favors coalescing to reduce redundant redraws."}
```

#### Component: `budget_risk`

**Required fields:**

- `strategy`: `"degrade" | "upgrade" | "stay"`
- `costs.frame_time_ms`, `costs.target_ms`, `costs.remaining_ms`
- `thresholds.degrade_threshold`, `thresholds.upgrade_threshold`
- `thresholds.e_alpha`, `thresholds.e_beta`, `thresholds.warmup_frames`
- `params.pid_kp`, `params.pid_ki`, `params.pid_kd`, `params.pid_integral_max`
- `params.eprocess_lambda`, `params.eprocess_sigma`
- `terms.pid_p`, `terms.pid_i`, `terms.pid_d`, `terms.pid_output`
- `terms.e_value`, `terms.frames_observed`, `terms.frames_since_change`, `terms.in_warmup`

**Example:**

```json
{"schema_version":"v1","run_id":"1d2b2e2d-4b3f-4b75-a6ad-7b0b1b3a5b7e","event_idx":88,"screen_mode":"inline","buffer_cols":120,"buffer_rows":40,"component":"budget_risk","strategy":"degrade","costs":{"frame_time_ms":22.1,"target_ms":16.0,"remaining_ms":-6.1},"thresholds":{"degrade_threshold":0.3,"upgrade_threshold":0.2,"e_alpha":0.05,"e_beta":0.5,"warmup_frames":30},"params":{"pid_kp":0.5,"pid_ki":0.05,"pid_kd":0.2,"pid_integral_max":5.0,"eprocess_lambda":0.5,"eprocess_sigma":1.0},"terms":{"pid_p":0.19,"pid_i":0.07,"pid_d":0.05,"pid_output":0.31,"e_value":23.4,"frames_observed":64,"frames_since_change":5,"in_warmup":false},"rationale":"Sustained over-budget frames crossed PID and e-process thresholds; degrade one level."}
```

---

## 10) Implementation Notes

### 10.1 Span Attributes

Use `tracing::Span::record()` for dynamic fields:

```rust
let span = tracing::info_span!("ftui.render.frame", width = ?width, height = ?height);
let _guard = span.enter();
// ... render ...
span.record("duration_us", elapsed.as_micros() as u64);
```

### 10.2 Redaction Helper

Implement a redaction utility for consistent handling:

```rust
pub fn redact_path(path: &Path) -> &'static str {
    "[redacted:path]"
}

pub fn redact_content(content: &str) -> &'static str {
    "[redacted:content]"
}

pub fn summarize_count(items: &[T]) -> String {
    format!("{} items", items.len())
}
```

### 10.3 Verbose Mode

Check `FTUI_TELEMETRY_VERBOSE` for conditional fields:

```rust
fn is_verbose() -> bool {
    std::env::var("FTUI_TELEMETRY_VERBOSE")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
```

---

## 11) Tests

### Unit Tests

- Redaction functions return placeholder strings
- Schema version field is present
- Duration fields are u64 microseconds
- Custom field prefixing works correctly

### Property Tests

- No user input content appears in any telemetry output
- Field names are ASCII lowercase with dots
- All enum variants have known cardinality

### E2E Tests

- Capture OTEL export and verify schema compliance
- Verify redaction in verbose and non-verbose modes
- Check schema version in exported spans
