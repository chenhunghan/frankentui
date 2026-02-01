# ANSI Escape Sequence Reference

This document provides a reference for the ANSI escape sequences used by FrankenTUI. It serves as the authoritative source for implementing the Presenter, TerminalWriter, and terminal-model tests.

## Notation

- `ESC` = `\x1b` (0x1B)
- `CSI` = `ESC [` (Control Sequence Introducer)
- `OSC` = `ESC ]` (Operating System Command)
- `ST` = `ESC \` or `\x07` (String Terminator)
- `{n}` = numeric parameter
- `{r}`, `{c}` = row, column (1-indexed)

## SGR (Select Graphic Rendition)

Format: `CSI {code}[;{code}...] m`

### Reset and Attributes

| Sequence | Code | Effect |
|----------|------|--------|
| `CSI 0 m` | 0 | Reset all attributes |
| `CSI 1 m` | 1 | Bold / increased intensity |
| `CSI 2 m` | 2 | Dim / decreased intensity |
| `CSI 3 m` | 3 | Italic |
| `CSI 4 m` | 4 | Underline |
| `CSI 5 m` | 5 | Slow blink |
| `CSI 7 m` | 7 | Reverse video (swap fg/bg) |
| `CSI 8 m` | 8 | Hidden / invisible |
| `CSI 9 m` | 9 | Strikethrough |

### Attribute Reset

| Sequence | Code | Effect |
|----------|------|--------|
| `CSI 21 m` | 21 | Double underline (or bold off) |
| `CSI 22 m` | 22 | Normal intensity (bold/dim off) |
| `CSI 23 m` | 23 | Italic off |
| `CSI 24 m` | 24 | Underline off |
| `CSI 25 m` | 25 | Blink off |
| `CSI 27 m` | 27 | Reverse off |
| `CSI 28 m` | 28 | Hidden off |
| `CSI 29 m` | 29 | Strikethrough off |

### Basic Colors (3/4-bit)

**Foreground (30-37, 90-97):**

| Code | Color | Bright Code | Bright Color |
|------|-------|-------------|--------------|
| 30 | Black | 90 | Bright Black (Gray) |
| 31 | Red | 91 | Bright Red |
| 32 | Green | 92 | Bright Green |
| 33 | Yellow | 93 | Bright Yellow |
| 34 | Blue | 94 | Bright Blue |
| 35 | Magenta | 95 | Bright Magenta |
| 36 | Cyan | 96 | Bright Cyan |
| 37 | White | 97 | Bright White |
| 39 | Default | - | - |

**Background (40-47, 100-107):**

| Code | Color | Bright Code | Bright Color |
|------|-------|-------------|--------------|
| 40 | Black | 100 | Bright Black |
| 41 | Red | 101 | Bright Red |
| 42 | Green | 102 | Bright Green |
| 43 | Yellow | 103 | Bright Yellow |
| 44 | Blue | 104 | Bright Blue |
| 45 | Magenta | 105 | Bright Magenta |
| 46 | Cyan | 106 | Bright Cyan |
| 47 | White | 107 | Bright White |
| 49 | Default | - | - |

### 256-Color Mode (8-bit)

| Sequence | Effect |
|----------|--------|
| `CSI 38;5;{n} m` | Set foreground to color `n` (0-255) |
| `CSI 48;5;{n} m` | Set background to color `n` (0-255) |

**256-color palette:**
- 0-7: Standard colors (same as 30-37)
- 8-15: High-intensity colors (same as 90-97)
- 16-231: 6x6x6 RGB cube (`16 + 36*r + 6*g + b`, r/g/b in 0-5)
- 232-255: Grayscale (24 shades, dark to light)

### TrueColor Mode (24-bit)

| Sequence | Effect |
|----------|--------|
| `CSI 38;2;{r};{g};{b} m` | Set foreground to RGB |
| `CSI 48;2;{r};{g};{b} m` | Set background to RGB |

## Cursor Movement

### Absolute Positioning

| Sequence | Name | Effect |
|----------|------|--------|
| `CSI {r};{c} H` | CUP | Move cursor to row `r`, column `c` |
| `CSI {r};{c} f` | HVP | Same as CUP |
| `CSI {c} G` | CHA | Move cursor to column `c` |
| `CSI {r} d` | VPA | Move cursor to row `r` |

### Relative Movement

| Sequence | Name | Effect |
|----------|------|--------|
| `CSI {n} A` | CUU | Move cursor up `n` rows |
| `CSI {n} B` | CUD | Move cursor down `n` rows |
| `CSI {n} C` | CUF | Move cursor forward `n` columns |
| `CSI {n} D` | CUB | Move cursor back `n` columns |
| `CSI {n} E` | CNL | Move to beginning of line `n` lines down |
| `CSI {n} F` | CPL | Move to beginning of line `n` lines up |

### Cursor Save/Restore

| Sequence | Name | Effect |
|----------|------|--------|
| `ESC 7` | DECSC | Save cursor position + attributes (DEC) |
| `ESC 8` | DECRC | Restore cursor position + attributes (DEC) |
| `CSI s` | SCOSC | Save cursor position (ANSI) |
| `CSI u` | SCORC | Restore cursor position (ANSI) |

**Note:** DEC save/restore (`ESC 7`/`ESC 8`) is preferred as it saves more state and has wider support. See [ADR-001](adr/ADR-001-inline-mode.md).

## Erase Operations

### Erase in Display (ED)

| Sequence | Effect |
|----------|--------|
| `CSI 0 J` | Erase from cursor to end of screen |
| `CSI 1 J` | Erase from start of screen to cursor |
| `CSI 2 J` | Erase entire screen |
| `CSI 3 J` | Erase entire screen + scrollback |

### Erase in Line (EL)

| Sequence | Effect |
|----------|--------|
| `CSI 0 K` | Erase from cursor to end of line |
| `CSI 1 K` | Erase from start of line to cursor |
| `CSI 2 K` | Erase entire line |

## Screen Modes

### Alternate Screen Buffer

| Sequence | Effect |
|----------|--------|
| `CSI ? 1049 h` | Enable alternate screen buffer (save main, switch) |
| `CSI ? 1049 l` | Disable alternate screen buffer (restore main) |

### Cursor Visibility

| Sequence | Name | Effect |
|----------|------|--------|
| `CSI ? 25 h` | DECTCEM | Show cursor |
| `CSI ? 25 l` | DECTCEM | Hide cursor |

## Synchronized Output (DEC 2026)

Reduces flicker by batching output updates.

| Sequence | Effect |
|----------|--------|
| `CSI ? 2026 h` | Begin synchronized update |
| `CSI ? 2026 l` | End synchronized update |

**Usage:**
```
CSI ? 2026 h    # Begin sync
... render UI ...
CSI ? 2026 l    # End sync (terminal now updates display)
```

**Notes:**
- Can be nested (terminal tracks nesting level)
- Terminals without support ignore these sequences
- FrankenTUI's terminal model tracks sync nesting for verification

## OSC 8 Hyperlinks

Format: `OSC 8 ; {params} ; {uri} ST`

### Start Hyperlink

```
ESC ] 8 ; ; https://example.com BEL
```

Or with parameters:
```
ESC ] 8 ; id=mylink ; https://example.com BEL
```

### End Hyperlink

```
ESC ] 8 ; ; BEL
```

**Example:**
```
ESC]8;;https://example.com\x07Click here\x1b]8;;\x07
```

**Notes:**
- The `id` parameter groups related link segments
- Empty URI ends the hyperlink
- `ST` can be `BEL` (`\x07`) or `ESC \` (`\x1b\x5c`)

## Mouse Tracking

### Enable Mouse Modes

| Sequence | Mode | Effect |
|----------|------|--------|
| `CSI ? 1000 h` | X10 | Button press only |
| `CSI ? 1002 h` | Button | Button press/release + motion while pressed |
| `CSI ? 1003 h` | Any | All mouse events including motion |
| `CSI ? 1006 h` | SGR | Enable SGR extended mouse encoding |

### Disable Mouse Modes

| Sequence | Effect |
|----------|--------|
| `CSI ? 1000 l` | Disable X10 mode |
| `CSI ? 1002 l` | Disable button mode |
| `CSI ? 1003 l` | Disable any-event mode |
| `CSI ? 1006 l` | Disable SGR encoding |

### SGR Mouse Event Format

Press: `CSI < {button};{x};{y} M`
Release: `CSI < {button};{x};{y} m`

**Button encoding:**
- 0: Left
- 1: Middle
- 2: Right
- 64: Scroll up
- 65: Scroll down
- +4: Shift held
- +8: Meta/Alt held
- +16: Control held
- +32: Motion event

## Bracketed Paste

### Enable/Disable

| Sequence | Effect |
|----------|--------|
| `CSI ? 2004 h` | Enable bracketed paste mode |
| `CSI ? 2004 l` | Disable bracketed paste mode |

### Paste Boundaries

When enabled, pasted text is wrapped:
- Start: `CSI 200 ~`
- End: `CSI 201 ~`

**Example input from terminal:**
```
\x1b[200~pasted text here\x1b[201~
```

## Focus Events

### Enable/Disable

| Sequence | Effect |
|----------|--------|
| `CSI ? 1004 h` | Enable focus event reporting |
| `CSI ? 1004 l` | Disable focus event reporting |

### Focus Events

| Sequence | Event |
|----------|-------|
| `CSI I` | Terminal gained focus |
| `CSI O` | Terminal lost focus |

## Scroll Regions (DECSTBM)

| Sequence | Effect |
|----------|--------|
| `CSI {top};{bottom} r` | Set scroll region to rows `top` through `bottom` |
| `CSI r` | Reset scroll region to full screen |

**Notes:**
- Used for inline mode scroll-region optimization
- Must be reset on exit/panic
- See [ADR-001](adr/ADR-001-inline-mode.md) for strategy details

## Device Status Reports

### Query Cursor Position

| Sequence | Effect |
|----------|--------|
| `CSI 6 n` | Request cursor position |

Response: `CSI {r};{c} R`

### Query Device Attributes

| Sequence | Effect |
|----------|--------|
| `CSI c` | Primary device attributes |
| `CSI > c` | Secondary device attributes |

## Terminal Identification

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `TERM` | Terminal type (e.g., `xterm-256color`) |
| `TERM_PROGRAM` | Terminal application (e.g., `iTerm.app`) |
| `COLORTERM` | Color support (`truecolor`, `24bit`) |
| `TMUX` | tmux session info (if in tmux) |
| `STY` | screen session info (if in screen) |

## Implementation Notes

### FrankenTUI Presenter Strategy

Per [ADR-002](adr/ADR-002-presenter-emission.md), the presenter uses **reset+apply** for style changes:

```rust
// For each style transition:
write!(w, "\x1b[0m")?;           // Reset
write!(w, "\x1b[{};...m", ...)?; // Apply new style
```

### Terminal Model Verification

The [terminal model](../crates/ftui-render/src/terminal_model.rs) parses and validates these sequences for testing:

- Cursor position stays in bounds
- SGR state doesn't leak between cells
- Hyperlinks are properly closed
- Synchronized output is balanced

## References

- [ECMA-48](https://www.ecma-international.org/publications-and-standards/standards/ecma-48/) - Control Functions for Coded Character Sets
- [XTerm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - Comprehensive reference
- [ANSI Escape Codes (Wikipedia)](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [Terminal WG Specs](https://gitlab.freedesktop.org/terminal-wg/specifications) - Modern terminal specifications
