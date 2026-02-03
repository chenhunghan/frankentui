# Code Review Report - FrankenTUI

**Date:** 2026-02-03
**Reviewer:** Gemini CLI (Code Review Agent)

## Executive Summary

A comprehensive deep-dive code review was performed on the **FrankenTUI (ftui)** codebase, focusing on architectural integrity, correctness, rendering logic, and widget implementation. The review covered core crates (`ftui-core`, `ftui-render`, `ftui-layout`, `ftui-style`, `ftui-text`) and the widget library (`ftui-widgets`).

**Overall Status:** The codebase is robust, well-tested, and adheres to its architectural specifications. One visual rendering bug was identified and fixed.

## Scope

The following areas were audited:

1.  **Core Architecture**: `ftui-core` (Lifecycle, Input), `ftui-render` (Buffer, Diff, ANSI).
2.  **Layout Engine**: `ftui-layout` (Flex, Constraints).
3.  **Text & Styling**: `ftui-text` (Wrapping, Editor), `ftui-style` (Cascading styles).
4.  **Widgets**:
    *   `Scrollbar` (Bug found & fixed)
    *   `Table` (Integer truncation fixed)
    *   `List` (Integer truncation fixed)
    *   `Tree`
    *   `TextArea`
    *   `Input`
    *   `ProgressBar`
    *   `Block`
    *   `Paragraph`
    *   `VirtualizedList`

## Findings & Fixes

### 1. Bug Fix: Scrollbar Wide-Character Corruption

**Issue:**
The `Scrollbar` widget's rendering loop iterated by cell index (`i`), drawing a symbol at each position. When using wide Unicode characters (e.g., emojis "🔴", "👍") for the track or thumb, drawing a symbol at index `i` would populate cells `i` and `i+1`. The subsequent iteration at `i+1` would then overwrite the "tail" of the previous wide character with a new "head", resulting in visual corruption.

**Fix:**
Modified the `render` method in `crates/ftui-widgets/src/scrollbar.rs` to conditionally skip iteration indices based on the drawn symbol's width and orientation:
*   **Horizontal:** The loop now skips `symbol_width` cells after drawing, preserving wide characters.
*   **Vertical:** The loop continues to increment by 1 (row), as wide characters stack vertically without overlapping.

**Verification:**
Added two regression tests to `crates/ftui-widgets/src/scrollbar.rs`:
*   `scrollbar_wide_symbols_horizontal`: Verifies contiguous wide character rendering.
*   `scrollbar_wide_symbols_vertical`: Verifies vertical stacking of wide characters.

### 2. Codebase Health

*   **Render Kernel**: `Buffer` correctly handles atomic wide-character writes. `BufferDiff` and `Presenter` are optimized and correct.
*   **Layout**: `Flex` solver handles division-by-zero and overflow edge cases gracefully.
*   **Input**: `InputParser` includes DoS protection and robust state machine logic for ANSI sequences.
*   **Text Editing**: `Editor` (and `TextArea`) uses a `Rope` structure with grapheme-aware cursors, ensuring Unicode correctness.
*   **Virtualization**: `VirtualizedList` and `FenwickTree` implement efficient O(log n) scrolling for variable-height items.

## Session 2 Findings

### 3. Bug Fix: Input Fairness Guard Logic

**Issue:**
In `crates/ftui-runtime/src/input_fairness.rs`, the `check_fairness` method always returned `should_process: true`, even when it determined that intervention was required (`yield_to_input: true`). This effectively disabled the starvation protection mechanism in `program.rs`.

**Fix:**
Updated `check_fairness` to set `should_process` to `!yield_to_input` in the `FairnessDecision` construction. This ensures that when `yield_to_input` is true (indicating input starvation risk), `should_process` becomes false, correctly signaling the runtime to skip the current resize event.

### 4. Bug Fix: Render Thread Memory Leak

**Issue:**
In `crates/ftui-runtime/src/render_thread.rs`, the `render_loop` never called `writer.gc()`. Since `TerminalWriter` owns the `GraphemePool` (used for interning complex Unicode characters), failure to garbage collect would lead to unbounded memory growth for long-running applications using the dedicated render thread feature.

**Fix:**
Added a loop counter and a periodic `writer.gc()` call (every 1000 iterations) to the render loop, mirroring the memory management strategy of the main `Program` loop.

## Session 3 Findings

### 5. Performance Fix: Rope Grapheme To Char Index

**Issue:**
`Rope::grapheme_to_char_idx` in `crates/ftui-text/src/rope.rs` was implemented by converting the *entire* rope to a string (`self.to_string()`) and then iterating over graphemes. This creates a massive memory allocation and performance bottleneck for large documents, effectively making grapheme-based operations O(N) in both time and memory where N is document size.

**Fix:**
Optimized `grapheme_to_char_idx` to iterate over lines via `self.lines()` (which returns copy-on-write slices) instead of allocating the full string. This reduces memory usage to O(LineLength) and is significantly faster for large documents while maintaining correctness.

### 6. Bug Fix: Integer Truncation in List and Table

**Issue:**
In `crates/ftui-widgets/src/list.rs` and `crates/ftui-widgets/src/table.rs`, measurement logic cast `usize` widths (from `unicode-width`) directly to `u16` using `as u16`. This is a truncating cast, meaning a line of width 65536 would be treated as width 0, potentially causing layout glitches for extremely long content.

**Fix:**
Replaced `as u16` with `.min(u16::MAX as usize) as u16` (saturating cast) in `ListItem::measure` and `Table::compute_intrinsic_widths`.

## Recommendations

*   **Performance**: The `Table` widget uses eager measurement (O(Rows * Cols)). For extremely large datasets, consider using `VirtualizedList` with a custom item renderer instead of the standard `Table` widget.
*   **Testing**: Continue adding property-based tests (proptests) for new widgets, as they have proven valuable in `ftui-text` and `ftui-layout`.

## Conclusion

The project is in a release-ready state. The identified issues have been resolved, and the codebase remains stable and robust.