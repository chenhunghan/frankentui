# State Machines: Terminal + Rendering Pipeline

This document is the formal-ish specification backbone for FrankenTUI.

It is intentionally written to be directly useful for:
- implementation structure (which module owns what)
- invariant placement (type-level vs runtime checks)
- test strategy (unit/property/PTY)

See Bead: bd-10i.13.1.

---

## 1) Terminal State Machine

### 1.1 State Variables
We model the terminal as a state machine that consumes bytes and updates a display grid.

Minimal state (conceptual):
- `cursor`: (x, y)
- `style`: current SGR state (fg/bg/attrs)
- `grid`: a width×height array of `Cell`
- `mode`: Normal | Raw | AltScreen

ftui-specific derived state:
- `link_state`: OSC 8 hyperlink open/close tracking
- `cursor_visible`: bool
- `sync_output`: bool (DEC 2026 nesting/active)
- `scroll_region`: optional (top..bottom) margins

### 1.2 Safety Invariants
- Cursor bounds: `0 <= x < width`, `0 <= y < height`.
- Grid validity: every cell is a valid `Cell` value.
- Mode cleanup: on exit, Raw/AltScreen/mouse/paste/focus modes are restored to safe defaults.

### 1.3 Where This Is Enforced
Type-level (compile-time-ish):
- `TerminalSession` owns terminal lifecycle so that cleanup cannot be “forgotten”.

Runtime checks:
- bounds checks on cursor moves (or explicit clamping policy)
- internal assertions in debug builds for invariants

Tests:
- PTY tests validate cleanup invariants under normal exit + panic.

Implementation module targets (will be updated as code lands):
- Terminal lifecycle + cleanup: `crates/ftui-core/src/terminal_session.rs`
- Capability model: `crates/ftui-core/src/terminal_capabilities.rs`

---

## 2) Rendering Pipeline State Machine

### 2.1 States
States (from plan):
- Idle
- Measuring
- Rendering
- Diffing
- Presenting
- Error

### 2.2 Transitions
- Idle → Measuring (render request)
- Measuring → Rendering (layout complete)
- Rendering → Diffing (draw complete)
- Diffing → Presenting (diff computed)
- Presenting → Idle (present complete)
- * → Error (I/O error, internal invariant violation)
- Error → Idle (recover)

### 2.3 Pipeline Invariants
I1. In Rendering state, only the back buffer is modified.
I2. In Presenting state, only ANSI output is produced.
I3. After Presenting, front buffer equals desired grid.
I4. Error state restores terminal to a safe state.
I5. Scissor stack intersection monotonically decreases on push.
I6. Opacity stack product stays in [0, 1].

### 2.4 Where This Is Enforced
Type-level:
- Separate “front” vs “back” buffers owned by Frame/Presenter APIs.

Runtime checks:
- scissor stack push/pop asserts intersection monotonicity in debug
- opacity stack push/pop clamps and asserts range

Tests:
- executable invariant tests (bd-10i.13.2)
- property tests for diff correctness (bd-2x0j)
- terminal-model presenter roundtrip tests (bd-10i.11.1)

Implementation module targets (will be updated as code lands):
- Buffer/Cell invariants: `crates/ftui-render/src/buffer.rs`, `crates/ftui-render/src/cell.rs`
- Diff engine: `crates/ftui-render/src/diff.rs`
- Presenter: `crates/ftui-render/src/presenter.rs`

---

## 3) Responsive Reflow Spec (Resize / Relayout)

This spec defines the invariants and observable behavior for resize-driven
reflow in FrankenTUI. It is deliberately testable: every rule below maps to
an instrumentation point and at least one unit/property/E2E test scenario.

### 3.1 Goals
- Continuous reflow during resize storms without flicker or ghosting.
- Atomic present: a frame is either fully correct for a given size or not shown.
- No placeholders: never show partial layouts or "blank" regions while reflowing.
- Bounded latency: reflow settles within a defined SLA after the final resize event.

### 3.2 Non-Goals
- Perfect visual smoothness on terminals that lack sync output (DEC 2026).
- Per-terminal pixel-perfect behavior (we operate in cell space).
- GPU-driven animation during resize (CPU baseline only).

### 3.3 Invariants (Must Hold)
R1. **Atomic present**: A present must correspond to exactly one (width, height)
    pair and a fully rendered buffer at that size. Partial or mixed-size output
    is forbidden.
R2. **No placeholder frames**: During a resize storm, either keep the last stable
    frame or present a fully reflowed frame. Never show “empty” filler.
R3. **Final-size convergence**: After the last resize event, the next present
    reflects the final size (no “lagging” intermediate size).
R4. **Inline anchor correctness**: In inline mode, the UI anchor and reserved
    height are recomputed on every size change (no fixed anchor during resize).
R5. **Shrink cleanup**: When the terminal shrinks, output must not leave stale
    glyphs beyond the new bounds (explicit clears or full redraw).
R6. **One-writer rule**: All output (logs + UI) must be serialized via
    `TerminalWriter`, even during resize handling.

### 3.4 Latency SLA
- **Target (p95):** ≤ 120 ms from final resize event → first stable present.
- **Hard cap (p99):** ≤ 250 ms (violations are test failures).
- **Degraded mode:** If the system is over budget, it must drop intermediate
  sizes and jump directly to the final size (still obeying R1–R6).

### 3.5 Atomic Present Rules
1. When a resize event arrives, invalidate the previous buffer and mark a
   reflow-required flag.
2. Do not present until a full layout + render pass completes for the new size.
3. On resize, perform a **full redraw** (diff against empty) to guarantee
   shrink cleanup and eliminate ghosting.
4. Present and flush exactly once for that size; do not interleave logs mid-frame.
5. After present, the front buffer equals the desired grid for that size.

Implementation note (current):
- `Program` currently renders a "Resizing..." placeholder during debounce
  (`crates/ftui-runtime/src/program.rs`). This violates R2 and must be removed
  before the spec is considered fully compliant.

### 3.6 Decision Rule (Resize Coalescing)
Current rule (deterministic, explainable):
- On resize event: update `pending_size` and `last_resize_ts`.
- On tick: if `now - last_resize_ts >= debounce`, apply `pending_size`.
- Always apply the latest size; ignore duplicates.

Evidence ledger / Bayes factor sketch (for future adaptive tuning):
- Hypotheses: `H_ongoing` (user still resizing) vs `H_settled` (resize complete).
- Evidence: inter-arrival time `dt`.
- Bayes factor (simple): `BF_settled = exp(dt / debounce)`, `BF_ongoing = exp(-dt / debounce)`.
- Decision: apply when `BF_settled > BF_ongoing` (equivalently `dt >= debounce`).

### 3.7 Failure Modes (Ledger)
- **Ghosting on shrink**: stale cells remain outside new bounds.
- **Flicker**: partial frame or cursor jumps during reflow.
- **Anchor drift**: inline UI region fails to re-anchor, overwriting logs.
- **Resize lag**: presents intermediate size after final resize.
- **Write interleaving**: log output interspersed with UI present.

For any heuristic (e.g., coalescing/pace control), record an evidence ledger:
- Input signals used (event rate, size delta, time since last present).
- Decision taken (coalesce vs render now).
- Expected impact (latency vs correctness).

### 3.8 Instrumentation Points (Required)
These points MUST emit structured JSONL entries:
1. **Resize event ingress** (raw event read).
2. **Coalescer output** (final size chosen, events collapsed).
3. **Reflow start** (layout + render begin).
4. **Diff stats** (cells changed, runs, bytes).
5. **Present end** (flush complete).
6. **Stability marker** (first stable frame after last resize).

### 3.9 JSONL Log Fields (Required)
Each entry must include:
- `ts_ms`, `event_id`, `phase`
- `cols`, `rows`
- `mode` (`inline` | `alt`)
- `ui_height`, `ui_anchor`
- `coalesced_events`, `coalesce_window_ms`
- `frame_id`, `frame_duration_ms`
- `diff_cells`, `diff_runs`, `present_bytes`
- `sla_budget_ms`, `sla_violation` (bool)
- `ghost_detected` (bool), `flicker_detected` (bool)

### 3.10 Test Plan (Required)
Unit + property tests:
- Event coalescing: idempotence and monotonic convergence to final size.
- Anchor recompute: inline UI start row matches new terminal height.
- Atomic present: no output emitted while in "reflowing" state.
- Shrink cleanup: no cells remain outside new bounds after present.
- Regression fixtures: capture historical resize/flicker bugs with fixed seeds + golden hashes.

E2E PTY scenarios (JSONL logging required):
- **Resize storm**: 5–10 rapid size changes, verify final size + SLA.
- **Shrink → expand**: verify no ghosting after shrink.
- **Inline mode**: logs + UI + resize; verify cursor save/restore and anchor.
- **Alt screen**: no scrollback leakage; present is atomic.
- **Rapid mode switch**: toggle inline/alt during resize; ensure no partial frames.
- **Deterministic mode**: fixed seed + timing controls; logs include seed + checksum.

Instrumentation requirements for tests:
- Capture resize events, frame timestamps, diff sizes.
- Emit flicker/ghosting checksums (per-frame hash of final buffer).

### 3.11 Optimization Protocol (Required)
If performance changes are introduced as part of reflow work:
- **Baseline**: measure p50/p95/p99 reflow latency with `hyperfine` and record raw output.
- **Profile**: collect CPU + allocation profiles; identify top 5 hotspots.
- **Opportunity matrix**: only implement changes with Score ≥ 2.0 (Impact×Confidence/Effort).
- **Isomorphism proof**: prove ordering/tie-breaking/seed stability and record golden checksums.

---

## 4) Notes for Contributors

- The goal is not “perfect formalism”; the goal is to prevent drift.
- If you change behavior in Buffer/Presenter/TerminalSession, update this document and add tests.
