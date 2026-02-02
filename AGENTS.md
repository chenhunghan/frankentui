# AGENTS.md — FrankenTUI (ftui)

> Guidelines for AI coding agents working in this Rust codebase.

---

## RULE NUMBER 1: NO FILE DELETION

**YOU ARE NEVER ALLOWED TO DELETE A FILE WITHOUT EXPRESS PERMISSION.** Even a new file that you yourself created, such as a test code file. You have a horrible track record of deleting critically important files or otherwise throwing away tons of expensive work. As a result, you have permanently lost any and all rights to determine that a file or folder should be deleted.

**YOU MUST ALWAYS ASK AND RECEIVE CLEAR, WRITTEN PERMISSION BEFORE EVER DELETING A FILE OR FOLDER OF ANY KIND.**

---

## Irreversible Git & Filesystem Actions — DO NOT EVER BREAK GLASS

1. **Absolutely forbidden commands:** `git reset --hard`, `git clean -fd`, `rm -rf`, or any command that can delete or overwrite code/data must never be run unless the user explicitly provides the exact command and states, in the same message, that they understand and want the irreversible consequences.
2. **No guessing:** If there is any uncertainty about what a command might delete or overwrite, stop immediately and ask the user for specific approval. "I think it's safe" is never acceptable.
3. **Safer alternatives first:** When cleanup or rollbacks are needed, request permission to use non-destructive options (`git status`, `git diff`, `git stash`, copying to backups) before ever considering a destructive command.
4. **Mandatory explicit plan:** Even after explicit user authorization, restate the command verbatim, list exactly what will be affected, and wait for a confirmation that your understanding is correct. Only then may you execute it—if anything remains ambiguous, refuse and escalate.
5. **Document the confirmation:** When running any approved destructive command, record (in the session notes / final response) the exact user text that authorized it, the command actually run, and the execution time. If that record is absent, the operation did not happen.

---

## Toolchain: Rust & Cargo

We only use **Cargo** in this project, NEVER any other package manager.

- **Edition:** Rust 2024 (nightly required — see `rust-toolchain.toml`)
- **Dependency versions:** Explicit versions for stability
- **Configuration:** Cargo.toml only
- **Unsafe code:** Forbidden (`#![forbid(unsafe_code)]`)

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `crossterm` | Terminal backend (events, raw mode, ANSI) |
| `unicode-width` | Grapheme width calculation for rendering |
| `pulldown-cmark` | GitHub-Flavored Markdown parsing |
| `tracing` | Structured logging and instrumentation |
| `insta` | Snapshot testing framework |

### Release Profile

The release build optimizes for size:

```toml
[profile.release]
opt-level = "z"     # Optimize for size (lean binary for distribution)
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit for better optimization
panic = "abort"     # Smaller binary, no unwinding overhead
strip = true        # Remove debug symbols
```

---

## Code Editing Discipline

### No Script-Based Changes

**NEVER** run a script that processes/changes code files in this repo. Brittle regex-based transformations create far more problems than they solve.

- **Always make code changes manually**, even when there are many instances
- For many simple changes: use parallel subagents
- For subtle/complex changes: do them methodically yourself

### No File Proliferation

If you want to change something or add a feature, **revise existing code files in place**.

**NEVER** create variations like:
- `mainV2.rs`
- `main_improved.rs`
- `main_enhanced.rs`

New files are reserved for **genuinely new functionality** that makes zero sense to include in any existing file. The bar for creating new files is **incredibly high**.

---

## Backwards Compatibility

We do not care about backwards compatibility—we're in early development with no users. We want to do things the **RIGHT** way with **NO TECH DEBT**.

- Never create "compatibility shims"
- Never create wrapper functions for deprecated APIs
- Just fix the code directly

---

## Compiler Checks (CRITICAL)

**After any substantive code changes, you MUST verify no errors were introduced:**

```bash
# Check for compiler errors and warnings
cargo check --all-targets

# Check for clippy lints (pedantic + nursery are enabled)
cargo clippy --all-targets -- -D warnings

# Verify formatting
cargo fmt --check
```

If you see errors, **carefully understand and resolve each issue**. Read sufficient context to fix them the RIGHT way.

---

## Testing

### Unit Tests

The workspace includes comprehensive tests across all crates:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific crate tests
cargo test -p ftui-core
cargo test -p ftui-render
cargo test -p ftui-widgets
cargo test -p ftui-runtime
```

### Snapshot Testing

FrankenTUI uses insta for visual snapshot testing:

```bash
# Run snapshot tests
cargo test -p ftui-demo-showcase

# Update snapshots (bless mode)
BLESS=1 cargo test -p ftui-demo-showcase

# Review snapshot changes
cargo insta review
```

### End-to-End Testing

```bash
# Run E2E test scripts
./scripts/e2e_test.sh
./scripts/widget_api_e2e.sh

# Run the demo showcase manually
cargo run -p ftui-demo-showcase
```

### Test Categories

| Crate | Focus |
|-------|-------|
| `ftui-core` | Terminal lifecycle, event parsing, capabilities |
| `ftui-render` | Buffer operations, diff computation, ANSI emission |
| `ftui-layout` | Constraint solving, flex/grid layout |
| `ftui-text` | Unicode width, text wrapping, grapheme handling |
| `ftui-widgets` | Widget rendering, state management |
| `ftui-runtime` | Event loop, command execution, subscriptions |
| `ftui-demo-showcase` | Snapshot tests for all demo screens |

---

## CI/CD Pipeline

### Jobs Overview

| Job | Trigger | Purpose | Blocking |
|-----|---------|---------|----------|
| `check` | PR, push | Format, clippy, tests | Yes |
| `coverage` | PR, push | Coverage thresholds | Yes |
| `snapshots` | PR, push | Visual regression testing | Yes |
| `benchmarks` | push to master | Performance budgets | Warn only |
| `e2e` | PR, push | End-to-end harness tests | Yes |

### Check Job

Runs format, clippy, and unit tests. Includes:
- `cargo fmt --check` - Code formatting
- `cargo clippy --all-targets -- -D warnings` - Lints (pedantic + nursery enabled)
- `cargo nextest run` - Full test suite with JUnit XML report

### Coverage Job

Runs `cargo llvm-cov` and enforces thresholds:
- **Overall:** ≥ 70%
- **ftui-render:** ≥ 80%
- **ftui-core:** ≥ 80%

### Snapshot Testing

Visual regression tests ensure rendering consistency:
- All demo screens tested at multiple sizes (80x24, 120x40)
- BLESS=1 to update baselines
- Changes require review before merge

### Debugging CI Failures

#### Snapshot Test Failure
1. Download snapshot artifacts
2. Compare expected vs actual renders
3. Run `BLESS=1 cargo test` locally if changes are intentional
4. Review visual diff before committing

#### Coverage Threshold Failure
1. Check which file(s) dropped below threshold in CI output
2. Run `cargo llvm-cov --html` locally to see uncovered lines
3. Add tests for uncovered code paths

---

## Third-Party Library Usage

If you aren't 100% sure how to use a third-party library, **SEARCH ONLINE** to find the latest documentation and mid-2025 best practices.

---

## FrankenTUI (ftui) — This Project

**This is the project you're working on.** FrankenTUI is a minimal, high-performance terminal UI kernel focused on correctness, determinism, and clean architecture.

### Design Philosophy

1. **Correctness over cleverness** — predictable terminal state is non-negotiable
2. **Deterministic output** — buffer diffs and explicit presentation over ad-hoc writes
3. **Inline first** — preserve scrollback while keeping chrome stable
4. **Layered architecture** — core → render → runtime → widgets, no cyclic dependencies
5. **Zero-surprise teardown** — RAII cleanup, even when apps crash

### Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                          Input Layer                              │
│   TerminalSession (crossterm) → Event (ftui-core)                 │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                          Runtime Loop                              │
│   Program/Model (ftui-runtime) → Cmd → Subscriptions              │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Render Kernel                              │
│   Frame → Buffer → BufferDiff → Presenter → ANSI                  │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                          Output Layer                              │
│   TerminalWriter (inline or alt-screen)                           │
└──────────────────────────────────────────────────────────────────┘
```

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `ftui` | Public facade + prelude |
| `ftui-core` | Terminal lifecycle, events, capabilities |
| `ftui-render` | Buffer, diff, ANSI presenter |
| `ftui-style` | Style + theme system |
| `ftui-text` | Spans, segments, rope editor |
| `ftui-layout` | Flex + Grid solvers |
| `ftui-runtime` | Elm/Bubbletea runtime |
| `ftui-widgets` | Core widget library (37 widgets) |
| `ftui-extras` | Feature-gated add-ons |
| `ftui-demo-showcase` | Reference app + snapshots |
| `ftui-harness` | Test utilities + snapshot framework |
| `ftui-pty` | PTY test utilities |

### Key Files

| File | Purpose |
|------|---------|
| `crates/ftui-core/src/terminal_session.rs` | RAII terminal lifecycle |
| `crates/ftui-render/src/buffer.rs` | 2D cell buffer with scissor stacks |
| `crates/ftui-render/src/cell.rs` | 16-byte cache-optimized Cell |
| `crates/ftui-render/src/diff.rs` | Efficient buffer diff computation |
| `crates/ftui-render/src/presenter.rs` | State-tracked ANSI emitter |
| `crates/ftui-runtime/src/program.rs` | Main event loop (Elm architecture) |
| `crates/ftui-runtime/src/terminal_writer.rs` | One-writer rule enforcement |
| `crates/ftui-widgets/src/lib.rs` | Widget trait + 37 implementations |
| `crates/ftui-demo-showcase/src/app.rs` | Demo application model |

### Core Abstractions

**Cell (16 bytes)** — Cache-line optimized for SIMD comparisons:
```rust
Cell {
    content: CellContent,  // 4 bytes - char or GraphemeId
    fg: PackedRgba,        // 4 bytes - foreground RGBA
    bg: PackedRgba,        // 4 bytes - background RGBA
    attrs: CellAttrs,      // 4 bytes - style flags + link ID
}
```

**Buffer** — 2D grid with scissor/opacity stacks:
```rust
Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,           // Row-major layout
    scissor_stack: Vec<Rect>,   // Clipping regions
    opacity_stack: Vec<f32>,    // Compositing opacity
}
```

**Model Trait** — Elm/Bubbletea architecture:
```rust
pub trait Model: Sized {
    type Message: From<Event> + Send;

    fn init(&mut self) -> Cmd<Self::Message>;
    fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message>;
    fn view(&self, frame: &mut Frame);
    fn subscriptions(&self) -> Vec<Box<dyn Subscription<Self::Message>>>;
}
```

### Key Invariants

1. **One-Writer Rule**: Only one owner of terminal output (enforced via `TerminalWriter`)
2. **Terminal State Restoration**: Guaranteed on any exit path via RAII
3. **Cell Size Fixed at 16 Bytes**: Non-negotiable for cache efficiency
4. **Buffer Dimensions Immutable**: Once created, width/height never change
5. **Scissor Stack Monotonic Intersection**: Each push intersects with current

### Screen Modes

**Inline Mode** — Preserves scrollback:
```rust
ScreenMode::Inline { ui_height: 12 }
```
- UI rendered at bottom of terminal
- Logs scroll normally above UI
- Uses cursor save/restore (DEC 7/8)

**AltScreen Mode** — Full-screen UI:
```rust
ScreenMode::AltScreen
```
- Takes over entire terminal
- No scrollback preservation

### Performance Requirements

- 16-byte Cell for SIMD comparisons
- Row-major buffer layout for cache prefetching
- State tracking in Presenter (avoid redundant escape sequences)
- 64KB buffered output (one write per frame)
- Budget system for graceful degradation

### Running the Demo

```bash
# Default (alt-screen mode)
cargo run -p ftui-demo-showcase

# Inline mode
FTUI_HARNESS_SCREEN_MODE=inline FTUI_HARNESS_UI_HEIGHT=12 cargo run -p ftui-demo-showcase

# Specific demo screen
FTUI_HARNESS_VIEW=dashboard cargo run -p ftui-demo-showcase
FTUI_HARNESS_VIEW=visual_effects cargo run -p ftui-demo-showcase
```

### Adding New Widgets

1. Create widget struct in `crates/ftui-widgets/src/`
2. Implement `Widget` or `StatefulWidget` trait
3. Add unit tests in same file
4. Export from `lib.rs`
5. Add snapshot tests in `ftui-demo-showcase`

```rust
pub trait Widget {
    fn render(&self, area: Rect, frame: &mut Frame);
    fn is_essential(&self) -> bool { false }  // Degradation support
}

pub trait StatefulWidget {
    type State;
    fn render(&self, area: Rect, frame: &mut Frame, state: &mut Self::State);
}
```

---

## MCP Agent Mail — Multi-Agent Coordination

A mail-like layer that lets coding agents coordinate asynchronously via MCP tools and resources. Provides identities, inbox/outbox, searchable threads, and advisory file reservations with human-auditable artifacts in Git.

### Why It's Useful

- **Prevents conflicts:** Explicit file reservations (leases) for files/globs
- **Token-efficient:** Messages stored in per-project archive, not in context
- **Quick reads:** `resource://inbox/...`, `resource://thread/...`

### Same Repository Workflow

1. **Register identity:**
   ```
   ensure_project(project_key=<abs-path>)
   register_agent(project_key, program, model)
   ```

2. **Reserve files before editing:**
   ```
   file_reservation_paths(project_key, agent_name, ["src/**"], ttl_seconds=3600, exclusive=true)
   ```

3. **Communicate with threads:**
   ```
   send_message(..., thread_id="FEAT-123")
   fetch_inbox(project_key, agent_name)
   acknowledge_message(project_key, agent_name, message_id)
   ```

4. **Quick reads:**
   ```
   resource://inbox/{Agent}?project=<abs-path>&limit=20
   resource://thread/{id}?project=<abs-path>&include_bodies=true
   ```

### Macros vs Granular Tools

- **Prefer macros for speed:** `macro_start_session`, `macro_prepare_thread`, `macro_file_reservation_cycle`, `macro_contact_handshake`
- **Use granular tools for control:** `register_agent`, `file_reservation_paths`, `send_message`, `fetch_inbox`, `acknowledge_message`

### Common Pitfalls

- `"from_agent not registered"`: Always `register_agent` in the correct `project_key` first
- `"FILE_RESERVATION_CONFLICT"`: Adjust patterns, wait for expiry, or use non-exclusive reservation
- **Auth errors:** If JWT+JWKS enabled, include bearer token with matching `kid`

---

## Beads (br) — Dependency-Aware Issue Tracking

Beads provides a lightweight, dependency-aware issue database and CLI (`br`) for selecting "ready work," setting priorities, and tracking status. It complements MCP Agent Mail's messaging and file reservations.

**Note:** br (beads_rust) is non-invasive and never executes git commands. After `br sync --flush-only`, you must manually run `git add .beads/` and `git commit`.

### Conventions

- **Single source of truth:** Beads for task status/priority/dependencies; Agent Mail for conversation and audit
- **Shared identifiers:** Use Beads issue ID (e.g., `br-123`) as Mail `thread_id` and prefix subjects with `[br-123]`
- **Reservations:** When starting a task, call `file_reservation_paths()` with the issue ID in `reason`

### Typical Agent Flow

1. **Pick ready work (Beads):**
   ```bash
   br ready --json  # Choose highest priority, no blockers
   ```

2. **Reserve edit surface (Mail):**
   ```
   file_reservation_paths(project_key, agent_name, ["src/**"], ttl_seconds=3600, exclusive=true, reason="br-123")
   ```

3. **Announce start (Mail):**
   ```
   send_message(..., thread_id="br-123", subject="[br-123] Start: <title>", ack_required=true)
   ```

4. **Work and update:** Reply in-thread with progress

5. **Complete and release:**
   ```bash
   br close br-123 --reason "Completed"
   ```
   ```
   release_file_reservations(project_key, agent_name, paths=["src/**"])
   ```
   Final Mail reply: `[br-123] Completed` with summary

### Mapping Cheat Sheet

| Concept | Value |
|---------|-------|
| Mail `thread_id` | `br-###` |
| Mail subject | `[br-###] ...` |
| File reservation `reason` | `br-###` |
| Commit messages | Include `br-###` for traceability |

---

## bv — Graph-Aware Triage Engine

bv is a graph-aware triage engine for Beads projects (`.beads/beads.jsonl`). It computes PageRank, betweenness, critical path, cycles, HITS, eigenvector, and k-core metrics deterministically.

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). For agent-to-agent coordination (messaging, work claiming, file reservations), use MCP Agent Mail.

**CRITICAL: Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.**

### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command
```

### Command Reference

**Planning:**
| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with `unblocks` lists |
| `--robot-priority` | Priority misalignment detection with confidence |

**Graph Analysis:**
| Command | Returns |
|---------|---------|
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core, articulation points, slack |
| `--robot-label-health` | Per-label health: `health_level`, `velocity_score`, `staleness`, `blocked_count` |
| `--robot-label-flow` | Cross-label dependency: `flow_matrix`, `dependencies`, `bottleneck_labels` |
| `--robot-label-attention [--attention-limit=N]` | Attention-ranked labels |

**History & Change Tracking:**
| Command | Returns |
|---------|---------|
| `--robot-history` | Bead-to-commit correlations |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues, cycles |

**Other:**
| Command | Returns |
|---------|---------|
| `--robot-burndown <sprint>` | Sprint burndown, scope changes, at-risk items |
| `--robot-forecast <id\|all>` | ETA predictions with dependency-aware scheduling |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |
| `--export-graph <file.html>` | Interactive HTML visualization |

### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank
bv --robot-triage --robot-triage-by-track    # Group by parallel work streams
bv --robot-triage --robot-triage-by-label    # Group by domain
```

### Understanding Robot Output

**All robot JSON includes:**
- `data_hash` — Fingerprint of source beads.jsonl
- `status` — Per-metric state: `computed|approx|timeout|skipped` + elapsed ms
- `as_of` / `as_of_commit` — Present when using `--as-of`

**Two-phase analysis:**
- **Phase 1 (instant):** degree, topo sort, density
- **Phase 2 (async, 500ms timeout):** PageRank, betweenness, HITS, eigenvector, cycles

### jq Quick Reference

```bash
bv --robot-triage | jq '.quick_ref'                        # At-a-glance summary
bv --robot-triage | jq '.recommendations[0]'               # Top recommendation
bv --robot-plan | jq '.plan.summary.highest_impact'        # Best unblock target
bv --robot-insights | jq '.status'                         # Check metric readiness
bv --robot-insights | jq '.Cycles'                         # Circular deps (must fix!)
```

---

## UBS — Ultimate Bug Scanner

**Golden Rule:** `ubs <changed-files>` before every commit. Exit 0 = safe. Exit >0 = fix & re-run.

### Commands

```bash
ubs file.rs file2.rs                    # Specific files (< 1s) — USE THIS
ubs $(git diff --name-only --cached)    # Staged files — before commit
ubs --only=rust,toml src/               # Language filter (3-5x faster)
ubs --ci --fail-on-warning .            # CI mode — before PR
ubs .                                   # Whole project (ignores target/, Cargo.lock)
```

### Output Format

```
⚠️  Category (N errors)
    file.rs:42:5 – Issue description
    💡 Suggested fix
Exit code: 1
```

Parse: `file:line:col` → location | 💡 → how to fix | Exit 0/1 → pass/fail

### Fix Workflow

1. Read finding → category + fix suggestion
2. Navigate `file:line:col` → view context
3. Verify real issue (not false positive)
4. Fix root cause (not symptom)
5. Re-run `ubs <file>` → exit 0
6. Commit

### Bug Severity

- **Critical (always fix):** Memory safety, use-after-free, data races, SQL injection
- **Important (production):** Unwrap panics, resource leaks, overflow checks
- **Contextual (judgment):** TODO/FIXME, println! debugging

---

## ast-grep vs ripgrep

**Use `ast-grep` when structure matters.** It parses code and matches AST nodes, ignoring comments/strings, and can **safely rewrite** code.

- Refactors/codemods: rename APIs, change import forms
- Policy checks: enforce patterns across a repo
- Editor/automation: LSP mode, `--json` output

**Use `ripgrep` when text is enough.** Fastest way to grep literals/regex.

- Recon: find strings, TODOs, log lines, config values
- Pre-filter: narrow candidate files before ast-grep

### Rule of Thumb

- Need correctness or **applying changes** → `ast-grep`
- Need raw speed or **hunting text** → `rg`
- Often combine: `rg` to shortlist files, then `ast-grep` to match/modify

### Rust Examples

```bash
# Find structured code (ignores comments)
ast-grep run -l Rust -p 'fn $NAME($$$ARGS) -> $RET { $$$BODY }'

# Find all unwrap() calls
ast-grep run -l Rust -p '$EXPR.unwrap()'

# Quick textual hunt
rg -n 'println!' -t rust

# Combine speed + precision
rg -l -t rust 'unwrap\(' | xargs ast-grep run -l Rust -p '$X.unwrap()' --json
```

---

## Morph Warp Grep — AI-Powered Code Search

**Use `mcp__morph-mcp__warp_grep` for exploratory "how does X work?" questions.** An AI agent expands your query, greps the codebase, reads relevant files, and returns precise line ranges with full context.

**Use `ripgrep` for targeted searches.** When you know exactly what you're looking for.

**Use `ast-grep` for structural patterns.** When you need AST precision for matching/rewriting.

### When to Use What

| Scenario | Tool | Why |
|----------|------|-----|
| "How is the render pipeline implemented?" | `warp_grep` | Exploratory; don't know where to start |
| "Where is the diff computation?" | `warp_grep` | Need to understand architecture |
| "Find all uses of `Buffer::new`" | `ripgrep` | Targeted literal search |
| "Find files with `unwrap()`" | `ripgrep` | Simple pattern |
| "Replace all `unwrap()` with `expect()`" | `ast-grep` | Structural refactor |

### warp_grep Usage

```
mcp__morph-mcp__warp_grep(
  repoPath: "/path/to/frankentui",
  query: "How does the buffer diff algorithm work?"
)
```

Returns structured results with file paths, line ranges, and extracted code snippets.

### Anti-Patterns

- **Don't** use `warp_grep` to find a specific function name → use `ripgrep`
- **Don't** use `ripgrep` to understand "how does X work" → wastes time with manual reads
- **Don't** use `ripgrep` for codemods → risks collateral edits

<!-- bv-agent-instructions-v1 -->

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) for issue tracking. Issues are stored in `.beads/` and tracked in git.

### Essential Commands

```bash
# View issues (launches TUI - avoid in automated sessions)
bv

# CLI commands for agents (use these instead)
br ready              # Show issues ready to work (no blockers)
br list --status=open # All open issues
br show <id>          # Full issue details with dependencies
br create --title="..." --type=task --priority=2
br update <id> --status=in_progress
br close <id> --reason="Completed"
br close <id1> <id2>  # Close multiple issues at once
br sync --flush-only  # Flush changes to .beads/ (does NOT run git)
```

### Workflow Pattern

1. **Start**: Run `br ready` to find actionable work
2. **Claim**: Use `br update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id>`
5. **Sync**: Always run `br sync --flush-only` then `git add .beads/ && git commit` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers, not words)
- **Types**: task, bug, feature, epic, question, docs
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Session Protocol

**Before ending any session, run this checklist:**

```bash
git status              # Check what changed
git add <files>         # Stage code changes
br sync --flush-only    # Flush beads changes to .beads/
git add .beads/         # Stage beads changes
git commit -m "..."     # Commit code and beads together
git push                # Push to remote
```

### Best Practices

- Check `br ready` at session start to find available work
- Update status as you work (in_progress → closed)
- Create new issues with `br create` when you discover tasks
- Use descriptive titles and set appropriate priority/type
- Always `br sync --flush-only` then `git add .beads/` before ending session

<!-- end-bv-agent-instructions -->

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   br sync --flush-only    # Flush beads changes to .beads/
   git add .beads/         # Stage beads changes
   git commit -m "Update beads" --allow-empty
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds


---

Note for Codex/GPT-5.2:

You constantly bother me and stop working with concerned questions that look similar to this:

```
Unexpected changes (need guidance)

- Working tree still shows edits I did not make in Cargo.toml, Cargo.lock, src/cli/commands/upgrade.rs, src/storage/sqlite.rs, tests/conformance.rs, tests/storage_deps.rs. Please advise whether to keep/commit/revert these before any further work. I did not touch them.

Next steps (pick one)

1. Decide how to handle the unrelated modified files above so we can resume cleanly.
2. Triage beads_rust-orko (clippy/cargo warnings) and beads_rust-ydqr (rustfmt failures).
3. If you want a full suite run later, fix conformance/clippy blockers and re‑run cargo test --all.
```

NEVER EVER DO THAT AGAIN. The answer is literally ALWAYS the same: those are changes created by the potentially dozen of other agents working on the project at the same time. This is not only a common occurence, it happens multiple times PER MINUTE. The way to deal with it is simple: you NEVER, under ANY CIRCUMSTANCE, stash, revert, overwrite, or otherwise disturb in ANY way the work of other agents. Just treat those changes identically to changes that you yourself made. Just fool yourself into think YOU made the changes and simply don't recall it for some reason.
