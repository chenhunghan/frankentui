#![forbid(unsafe_code)]

//! Mind-blowing dashboard screen.
//!
//! Showcases EVERY major FrankenTUI capability simultaneously:
//! - Animated gradient title
//! - Live plasma visual effect (Braille canvas)
//! - Real-time sparkline charts
//! - Syntax-highlighted code preview
//! - GFM markdown preview
//! - System stats (FPS, theme, size)
//! - Keyboard shortcuts
//!
//! Dynamically reflowable from 40x10 to 200x50+.

use std::collections::VecDeque;
use std::time::Instant;

use ftui_core::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ftui_core::geometry::Rect;
use ftui_extras::canvas::{Canvas, Mode, Painter};
use ftui_extras::charts::Sparkline;
use ftui_extras::markdown::{MarkdownRenderer, MarkdownTheme};
use ftui_extras::syntax::SyntaxHighlighter;
use ftui_extras::text_effects::{
    ColorGradient, CursorPosition, CursorStyle, Direction, DissolveMode, RevealMode,
    StyledMultiLine, StyledText, TextEffect,
};
use ftui_layout::{Constraint, Flex};
use ftui_render::cell::PackedRgba;
use ftui_render::frame::Frame;
use ftui_runtime::Cmd;
use ftui_style::Style;
use ftui_text::{Line, Span, Text, WrapMode};
use ftui_widgets::Widget;
use ftui_widgets::block::{Alignment, Block};
use ftui_widgets::borders::{BorderType, Borders};
use ftui_widgets::paragraph::Paragraph;
use ftui_widgets::progress::{MiniBar, MiniBarColors};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{HelpEntry, Screen};
use crate::data::{AlertSeverity, SimulatedData};
use crate::theme;

struct CodeSample {
    label: &'static str,
    lang: &'static str,
    code: &'static str,
}

const CODE_SAMPLES: &[CodeSample] = &[
    CodeSample {
        label: "Rust",
        lang: "rs",
        code: r#"// runtime.rs
use ftui_runtime::{Cmd, Model, Program};
use ftui_core::event::Event;

struct AppState {
    frames: u64,
    last_resize: (u16, u16),
}

impl Model for AppState {
    type Message = Event;

    fn update(&mut self, msg: Event) -> Cmd<Event> {
        match msg {
            Event::Key(k) if k.is_char('q') => Cmd::quit(),
            Event::Resize { width, height } => {
                self.last_resize = (width, height);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut ftui_render::frame::Frame) {
        // draw widgets...
        let _ = frame;
    }
}

fn main() -> ftui::Result<()> {
    let mut app = AppState { frames: 0, last_resize: (0, 0) };
    Program::new(&mut app).run()
}
"#,
    },
    CodeSample {
        label: "TypeScript",
        lang: "ts",
        code: r#"// api.ts
type Mode = "inline" | "alt";

interface Session {
  id: string;
  mode: Mode;
  caps: string[];
}

export async function startSession(mode: Mode): Promise<Session> {
  const res = await fetch("/api/session", {
    method: "POST",
    body: JSON.stringify({ mode }),
  });
  if (!res.ok) throw new Error("boot failed");
  return res.json();
}

export function diff(a: string[], b: string[]) {
  return a.filter((x) => !b.includes(x));
}
"#,
    },
    CodeSample {
        label: "Python",
        lang: "py",
        code: r#"# pipeline.py
from dataclasses import dataclass
from typing import Iterable

@dataclass
class Frame:
    id: int
    dirty: bool

async def render(frames: Iterable[Frame]) -> int:
    count = 0
    async for f in frames:
        if f.dirty:
            count += 1
    return count

def diff(prev: list[str], nxt: list[str]) -> list[str]:
    return [x for x in nxt if x not in prev]
"#,
    },
    CodeSample {
        label: "Go",
        lang: "go",
        code: r#"// diff.go
package diff

import "context"

type Cell struct{ ch rune }

func Compute(ctx context.Context, a, b []Cell) (int, error) {
    changed := 0
    for i := range a {
        select {
        case <-ctx.Done():
            return changed, ctx.Err()
        default:
            if a[i] != b[i] {
                changed++
            }
        }
    }
    return changed, nil
}
"#,
    },
    CodeSample {
        label: "SQL",
        lang: "sql",
        code: r#"WITH dirty AS (
  SELECT frame_id, count(*) AS cells
  FROM buffer_diff
  WHERE changed = true
  GROUP BY frame_id
),
ranked AS (
  SELECT frame_id, cells,
         dense_rank() OVER (ORDER BY cells DESC) AS r
  FROM dirty
)
SELECT frame_id, cells
FROM ranked
WHERE r <= 3
ORDER BY cells DESC;"#,
    },
    CodeSample {
        label: "JSON",
        lang: "json",
        code: r#"{
  "mode": "inline",
  "uiHeight": 12,
  "renderer": {
    "diff": "row-major",
    "cellBytes": 16
  },
  "features": ["mouse", "paste", "focus"],
  "theme": "NordicFrost"
}"#,
    },
    CodeSample {
        label: "YAML",
        lang: "yaml",
        code: r#"pipeline:
  - name: render
    budget_ms: 12
    steps:
      - sanitize
      - diff
      - present
  - name: snapshot
    sizes: [80x24, 120x40]"#,
    },
];

const DASH_MARKDOWN_SAMPLES: &[&str] = &[
    r#"# FrankenTUI Field Notes

> **Goal:** deterministic output, no surprises.

## Highlights
- [x] Inline mode with scrollback
- [x] One-writer rule
- [x] 16-byte cell invariant
- [ ] GPU raster (not needed)

### Architecture Table
| Layer | Role | Notes |
| --- | --- | --- |
| core | input | crossterm events |
| render | diff | row-major scan |
| runtime | loop | Elm-style model |

```rust
fn render(frame: &mut Frame) {
    frame.clear();
}
```

```json
{ "mode": "inline", "ui_height": 12 }
```

> [!NOTE]
> Math: `E = mc^2` and `∑ᵢ xᵢ`

Footnote[^1] and **links**: https://ftui.dev

[^1]: Determinism beats magic.
"#,
    r#"# Rendering Playbook

1. **Build** the frame
2. **Diff** buffers
3. **Present** ANSI

## Task List
- [x] Dirty-row tracking
- [x] ANSI cost model
- [ ] GPU? nope

| Metric | Target |
| --- | --- |
| Frame | <16ms |
| Diff | <4ms |

```bash
FTUI_HARNESS_SCREEN_MODE=inline cargo run -p ftui-harness
```

> [!TIP]
> Use `Cmd::batch` for side effects.
"#,
];

const EFFECT_GFM_SAMPLES: &[&str] = &[
    r#"# FX Lab
> *"Render truth, not pixels."*

- [x] Inline scrollback
- [x] Deterministic diff
- [ ] GPU hype

| Key | Action |
| --- | --- |
| `e` | next FX |
| `c` | next code |

```bash
ftui run --inline --height 12
```

[^1]: Effects are deterministic.
"#,
    r#"## GFM Stress
1. **Bold** + _italic_
2. `code` + ~~strike~~
3. Link: https://ftui.dev

| op | cost |
| -- | --- |
| diff | O(n) |

> [!TIP]
> Use `Cmd::batch`.
"#,
    r#"### Mixed
- [x] Tasks
- [ ] Benchmarks

```sql
SELECT * FROM diff WHERE dirty = true;
```

Math: `∫ f(x) dx` and `α + β`
"#,
];

#[derive(Clone, Copy)]
enum EffectKind {
    None,
    FadeIn,
    FadeOut,
    Pulse,
    OrganicPulse,
    HorizontalGradient,
    AnimatedGradient,
    RainbowGradient,
    VerticalGradient,
    DiagonalGradient,
    RadialGradient,
    ColorCycle,
    ColorWave,
    Glow,
    PulsingGlow,
    Typewriter,
    Scramble,
    Glitch,
    Wave,
    Bounce,
    Shake,
    Cascade,
    Cursor,
    Reveal,
    RevealMask,
    ChromaticAberration,
    Scanline,
    ParticleDissolve,
}

struct EffectDemo {
    name: &'static str,
    kind: EffectKind,
}

const EFFECT_DEMOS: &[EffectDemo] = &[
    EffectDemo {
        name: "None",
        kind: EffectKind::None,
    },
    EffectDemo {
        name: "FadeIn",
        kind: EffectKind::FadeIn,
    },
    EffectDemo {
        name: "FadeOut",
        kind: EffectKind::FadeOut,
    },
    EffectDemo {
        name: "Pulse",
        kind: EffectKind::Pulse,
    },
    EffectDemo {
        name: "OrganicPulse",
        kind: EffectKind::OrganicPulse,
    },
    EffectDemo {
        name: "HorizontalGradient",
        kind: EffectKind::HorizontalGradient,
    },
    EffectDemo {
        name: "AnimatedGradient",
        kind: EffectKind::AnimatedGradient,
    },
    EffectDemo {
        name: "RainbowGradient",
        kind: EffectKind::RainbowGradient,
    },
    EffectDemo {
        name: "VerticalGradient",
        kind: EffectKind::VerticalGradient,
    },
    EffectDemo {
        name: "DiagonalGradient",
        kind: EffectKind::DiagonalGradient,
    },
    EffectDemo {
        name: "RadialGradient",
        kind: EffectKind::RadialGradient,
    },
    EffectDemo {
        name: "ColorCycle",
        kind: EffectKind::ColorCycle,
    },
    EffectDemo {
        name: "ColorWave",
        kind: EffectKind::ColorWave,
    },
    EffectDemo {
        name: "Glow",
        kind: EffectKind::Glow,
    },
    EffectDemo {
        name: "PulsingGlow",
        kind: EffectKind::PulsingGlow,
    },
    EffectDemo {
        name: "Typewriter",
        kind: EffectKind::Typewriter,
    },
    EffectDemo {
        name: "Scramble",
        kind: EffectKind::Scramble,
    },
    EffectDemo {
        name: "Glitch",
        kind: EffectKind::Glitch,
    },
    EffectDemo {
        name: "Wave",
        kind: EffectKind::Wave,
    },
    EffectDemo {
        name: "Bounce",
        kind: EffectKind::Bounce,
    },
    EffectDemo {
        name: "Shake",
        kind: EffectKind::Shake,
    },
    EffectDemo {
        name: "Cascade",
        kind: EffectKind::Cascade,
    },
    EffectDemo {
        name: "Cursor",
        kind: EffectKind::Cursor,
    },
    EffectDemo {
        name: "Reveal",
        kind: EffectKind::Reveal,
    },
    EffectDemo {
        name: "RevealMask",
        kind: EffectKind::RevealMask,
    },
    EffectDemo {
        name: "ChromaticAberration",
        kind: EffectKind::ChromaticAberration,
    },
    EffectDemo {
        name: "Scanline",
        kind: EffectKind::Scanline,
    },
    EffectDemo {
        name: "ParticleDissolve",
        kind: EffectKind::ParticleDissolve,
    },
];

/// Dashboard state.
pub struct Dashboard {
    // Animation
    tick_count: u64,
    time: f64,

    // Data sources
    simulated_data: SimulatedData,

    // FPS tracking
    frame_times: VecDeque<u64>,
    last_frame: Option<Instant>,
    fps: f64,

    // Syntax highlighter (cached)
    highlighter: SyntaxHighlighter,

    // Markdown renderer (cached)
    md_renderer: MarkdownRenderer,

    // Code showcase state
    code_index: usize,

    // Markdown streaming state
    md_sample_index: usize,
    md_stream_pos: usize,

    // Text effects showcase state
    effect_index: usize,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    pub fn new() -> Self {
        let mut simulated_data = SimulatedData::default();
        // Pre-populate some history
        for t in 0..30 {
            simulated_data.tick(t);
        }

        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_theme(theme::syntax_theme());

        Self {
            tick_count: 30,
            time: 0.0,
            simulated_data,
            frame_times: VecDeque::with_capacity(60),
            last_frame: None,
            fps: 0.0,
            highlighter,
            md_renderer: MarkdownRenderer::new(MarkdownTheme::default()),
            code_index: 0,
            md_sample_index: 0,
            md_stream_pos: 0,
            effect_index: 0,
        }
    }

    pub fn apply_theme(&mut self) {
        self.highlighter.set_theme(theme::syntax_theme());
    }

    fn current_code_sample(&self) -> &'static CodeSample {
        &CODE_SAMPLES[self.code_index % CODE_SAMPLES.len()]
    }

    fn current_markdown_sample(&self) -> &'static str {
        DASH_MARKDOWN_SAMPLES[self.md_sample_index % DASH_MARKDOWN_SAMPLES.len()]
    }

    fn markdown_stream_complete(&self) -> bool {
        self.md_stream_pos >= self.current_markdown_sample().len()
    }

    fn tick_markdown_stream(&mut self) {
        if self.markdown_stream_complete() {
            return;
        }
        let md = self.current_markdown_sample();
        let max_len = md.len();
        let mut new_pos = self.md_stream_pos.saturating_add(6);
        while new_pos < max_len && !md.is_char_boundary(new_pos) {
            new_pos += 1;
        }
        self.md_stream_pos = new_pos.min(max_len);
    }

    fn reset_markdown_stream(&mut self) {
        self.md_stream_pos = 0;
    }

    fn current_effect_demo(&self) -> &'static EffectDemo {
        &EFFECT_DEMOS[self.effect_index % EFFECT_DEMOS.len()]
    }

    fn build_effect(&self, kind: EffectKind, text_len: usize) -> TextEffect {
        let progress = (self.time * 0.6).sin() * 0.5 + 0.5;
        let progress = progress.clamp(0.0, 1.0);
        let visible = (progress * text_len.max(1) as f64).max(1.0);

        match kind {
            EffectKind::None => TextEffect::None,
            EffectKind::FadeIn => TextEffect::FadeIn { progress },
            EffectKind::FadeOut => TextEffect::FadeOut { progress },
            EffectKind::Pulse => TextEffect::Pulse {
                speed: 1.8,
                min_alpha: 0.25,
            },
            EffectKind::OrganicPulse => TextEffect::OrganicPulse {
                speed: 0.6,
                min_brightness: 0.35,
                asymmetry: 0.55,
                phase_variation: 0.25,
                seed: 42,
            },
            EffectKind::HorizontalGradient => TextEffect::HorizontalGradient {
                gradient: ColorGradient::sunset(),
            },
            EffectKind::AnimatedGradient => TextEffect::AnimatedGradient {
                gradient: ColorGradient::cyberpunk(),
                speed: 0.4,
            },
            EffectKind::RainbowGradient => TextEffect::RainbowGradient { speed: 0.6 },
            EffectKind::VerticalGradient => TextEffect::VerticalGradient {
                gradient: ColorGradient::ocean(),
            },
            EffectKind::DiagonalGradient => TextEffect::DiagonalGradient {
                gradient: ColorGradient::lavender(),
                angle: 45.0,
            },
            EffectKind::RadialGradient => TextEffect::RadialGradient {
                gradient: ColorGradient::fire(),
                center: (0.5, 0.5),
                aspect: 1.2,
            },
            EffectKind::ColorCycle => TextEffect::ColorCycle {
                colors: vec![
                    theme::accent::PRIMARY.into(),
                    theme::accent::ACCENT_3.into(),
                    theme::accent::ACCENT_6.into(),
                    theme::accent::ACCENT_9.into(),
                ],
                speed: 0.9,
            },
            EffectKind::ColorWave => TextEffect::ColorWave {
                color1: theme::accent::PRIMARY.into(),
                color2: theme::accent::ACCENT_8.into(),
                speed: 1.2,
                wavelength: 8.0,
            },
            EffectKind::Glow => TextEffect::Glow {
                color: PackedRgba::rgb(255, 200, 100),
                intensity: 0.6,
            },
            EffectKind::PulsingGlow => TextEffect::PulsingGlow {
                color: PackedRgba::rgb(255, 120, 180),
                speed: 1.4,
            },
            EffectKind::Typewriter => TextEffect::Typewriter {
                visible_chars: visible,
            },
            EffectKind::Scramble => TextEffect::Scramble { progress },
            EffectKind::Glitch => TextEffect::Glitch {
                intensity: 0.25 + 0.35 * progress,
            },
            EffectKind::Wave => TextEffect::Wave {
                amplitude: 1.2,
                wavelength: 10.0,
                speed: 1.0,
                direction: Direction::Down,
            },
            EffectKind::Bounce => TextEffect::Bounce {
                height: 2.0,
                speed: 1.2,
                stagger: 0.15,
                damping: 0.88,
            },
            EffectKind::Shake => TextEffect::Shake {
                intensity: 0.8,
                speed: 6.0,
                seed: 7,
            },
            EffectKind::Cascade => TextEffect::Cascade {
                speed: 18.0,
                direction: Direction::Right,
                stagger: 0.08,
            },
            EffectKind::Cursor => TextEffect::Cursor {
                style: CursorStyle::Block,
                blink_speed: 2.5,
                position: CursorPosition::End,
            },
            EffectKind::Reveal => TextEffect::Reveal {
                mode: RevealMode::CenterOut,
                progress,
                seed: 13,
            },
            EffectKind::RevealMask => TextEffect::RevealMask {
                angle: 35.0,
                progress,
                softness: 0.3,
            },
            EffectKind::ChromaticAberration => TextEffect::ChromaticAberration {
                offset: 2,
                direction: Direction::Right,
                animated: true,
                speed: 0.4,
            },
            EffectKind::Scanline => TextEffect::Scanline {
                intensity: 0.35,
                line_gap: 2,
                scroll: true,
                scroll_speed: 0.7,
                flicker: 0.05,
            },
            EffectKind::ParticleDissolve => TextEffect::ParticleDissolve {
                progress,
                mode: DissolveMode::Dissolve,
                speed: 0.8,
                gravity: 0.4,
                seed: 9,
            },
        }
    }

    /// Update FPS calculation.
    fn update_fps(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_frame {
            let elapsed_us = now.duration_since(last).as_micros() as u64;
            self.frame_times.push_back(elapsed_us);
            if self.frame_times.len() > 30 {
                self.frame_times.pop_front();
            }
            if !self.frame_times.is_empty() {
                let avg_us: u64 =
                    self.frame_times.iter().sum::<u64>() / self.frame_times.len() as u64;
                if avg_us > 0 {
                    self.fps = 1_000_000.0 / avg_us as f64;
                }
            }
        }
        self.last_frame = Some(now);
    }

    // =========================================================================
    // Panel Renderers
    // =========================================================================

    /// Render animated gradient header.
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 1 {
            return;
        }

        let title = "FRANKENTUI DASHBOARD";
        let gradient = ColorGradient::new(vec![
            (0.0, theme::accent::ACCENT_2.into()),
            (0.5, theme::accent::ACCENT_1.into()),
            (1.0, theme::accent::ACCENT_3.into()),
        ]);
        let effect = TextEffect::AnimatedGradient {
            gradient,
            speed: 0.3,
        };

        let styled = StyledText::new(title).effect(effect).bold().time(self.time);

        styled.render(area, frame);
    }

    /// Render mini plasma effect using Braille canvas.
    fn render_plasma(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.width < 4 || area.height < 3 {
            return;
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Plasma")
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::DASHBOARD));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() || inner.width < 2 || inner.height < 2 {
            return;
        }

        let mut painter = Painter::for_area(inner, Mode::Braille);
        let (pw, ph) = painter.size();

        // Simple plasma using two sine waves
        let t = self.time * 0.5;
        let hue_shift = (t * 0.07).rem_euclid(1.0);
        for py in 0..ph as i32 {
            for px in 0..pw as i32 {
                let x = px as f64 / pw as f64;
                let y = py as f64 / ph as f64;

                // Two-wave plasma formula
                let v1 = (x * 10.0 + t * 2.0).sin();
                let v2 = (y * 10.0 + t * 1.5).sin();
                let v3 = ((x + y) * 8.0 + t).sin();
                let v = (v1 + v2 + v3) / 3.0;

                // Map plasma value to a theme-coherent accent gradient.
                let color = theme::accent_gradient((v + 1.0) * 0.5 + hue_shift);

                painter.point_colored(px, py, color);
            }
        }

        Canvas::from_painter(&painter)
            .style(Style::new().fg(theme::fg::PRIMARY))
            .render(inner, frame);
    }

    /// Render sparklines panel.
    fn render_sparklines(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 3 {
            return;
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Charts")
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::DATA_VIZ));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() || inner.height < 2 {
            return;
        }

        // Split into rows for each sparkline
        let rows = Flex::vertical()
            .constraints([
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(1),
            ])
            .split(inner);

        let cpu_data: Vec<f64> = self.simulated_data.cpu_history.iter().copied().collect();
        let mem_data: Vec<f64> = self.simulated_data.memory_history.iter().copied().collect();
        let net_data: Vec<f64> = self.simulated_data.network_in.iter().copied().collect();

        // CPU sparkline
        if !rows[0].is_empty() && !cpu_data.is_empty() {
            let label_area = Rect::new(rows[0].x, rows[0].y, 4.min(rows[0].width), 1);
            let spark_area = Rect::new(
                rows[0].x + 4.min(rows[0].width),
                rows[0].y,
                rows[0].width.saturating_sub(4),
                1,
            );
            Paragraph::new("CPU ")
                .style(Style::new().fg(theme::fg::SECONDARY))
                .render(label_area, frame);
            if !spark_area.is_empty() {
                Sparkline::new(&cpu_data)
                    .style(Style::new().fg(theme::accent::PRIMARY))
                    .gradient(
                        theme::accent::PRIMARY.into(),
                        theme::accent::ACCENT_7.into(),
                    )
                    .render(spark_area, frame);
            }
        }

        // Memory sparkline
        if rows.len() > 1 && !rows[1].is_empty() && !mem_data.is_empty() {
            let label_area = Rect::new(rows[1].x, rows[1].y, 4.min(rows[1].width), 1);
            let spark_area = Rect::new(
                rows[1].x + 4.min(rows[1].width),
                rows[1].y,
                rows[1].width.saturating_sub(4),
                1,
            );
            Paragraph::new("MEM ")
                .style(Style::new().fg(theme::fg::SECONDARY))
                .render(label_area, frame);
            if !spark_area.is_empty() {
                Sparkline::new(&mem_data)
                    .style(Style::new().fg(theme::accent::SUCCESS))
                    .gradient(
                        theme::accent::SUCCESS.into(),
                        theme::accent::ACCENT_9.into(),
                    )
                    .render(spark_area, frame);
            }
        }

        // Network sparkline
        if rows.len() > 2 && !rows[2].is_empty() && !net_data.is_empty() {
            let label_area = Rect::new(rows[2].x, rows[2].y, 4.min(rows[2].width), 1);
            let spark_area = Rect::new(
                rows[2].x + 4.min(rows[2].width),
                rows[2].y,
                rows[2].width.saturating_sub(4),
                1,
            );
            Paragraph::new("NET ")
                .style(Style::new().fg(theme::fg::SECONDARY))
                .render(label_area, frame);
            if !spark_area.is_empty() {
                Sparkline::new(&net_data)
                    .style(Style::new().fg(theme::accent::WARNING))
                    .gradient(
                        theme::accent::WARNING.into(),
                        theme::accent::ACCENT_10.into(),
                    )
                    .render(spark_area, frame);
            }
        }
    }

    /// Render syntax-highlighted code preview.
    fn render_code(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 3 {
            return;
        }

        let sample = self.current_code_sample();
        let title = format!(
            "Code · {} ({}/{})",
            sample.label,
            self.code_index + 1,
            CODE_SAMPLES.len()
        );

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title.as_str())
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::CODE_EXPLORER));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let highlighted = self.highlighter.highlight(sample.code, sample.lang);

        // Render as paragraph with styled text
        render_text(frame, inner, &highlighted);
    }

    /// Render system info panel.
    ///
    /// `dashboard_size` is the total dashboard area (width, height) for display.
    fn render_info(&self, frame: &mut Frame, area: Rect, dashboard_size: (u16, u16)) {
        if area.is_empty() || area.height < 3 {
            return;
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Info")
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::PERFORMANCE));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let theme_name = theme::current_theme_name();
        let info = format!(
            "FPS: {:.0}\n{}x{}\n{}\nTick: {}",
            self.fps, dashboard_size.0, dashboard_size.1, theme_name, self.tick_count
        );

        if inner.height < 6 {
            Paragraph::new(info)
                .style(Style::new().fg(theme::fg::SECONDARY))
                .render(inner, frame);
            return;
        }

        let rows = Flex::vertical()
            .constraints([Constraint::Min(2), Constraint::Fixed(3)])
            .split(inner);

        Paragraph::new(info)
            .style(Style::new().fg(theme::fg::SECONDARY))
            .render(rows[0], frame);

        self.render_mini_bars(frame, rows[1]);
    }

    /// Render compact mini-bars for CPU/MEM/Disk usage.
    fn render_mini_bars(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 1 {
            return;
        }

        let rows = Flex::vertical()
            .constraints([
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(1),
            ])
            .split(area);

        let cpu = self
            .simulated_data
            .cpu_history
            .back()
            .copied()
            .unwrap_or(0.0)
            / 100.0;
        let mem = self
            .simulated_data
            .memory_history
            .back()
            .copied()
            .unwrap_or(0.0)
            / 100.0;
        let disk = self
            .simulated_data
            .disk_usage
            .first()
            .map(|(_, v)| *v / 100.0)
            .unwrap_or(0.0);

        let colors = MiniBarColors::new(
            theme::intent::success_text(),
            theme::intent::warning_text(),
            theme::intent::info_text(),
            theme::intent::error_text(),
        );

        self.render_mini_bar_row(frame, rows[0], "CPU", cpu, colors);
        self.render_mini_bar_row(frame, rows[1], "MEM", mem, colors);
        self.render_mini_bar_row(frame, rows[2], "DSK", disk, colors);
    }

    fn render_mini_bar_row(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        value: f64,
        colors: MiniBarColors,
    ) {
        if area.is_empty() {
            return;
        }

        let label_width = 4.min(area.width);
        let label_area = Rect::new(area.x, area.y, label_width, 1);
        Paragraph::new(format!("{label} "))
            .style(Style::new().fg(theme::fg::SECONDARY))
            .render(label_area, frame);

        let bar_width = area.width.saturating_sub(label_width);
        if bar_width == 0 {
            return;
        }

        let bar_area = Rect::new(area.x + label_width, area.y, bar_width, 1);
        MiniBar::new(value, bar_width)
            .colors(colors)
            .show_percent(true)
            .render(bar_area, frame);
    }

    /// Render markdown preview.
    fn render_markdown(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 2 {
            return;
        }

        let progress_pct = (self.md_stream_pos as f64
            / self.current_markdown_sample().len().max(1) as f64
            * 100.0) as u8;
        let status = if self.markdown_stream_complete() {
            "Complete".to_string()
        } else {
            format!("Streaming… {progress_pct}%")
        };
        let title = format!("Markdown · {status}");

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title.as_str())
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::MARKDOWN));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let md = self.current_markdown_sample();
        let end = self.md_stream_pos.min(md.len());
        let fragment = &md[..end];
        let mut rendered = self.md_renderer.render_streaming(fragment);

        if !self.markdown_stream_complete() {
            let cursor = Span::styled("▌", Style::new().fg(theme::accent::PRIMARY).blink());
            let mut lines: Vec<Line> = rendered.lines().to_vec();
            if let Some(last_line) = lines.last_mut() {
                last_line.push_span(cursor);
            } else {
                lines.push(Line::from_spans([cursor]));
            }
            rendered = Text::from_lines(lines);
        }

        Paragraph::new(rendered)
            .wrap(WrapMode::Word)
            .render(inner, frame);
    }

    /// Render text effects showcase using complex GFM samples.
    fn render_text_effects(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 2 {
            return;
        }

        let demo = self.current_effect_demo();
        let title = format!(
            "Text FX · {} ({}/{})",
            demo.name,
            self.effect_index + 1,
            EFFECT_DEMOS.len()
        );

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title.as_str())
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::WIDGET_GALLERY));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let rows = Flex::vertical()
            .constraints([
                Constraint::Fixed(1),
                Constraint::Min(1),
                Constraint::Fixed(1),
            ])
            .split(inner);

        let sample = EFFECT_GFM_SAMPLES[self.effect_index % EFFECT_GFM_SAMPLES.len()];
        let header = format!(
            "Sample {} of {}",
            (self.effect_index % EFFECT_GFM_SAMPLES.len()) + 1,
            EFFECT_GFM_SAMPLES.len()
        );
        Paragraph::new(header)
            .style(theme::muted())
            .render(rows[0], frame);

        if !rows[1].is_empty() {
            let max_width = rows[1].width;
            let max_lines = rows[1].height;
            let mut lines = Vec::new();
            for raw in sample.lines() {
                if lines.len() as u16 >= max_lines {
                    break;
                }
                let clipped = truncate_to_width(raw, max_width);
                lines.push(clipped);
            }
            let text_len: usize = lines.iter().map(|l| l.chars().count()).sum();
            let effect = self.build_effect(demo.kind, text_len);
            let styled = StyledMultiLine::new(lines)
                .effect(effect)
                .base_color(theme::fg::PRIMARY.into())
                .time(self.time)
                .seed(self.tick_count);
            styled.render(rows[1], frame);
        }

        Paragraph::new("e: next effect")
            .style(theme::muted())
            .render(rows[2], frame);
    }

    /// Render activity feed showing recent simulated events.
    fn render_activity_feed(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() || area.height < 3 {
            return;
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Activity")
            .title_alignment(Alignment::Center)
            .style(Style::new().fg(theme::screen_accent::ADVANCED));

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        // Get recent alerts from simulated data
        let max_items = inner.height as usize;
        let alerts: Vec<_> = self
            .simulated_data
            .alerts
            .iter()
            .rev()
            .take(max_items)
            .collect();

        for (i, alert) in alerts.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }

            let y = inner.y + i as u16;

            let (label, indicator, color, effect) = match alert.severity {
                AlertSeverity::Error => (
                    "CRIT",
                    "✖",
                    theme::intent::error_text(),
                    TextEffect::PulsingGlow {
                        color: PackedRgba::rgb(255, 80, 100),
                        speed: 1.6,
                    },
                ),
                AlertSeverity::Warning => (
                    "WARN",
                    "▲",
                    theme::intent::warning_text(),
                    TextEffect::Pulse {
                        speed: 1.4,
                        min_alpha: 0.35,
                    },
                ),
                AlertSeverity::Info => (
                    "INFO",
                    "●",
                    theme::intent::info_text(),
                    TextEffect::ColorWave {
                        color1: theme::accent::PRIMARY.into(),
                        color2: theme::accent::ACCENT_8.into(),
                        speed: 1.1,
                        wavelength: 6.0,
                    },
                ),
            };

            // Format timestamp as MM:SS
            let ts_secs = (alert.timestamp / 10) % 3600;
            let ts_min = ts_secs / 60;
            let ts_sec = ts_secs % 60;
            let time_str = format!("{:02}:{:02}", ts_min, ts_sec);

            let prefix_plain = format!("{indicator} {label} {time_str} · ");
            let prefix_width = UnicodeWidthStr::width(prefix_plain.as_str()) as u16;
            let prefix_area = Rect::new(inner.x, y, prefix_width.min(inner.width), 1);

            let prefix_line = Line::from_spans([
                Span::styled(format!("{indicator} "), Style::new().fg(color).bold()),
                Span::styled(format!("{label} "), Style::new().fg(color).bold()),
                Span::styled(time_str.clone(), theme::muted()),
                Span::styled(" · ", theme::muted()),
            ]);

            Paragraph::new(Text::from_lines([prefix_line])).render(prefix_area, frame);

            if inner.width <= prefix_width + 1 {
                continue;
            }

            let msg_area = Rect::new(
                inner.x + prefix_width,
                y,
                inner.width.saturating_sub(prefix_width),
                1,
            );
            let msg = truncate_to_width(&alert.message, msg_area.width);
            let styled = StyledText::new(msg)
                .base_color(color)
                .effect(effect)
                .time(self.time)
                .seed(alert.timestamp);
            styled.render(msg_area, frame);
        }

        // If no alerts yet, show placeholder
        if alerts.is_empty() {
            let styled = StyledText::new("All systems nominal")
                .effect(TextEffect::AnimatedGradient {
                    gradient: ColorGradient::ocean(),
                    speed: 0.4,
                })
                .time(self.time);
            styled.render(inner, frame);
        }
    }

    /// Render navigation footer.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() {
            return;
        }

        let hint = "c:code e:fx m:md | 1-9:screens | Tab:next | t:theme | ?:help | q:quit";
        Paragraph::new(hint)
            .style(Style::new().fg(theme::fg::MUTED).bg(theme::alpha::SURFACE))
            .render(area, frame);
    }

    // =========================================================================
    // Layout Variants
    // =========================================================================

    /// Large layout (100x30+).
    fn render_large(&self, frame: &mut Frame, area: Rect) {
        // Main vertical split: header, content, footer
        let main = Flex::vertical()
            .constraints([
                Constraint::Fixed(1), // Header
                Constraint::Min(10),  // Content
                Constraint::Fixed(1), // Footer
            ])
            .split(area);

        self.render_header(frame, main[0]);
        self.render_footer(frame, main[2]);

        // Content area: split into top row and bottom row
        let content_rows = Flex::vertical()
            .constraints([Constraint::Percentage(55.0), Constraint::Percentage(45.0)])
            .split(main[1]);

        // Top row: 4 panels (plasma, charts, code, info)
        let top_cols = Flex::horizontal()
            .constraints([
                Constraint::Percentage(20.0),
                Constraint::Percentage(30.0),
                Constraint::Percentage(30.0),
                Constraint::Percentage(20.0),
            ])
            .split(content_rows[0]);

        self.render_plasma(frame, top_cols[0]);
        self.render_sparklines(frame, top_cols[1]);
        self.render_code(frame, top_cols[2]);
        self.render_info(frame, top_cols[3], (area.width, area.height));

        // Bottom row: stats, activity feed, markdown
        let bottom_cols = Flex::horizontal()
            .constraints([
                Constraint::Percentage(25.0),
                Constraint::Percentage(40.0),
                Constraint::Percentage(35.0),
            ])
            .split(content_rows[1]);

        self.render_text_effects(frame, bottom_cols[0]);
        self.render_activity_feed(frame, bottom_cols[1]);
        self.render_markdown(frame, bottom_cols[2]);
    }

    /// Medium layout (70x20+).
    fn render_medium(&self, frame: &mut Frame, area: Rect) {
        let main = Flex::vertical()
            .constraints([
                Constraint::Fixed(1), // Header
                Constraint::Min(8),   // Content
                Constraint::Fixed(1), // Footer
            ])
            .split(area);

        self.render_header(frame, main[0]);
        self.render_footer(frame, main[2]);

        // Content: top row with panels, bottom row with stats + activity
        let content_rows = Flex::vertical()
            .constraints([Constraint::Percentage(60.0), Constraint::Percentage(40.0)])
            .split(main[1]);

        // Top row: 3 panels
        let top_cols = Flex::horizontal()
            .constraints([
                Constraint::Percentage(25.0),
                Constraint::Percentage(40.0),
                Constraint::Percentage(35.0),
            ])
            .split(content_rows[0]);

        self.render_plasma(frame, top_cols[0]);
        self.render_sparklines(frame, top_cols[1]);

        // Combined code + info in the third column
        let right_split = Flex::vertical()
            .constraints([Constraint::Percentage(60.0), Constraint::Percentage(40.0)])
            .split(top_cols[2]);

        self.render_code(frame, right_split[0]);
        self.render_info(frame, right_split[1], (area.width, area.height));

        // Bottom row: text effects, activity feed, markdown stream
        let bottom_cols = Flex::horizontal()
            .constraints([
                Constraint::Percentage(30.0),
                Constraint::Percentage(40.0),
                Constraint::Percentage(30.0),
            ])
            .split(content_rows[1]);

        self.render_text_effects(frame, bottom_cols[0]);
        self.render_activity_feed(frame, bottom_cols[1]);
        self.render_markdown(frame, bottom_cols[2]);
    }

    /// Tiny layout (<70x20).
    fn render_tiny(&self, frame: &mut Frame, area: Rect) {
        let main = Flex::vertical()
            .constraints([
                Constraint::Fixed(1), // Header
                Constraint::Min(4),   // Content
                Constraint::Fixed(1), // Footer
            ])
            .split(area);

        self.render_header(frame, main[0]);

        // Compact footer
        let hint = "t:theme q:quit";
        Paragraph::new(hint)
            .style(Style::new().fg(theme::fg::MUTED).bg(theme::alpha::SURFACE))
            .render(main[2], frame);

        // Content: two columns
        let cols = Flex::horizontal()
            .constraints([Constraint::Percentage(35.0), Constraint::Percentage(65.0)])
            .split(main[1]);

        // Left: plasma
        self.render_plasma(frame, cols[0]);

        // Right: compact info with sparklines
        let right_rows = Flex::vertical()
            .constraints([Constraint::Min(1), Constraint::Fixed(2)])
            .split(cols[1]);

        // Sparklines (just CPU and MEM)
        if !right_rows[0].is_empty() {
            let spark_rows = Flex::vertical()
                .constraints([Constraint::Fixed(1), Constraint::Fixed(1)])
                .split(right_rows[0]);

            let cpu_data: Vec<f64> = self.simulated_data.cpu_history.iter().copied().collect();
            let mem_data: Vec<f64> = self.simulated_data.memory_history.iter().copied().collect();

            if !spark_rows[0].is_empty() && !cpu_data.is_empty() {
                let label_w = 4.min(spark_rows[0].width);
                Paragraph::new("CPU ")
                    .style(Style::new().fg(theme::fg::SECONDARY))
                    .render(
                        Rect::new(spark_rows[0].x, spark_rows[0].y, label_w, 1),
                        frame,
                    );
                let spark_area = Rect::new(
                    spark_rows[0].x + label_w,
                    spark_rows[0].y,
                    spark_rows[0].width.saturating_sub(label_w),
                    1,
                );
                if !spark_area.is_empty() {
                    Sparkline::new(&cpu_data)
                        .style(Style::new().fg(theme::accent::PRIMARY))
                        .render(spark_area, frame);
                }
            }

            if spark_rows.len() > 1 && !spark_rows[1].is_empty() && !mem_data.is_empty() {
                let label_w = 4.min(spark_rows[1].width);
                Paragraph::new("MEM ")
                    .style(Style::new().fg(theme::fg::SECONDARY))
                    .render(
                        Rect::new(spark_rows[1].x, spark_rows[1].y, label_w, 1),
                        frame,
                    );
                let spark_area = Rect::new(
                    spark_rows[1].x + label_w,
                    spark_rows[1].y,
                    spark_rows[1].width.saturating_sub(label_w),
                    1,
                );
                if !spark_area.is_empty() {
                    Sparkline::new(&mem_data)
                        .style(Style::new().fg(theme::accent::SUCCESS))
                        .render(spark_area, frame);
                }
            }
        }

        // Compact info
        if !right_rows[1].is_empty() {
            let info = format!("FPS:{:.0} {}x{}", self.fps, area.width, area.height);
            Paragraph::new(info)
                .style(Style::new().fg(theme::fg::MUTED))
                .render(right_rows[1], frame);
        }
    }
}

/// Helper to render Text widget line by line.
fn render_text(frame: &mut Frame, area: Rect, text: &Text) {
    if area.is_empty() {
        return;
    }

    let lines = text.lines();
    for (i, line) in lines.iter().enumerate() {
        if i as u16 >= area.height {
            break;
        }
        let line_y = area.y + i as u16;
        // Render each span in the line
        let mut x_offset = 0u16;
        for span in line.spans() {
            let text_len = span.content.chars().count() as u16;
            if x_offset >= area.width {
                break;
            }
            let span_area = Rect::new(
                area.x + x_offset,
                line_y,
                (area.width - x_offset).min(text_len),
                1,
            );
            let style = span.style.unwrap_or_default();
            Paragraph::new(span.content.as_ref())
                .style(style)
                .render(span_area, frame);
            x_offset += text_len;
        }
    }
}

fn truncate_to_width(text: &str, max_width: u16) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut width = 0usize;
    let max = max_width as usize;
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + w > max {
            break;
        }
        out.push(ch);
        width += w;
    }
    out
}

impl Screen for Dashboard {
    type Message = Event;

    fn update(&mut self, event: &Event) -> Cmd<Self::Message> {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                // Reset animations
                KeyCode::Char('r') => {
                    self.tick_count = 0;
                    self.time = 0.0;
                    self.reset_markdown_stream();
                }
                // Cycle code samples
                KeyCode::Char('c') => {
                    self.code_index = (self.code_index + 1) % CODE_SAMPLES.len();
                }
                // Cycle text effects (also rotates sample)
                KeyCode::Char('e') => {
                    self.effect_index = (self.effect_index + 1) % EFFECT_DEMOS.len();
                }
                // Cycle markdown samples + restart stream
                KeyCode::Char('m') => {
                    self.md_sample_index = (self.md_sample_index + 1) % DASH_MARKDOWN_SAMPLES.len();
                    self.reset_markdown_stream();
                }
                _ => {}
            }
        }

        Cmd::None
    }

    fn tick(&mut self, tick_count: u64) {
        self.tick_count = tick_count;
        self.time = tick_count as f64 * 0.1; // 100ms per tick
        self.tick_markdown_stream();
        self.simulated_data.tick(tick_count);
        self.update_fps();
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() {
            return;
        }

        // Choose layout based on terminal size
        let _layout = match (area.width, area.height) {
            (w, h) if w >= 100 && h >= 30 => {
                self.render_large(frame, area);
                "large"
            }
            (w, h) if w >= 70 && h >= 20 => {
                self.render_medium(frame, area);
                "medium"
            }
            _ => {
                self.render_tiny(frame, area);
                "tiny"
            }
        };
        crate::debug_render!(
            "dashboard",
            "layout={_layout}, area={}x{}, tick={}",
            area.width,
            area.height,
            self.tick_count
        );
    }

    fn keybindings(&self) -> Vec<HelpEntry> {
        vec![
            HelpEntry {
                key: "r",
                action: "Reset animations",
            },
            HelpEntry {
                key: "c",
                action: "Cycle code language",
            },
            HelpEntry {
                key: "e",
                action: "Cycle text effects",
            },
            HelpEntry {
                key: "m",
                action: "Cycle markdown sample",
            },
            HelpEntry {
                key: "t",
                action: "Cycle theme",
            },
        ]
    }

    fn title(&self) -> &'static str {
        "Dashboard"
    }

    fn tab_label(&self) -> &'static str {
        "Dashboard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftui_render::grapheme_pool::GraphemePool;

    #[test]
    fn dashboard_renders_header() {
        let mut state = Dashboard::new();
        state.tick(10);

        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(120, 40, &mut pool);

        state.view(&mut frame, Rect::new(0, 0, 120, 40));

        // Header should be present (first row should not be empty)
        let mut has_content = false;
        for x in 0..120 {
            if let Some(cell) = frame.buffer.get(x, 0)
                && cell.content.as_char() != Some(' ')
                && !cell.is_empty()
            {
                has_content = true;
                break;
            }
        }
        assert!(has_content, "Header should render content");
    }

    #[test]
    fn dashboard_shows_metrics() {
        let mut state = Dashboard::new();
        // Populate some history
        for t in 0..50 {
            state.tick(t);
        }

        assert!(
            !state.simulated_data.cpu_history.is_empty(),
            "CPU history should be populated"
        );
        assert!(
            !state.simulated_data.memory_history.is_empty(),
            "Memory history should be populated"
        );
    }

    #[test]
    fn dashboard_sparklines_update() {
        let mut state = Dashboard::new();
        let initial_len = state.simulated_data.cpu_history.len();

        state.tick(100);

        assert!(
            state.simulated_data.cpu_history.len() > initial_len,
            "CPU history should grow on tick"
        );
    }

    #[test]
    fn dashboard_handles_resize() {
        let state = Dashboard::new();

        // Small terminal
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(40, 15, &mut pool);
        state.view(&mut frame, Rect::new(0, 0, 40, 15));
        // Should not panic

        // Large terminal
        let mut pool2 = GraphemePool::new();
        let mut frame2 = Frame::new(200, 60, &mut pool2);
        state.view(&mut frame2, Rect::new(0, 0, 200, 60));
        // Should not panic
    }

    #[test]
    fn dashboard_activity_feed_populates() {
        let mut state = Dashboard::new();

        // Run enough ticks to generate alerts (ALERT_INTERVAL is 20)
        for t in 0..100 {
            state.tick(t);
        }

        // Should have alerts
        assert!(
            !state.simulated_data.alerts.is_empty(),
            "Alerts should be generated after sufficient ticks"
        );
    }

    #[test]
    fn dashboard_text_effects_renders() {
        let state = Dashboard::new();
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(50, 5, &mut pool);

        // Render just the text effects panel
        state.render_text_effects(&mut frame, Rect::new(0, 0, 50, 5));

        // Check that content was rendered (border + stats)
        let top_left = frame.buffer.get(0, 0).and_then(|c| c.content.as_char());
        assert!(
            top_left.is_some(),
            "Text effects panel should render border character"
        );
    }

    #[test]
    fn dashboard_activity_feed_renders() {
        let mut state = Dashboard::new();
        // Generate some alerts first
        for t in 0..100 {
            state.tick(t);
        }

        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(60, 10, &mut pool);

        // Render just the activity feed panel
        state.render_activity_feed(&mut frame, Rect::new(0, 0, 60, 10));

        // Check that border was rendered
        let top_left = frame.buffer.get(0, 0).and_then(|c| c.content.as_char());
        assert!(
            top_left.is_some(),
            "Activity feed should render border character"
        );
    }

    #[test]
    fn dashboard_tick_updates_time() {
        let mut state = Dashboard::new();
        assert_eq!(state.tick_count, 30); // Pre-populated in new()

        state.tick(50);
        assert_eq!(state.tick_count, 50);
        assert!(
            (state.time - 5.0).abs() < f64::EPSILON,
            "time should be tick * 0.1"
        );
    }

    #[test]
    fn dashboard_empty_area_handled() {
        let state = Dashboard::new();
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(1, 1, &mut pool);

        // Should not panic with empty area
        state.view(&mut frame, Rect::new(0, 0, 0, 0));
    }

    #[test]
    fn dashboard_layout_large_threshold() {
        let state = Dashboard::new();
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(100, 30, &mut pool);

        // At exactly 100x30, should use large layout
        state.view(&mut frame, Rect::new(0, 0, 100, 30));
        // Should not panic
    }

    #[test]
    fn dashboard_layout_medium_threshold() {
        let state = Dashboard::new();
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(70, 20, &mut pool);

        // At exactly 70x20, should use medium layout
        state.view(&mut frame, Rect::new(0, 0, 70, 20));
        // Should not panic
    }

    #[test]
    fn dashboard_layout_tiny_threshold() {
        let state = Dashboard::new();
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(50, 15, &mut pool);

        // Below medium thresholds, should use tiny layout
        state.view(&mut frame, Rect::new(0, 0, 50, 15));
        // Should not panic
    }
}
