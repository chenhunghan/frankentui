#![forbid(unsafe_code)]

//! Macro Recorder screen — record, replay, and visualize input macros.
//!
//! Demonstrates:
//! - `FilteredEventRecorder` for live event capture
//! - Deterministic playback with speed control
//! - Timeline and scenario runner panels

use std::time::Instant;

use ftui_core::event::{Event, KeyCode, KeyEvent, KeyEventKind, Modifiers, MouseEvent};
use ftui_core::geometry::Rect;
use ftui_layout::{Constraint, Flex};
use ftui_render::frame::Frame;
use ftui_runtime::Cmd;
use ftui_runtime::input_macro::{FilteredEventRecorder, InputMacro, RecordingFilter};
use ftui_style::Style;
use ftui_text::{Line, Span, Text};
use ftui_widgets::Widget;
use ftui_widgets::block::{Alignment, Block};
use ftui_widgets::borders::{BorderType, Borders};
use ftui_widgets::paragraph::Paragraph;

use super::{HelpEntry, Screen};
use crate::theme;

const TICK_MS: u64 = 100;
const SPEED_MIN: f64 = 0.25;
const SPEED_MAX: f64 = 4.0;
const SPEED_STEP: f64 = 0.25;
const MAX_EVENT_LINES: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
enum UiState {
    Idle,
    Recording,
    Stopped,
    Playing,
    Error(String),
}

#[derive(Debug, Clone)]
struct PlaybackState {
    playhead: usize,
    elapsed_ms: f64,
    next_due_ms: u64,
    last_tick: u64,
}

#[derive(Debug, Clone, Copy)]
struct ScenarioInfo {
    name: &'static str,
    description: &'static str,
}

const SCENARIOS: &[ScenarioInfo] = &[
    ScenarioInfo {
        name: "Tab Tour",
        description: "Cycle screens, toggle help, return to recorder",
    },
    ScenarioInfo {
        name: "Search Flow",
        description: "Open Shakespeare screen, search for a phrase",
    },
    ScenarioInfo {
        name: "Layout Lab",
        description: "Adjust constraints and switch grid presets",
    },
];

pub struct MacroRecorderScreen {
    state: UiState,
    recorder: Option<FilteredEventRecorder>,
    macro_data: Option<InputMacro>,
    playback: Option<PlaybackState>,
    pending_playback: Vec<Event>,
    recording_started: Option<Instant>,
    recorded_events: usize,
    filtered_events: usize,
    speed: f64,
    looping: bool,
    terminal_size: (u16, u16),
    last_tick_count: u64,
}

impl Default for MacroRecorderScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRecorderScreen {
    pub fn new() -> Self {
        Self {
            state: UiState::Idle,
            recorder: None,
            macro_data: None,
            playback: None,
            pending_playback: Vec::new(),
            recording_started: None,
            recorded_events: 0,
            filtered_events: 0,
            speed: 1.0,
            looping: false,
            terminal_size: (80, 24),
            last_tick_count: 0,
        }
    }

    pub fn record_event(&mut self, event: &Event, filter_controls: bool) {
        let Some(recorder) = &mut self.recorder else {
            return;
        };
        if !recorder.is_recording() {
            return;
        }
        if filter_controls && is_control_key(event) {
            return;
        }
        if recorder.record(event) {
            self.recorded_events = recorder.event_count();
            self.filtered_events = recorder.filtered_count();
        }
    }

    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    pub fn drain_playback_events(&mut self) -> Vec<Event> {
        if self.pending_playback.is_empty() {
            return Vec::new();
        }
        std::mem::take(&mut self.pending_playback)
    }

    fn start_recording(&mut self) {
        let filter = RecordingFilter::keys_only();
        let recorder = FilteredEventRecorder::new("macro", filter)
            .with_terminal_size(self.terminal_size.0, self.terminal_size.1);
        self.recorder = Some(recorder);
        if let Some(recorder) = &mut self.recorder {
            recorder.start();
        }
        self.recorded_events = 0;
        self.filtered_events = 0;
        self.macro_data = None;
        self.playback = None;
        self.recording_started = Some(Instant::now());
        self.state = UiState::Recording;
    }

    fn stop_recording(&mut self) {
        let Some(recorder) = self.recorder.take() else {
            return;
        };
        let recorded = recorder.event_count();
        let filtered = recorder.filtered_count();
        let macro_data = recorder.finish();
        self.recorded_events = recorded;
        self.filtered_events = filtered;
        self.macro_data = Some(macro_data);
        self.playback = None;
        self.recording_started = None;
        self.state = UiState::Stopped;
    }

    fn start_playback(&mut self, tick_count: u64) {
        let Some(macro_data) = &self.macro_data else {
            self.state = UiState::Error("No macro recorded".to_string());
            return;
        };
        if macro_data.is_empty() {
            self.state = UiState::Error("Macro is empty".to_string());
            return;
        }
        let next_due_ms = macro_data
            .events()
            .first()
            .map(|e| e.delay.as_millis() as u64)
            .unwrap_or(0);
        self.playback = Some(PlaybackState {
            playhead: 0,
            elapsed_ms: 0.0,
            next_due_ms,
            last_tick: tick_count,
        });
        self.state = UiState::Playing;
    }

    fn pause_playback(&mut self) {
        if self.playback.is_some() {
            self.state = UiState::Stopped;
        }
    }

    fn stop_playback(&mut self) {
        self.playback = None;
        self.state = UiState::Stopped;
    }

    fn toggle_playback(&mut self, tick_count: u64) {
        match self.state {
            UiState::Playing => self.pause_playback(),
            UiState::Stopped => {
                if self.playback.is_some() {
                    self.state = UiState::Playing;
                    if let Some(playback) = &mut self.playback {
                        playback.last_tick = tick_count;
                    }
                } else {
                    self.start_playback(tick_count);
                }
            }
            UiState::Idle => self.start_playback(tick_count),
            UiState::Recording => {}
            UiState::Error(_) => {
                if self.macro_data.is_some() {
                    self.state = UiState::Stopped;
                } else {
                    self.state = UiState::Idle;
                }
            }
        }
    }

    fn toggle_loop(&mut self) {
        self.looping = !self.looping;
    }

    fn adjust_speed(&mut self, delta: f64) {
        let mut speed = self.speed + delta;
        speed = speed.clamp(SPEED_MIN, SPEED_MAX);
        self.speed = speed;
    }

    fn handle_controls(&mut self, event: &Event) {
        let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        else {
            return;
        };

        let (code, modifiers) = (*code, *modifiers);

        match (code, modifiers) {
            (KeyCode::Char('r'), Modifiers::NONE) => {
                if self.state == UiState::Recording {
                    self.stop_recording();
                } else {
                    self.start_recording();
                }
            }
            (KeyCode::Char('p'), Modifiers::NONE) => self.toggle_playback(self.last_tick_count),
            (KeyCode::Char('l'), Modifiers::NONE) => self.toggle_loop(),
            (KeyCode::Char('+'), Modifiers::NONE) | (KeyCode::Char('='), Modifiers::NONE) => {
                self.adjust_speed(SPEED_STEP)
            }
            (KeyCode::Char('-'), Modifiers::NONE) => self.adjust_speed(-SPEED_STEP),
            (KeyCode::Escape, _) => {
                if self.state == UiState::Recording {
                    self.stop_recording();
                } else if self.state == UiState::Playing {
                    self.stop_playback();
                } else if matches!(self.state, UiState::Error(_)) {
                    if self.macro_data.is_some() {
                        self.state = UiState::Stopped;
                    } else {
                        self.state = UiState::Idle;
                    }
                }
            }
            _ => {}
        }
    }

    fn playback_tick(&mut self, tick_count: u64) {
        if self.state != UiState::Playing {
            return;
        }
        let Some(macro_data) = &self.macro_data else {
            return;
        };
        let Some(playback) = &mut self.playback else {
            return;
        };

        let delta_ticks = tick_count.saturating_sub(playback.last_tick).max(1);
        playback.last_tick = tick_count;
        playback.elapsed_ms += delta_ticks as f64 * TICK_MS as f64 * self.speed;

        let events = macro_data.events();
        while playback.playhead < events.len() && playback.elapsed_ms >= playback.next_due_ms as f64
        {
            let timed = &events[playback.playhead];
            self.pending_playback.push(timed.event.clone());
            playback.playhead += 1;
            if playback.playhead < events.len() {
                playback.next_due_ms += events[playback.playhead].delay.as_millis() as u64;
            }
        }

        if playback.playhead >= events.len() {
            if self.looping {
                playback.playhead = 0;
                playback.elapsed_ms = 0.0;
                playback.next_due_ms = events
                    .first()
                    .map(|e| e.delay.as_millis() as u64)
                    .unwrap_or(0);
                playback.last_tick = tick_count;
            } else {
                self.state = UiState::Stopped;
                self.playback = None;
            }
        }
    }

    fn render_controls_panel(&self, frame: &mut Frame, area: Rect) {
        let border_style = Style::new().fg(theme::screen_accent::ADVANCED);
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Macro Recorder")
            .title_alignment(Alignment::Center)
            .style(border_style);

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let state_label = match &self.state {
            UiState::Idle => "Idle",
            UiState::Recording => "Recording",
            UiState::Stopped => {
                if self.playback.is_some() {
                    "Paused"
                } else {
                    "Stopped"
                }
            }
            UiState::Playing => "Playing",
            UiState::Error(_) => "Error",
        };

        let state_style = match &self.state {
            UiState::Recording => Style::new().fg(theme::accent::ERROR).bold(),
            UiState::Playing => Style::new().fg(theme::accent::SUCCESS).bold(),
            UiState::Error(_) => Style::new().fg(theme::accent::ERROR).bold(),
            UiState::Stopped => Style::new().fg(theme::accent::WARNING),
            UiState::Idle => Style::new().fg(theme::fg::MUTED),
        };

        let duration = self
            .recording_started
            .map(|t| t.elapsed())
            .or_else(|| self.macro_data.as_ref().map(|m| m.total_duration()))
            .unwrap_or_default();

        let duration_label = format_duration(duration);

        let event_count = if let Some(macro_data) = &self.macro_data {
            macro_data.len()
        } else {
            self.recorded_events
        };

        let progress = self.playback_progress();

        let mut lines = vec![
            Line::from_spans([
                Span::styled("State: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(state_label, state_style),
                Span::raw("   "),
                Span::styled("Events: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(
                    format!("{}", event_count),
                    Style::new().fg(theme::fg::PRIMARY).bold(),
                ),
                Span::raw("   "),
                Span::styled("Duration: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(duration_label, Style::new().fg(theme::fg::PRIMARY)),
                Span::raw("   "),
                Span::styled("Filtered: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(
                    format!("{}", self.filtered_events),
                    Style::new().fg(theme::fg::MUTED),
                ),
            ]),
            Line::from_spans([
                Span::styled("Controls: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled("r", Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" record/stop  "),
                Span::styled("p", Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" play/pause  "),
                Span::styled("l", Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" loop  "),
                Span::styled("+/-", Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" speed  "),
                Span::styled("Esc", Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" stop"),
            ]),
            Line::from_spans([
                Span::styled("Speed: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(
                    format!("{:.2}x", self.speed),
                    Style::new().fg(theme::fg::PRIMARY),
                ),
                Span::raw("   "),
                Span::styled("Loop: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(
                    if self.looping { "On" } else { "Off" },
                    Style::new().fg(if self.looping {
                        theme::accent::SUCCESS
                    } else {
                        theme::fg::MUTED
                    }),
                ),
                Span::raw("   "),
                Span::styled("Progress: ", Style::new().fg(theme::fg::SECONDARY)),
                Span::styled(
                    format!("{:>3.0}%", progress * 100.0),
                    Style::new().fg(theme::fg::PRIMARY),
                ),
            ]),
        ];

        if let UiState::Error(message) = &self.state {
            lines.push(Line::from_spans([
                Span::styled("Error: ", Style::new().fg(theme::accent::ERROR)),
                Span::styled(message, Style::new().fg(theme::fg::MUTED)),
            ]));
        }

        let lines = Text::from_lines(lines);

        Paragraph::new(lines)
            .style(Style::new().fg(theme::fg::PRIMARY))
            .render(inner, frame);
    }

    fn render_timeline_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Timeline")
            .title_alignment(Alignment::Center)
            .style(theme::content_border());

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let Some(macro_data) = &self.macro_data else {
            Paragraph::new("No macro recorded yet.")
                .style(Style::new().fg(theme::fg::MUTED))
                .render(inner, frame);
            return;
        };

        let events = macro_data.events();
        if events.is_empty() {
            Paragraph::new("Macro is empty.")
                .style(Style::new().fg(theme::fg::MUTED))
                .render(inner, frame);
            return;
        }

        let playhead = self.playback.as_ref().map(|p| p.playhead).unwrap_or(0);
        let max_lines = inner.height as usize;
        let mut lines = Vec::new();

        let visible = MAX_EVENT_LINES.min(max_lines).min(events.len());
        let start = if playhead >= visible {
            playhead + 1 - visible
        } else {
            0
        };

        let mut cumulative_ms: u64 = 0;
        for (idx, timed) in events.iter().enumerate() {
            if idx < start {
                cumulative_ms += timed.delay.as_millis() as u64;
                continue;
            }
            if lines.len() >= visible {
                break;
            }
            cumulative_ms += timed.delay.as_millis() as u64;

            let marker = if self.state == UiState::Playing && idx == playhead {
                ">"
            } else if idx < playhead {
                "*"
            } else {
                " "
            };
            let label = format_event(&timed.event);
            let line = Line::from_spans([
                Span::styled(marker, Style::new().fg(theme::accent::PRIMARY)),
                Span::raw(" "),
                Span::styled(format!("{:03}", idx + 1), Style::new().fg(theme::fg::MUTED)),
                Span::raw("  +"),
                Span::styled(
                    format!("{:>4}ms", timed.delay.as_millis()),
                    Style::new().fg(theme::fg::SECONDARY),
                ),
                Span::raw("  @"),
                Span::styled(
                    format!("{:>5}ms", cumulative_ms),
                    Style::new().fg(theme::fg::MUTED),
                ),
                Span::raw("  "),
                Span::styled(label, Style::new().fg(theme::fg::PRIMARY)),
            ]);
            lines.push(line);
        }

        Paragraph::new(Text::from_lines(lines))
            .style(Style::new().fg(theme::fg::PRIMARY))
            .render(inner, frame);
    }

    fn render_scenarios_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Scenario Runner")
            .title_alignment(Alignment::Center)
            .style(theme::content_border());

        let inner = block.inner(area);
        block.render(area, frame);

        if inner.is_empty() {
            return;
        }

        let mut lines = Vec::new();
        lines.push(Line::from_spans([Span::styled(
            "Preset scenarios (WIP)",
            Style::new().fg(theme::fg::SECONDARY),
        )]));
        lines.push(Line::new());

        for scenario in SCENARIOS {
            lines.push(Line::from_spans([Span::styled(
                scenario.name,
                Style::new().fg(theme::accent::PRIMARY),
            )]));
            lines.push(Line::from_spans([Span::styled(
                format!("  {}", scenario.description),
                Style::new().fg(theme::fg::MUTED),
            )]));
            lines.push(Line::new());
        }

        Paragraph::new(Text::from_lines(lines))
            .style(Style::new().fg(theme::fg::PRIMARY))
            .render(inner, frame);
    }

    fn playback_progress(&self) -> f64 {
        let Some(macro_data) = &self.macro_data else {
            return 0.0;
        };
        let total = macro_data.total_duration().as_millis() as f64;
        if total <= 0.0 {
            return if macro_data.is_empty() { 0.0 } else { 1.0 };
        }
        let elapsed = self.playback.as_ref().map(|p| p.elapsed_ms).unwrap_or(0.0);
        (elapsed / total).clamp(0.0, 1.0)
    }
}

impl Screen for MacroRecorderScreen {
    type Message = ();

    fn update(&mut self, event: &Event) -> Cmd<Self::Message> {
        if let Event::Resize { width, height } = event {
            self.terminal_size = (*width, *height);
        }
        self.handle_controls(event);
        Cmd::None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let sections = Flex::vertical()
            .constraints([Constraint::Fixed(5), Constraint::Min(1)])
            .split(area);

        self.render_controls_panel(frame, sections[0]);

        let bottom = Flex::horizontal()
            .constraints([Constraint::Percentage(60.0), Constraint::Percentage(40.0)])
            .split(sections[1]);

        self.render_timeline_panel(frame, bottom[0]);
        self.render_scenarios_panel(frame, bottom[1]);
    }

    fn keybindings(&self) -> Vec<HelpEntry> {
        vec![
            HelpEntry {
                key: "r",
                action: "Record / Stop",
            },
            HelpEntry {
                key: "p",
                action: "Play / Pause",
            },
            HelpEntry {
                key: "l",
                action: "Toggle loop",
            },
            HelpEntry {
                key: "+/-",
                action: "Adjust speed",
            },
            HelpEntry {
                key: "Esc",
                action: "Stop playback",
            },
        ]
    }

    fn tick(&mut self, tick_count: u64) {
        self.last_tick_count = tick_count;
        self.playback_tick(tick_count);
    }

    fn title(&self) -> &'static str {
        "Macro Recorder"
    }

    fn tab_label(&self) -> &'static str {
        "Macro"
    }
}

fn is_control_key(event: &Event) -> bool {
    let Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
    }) = event
    else {
        return false;
    };

    let (code, modifiers) = (*code, *modifiers);

    matches!(
        (code, modifiers),
        (KeyCode::Char('r'), Modifiers::NONE)
            | (KeyCode::Char('p'), Modifiers::NONE)
            | (KeyCode::Char('l'), Modifiers::NONE)
            | (KeyCode::Char('+'), Modifiers::NONE)
            | (KeyCode::Char('='), Modifiers::NONE)
            | (KeyCode::Char('-'), Modifiers::NONE)
            | (KeyCode::Escape, _)
    )
}

fn format_event(event: &Event) -> String {
    match event {
        Event::Key(key) => format_key_event(key),
        Event::Mouse(mouse) => format_mouse_event(mouse),
        Event::Paste(text) => format!("Paste({} chars)", text.text.len()),
        Event::Resize { width, height } => format!("Resize {}x{}", width, height),
        Event::Focus(focus) => format!("Focus({:?})", focus),
        Event::Clipboard(_) => "Clipboard".to_string(),
        Event::Tick => "Tick".to_string(),
    }
}

fn format_key_event(key: &KeyEvent) -> String {
    let mut parts: Vec<String> = Vec::new();
    if key.modifiers.contains(Modifiers::CTRL) {
        parts.push("Ctrl".to_string());
    }
    if key.modifiers.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if key.modifiers.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if key.modifiers.contains(Modifiers::SUPER) {
        parts.push("Super".to_string());
    }

    let code = match key.code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::F(n) => format!("F{}", n),
        other => format!("{:?}", other),
    };

    parts.push(code);

    if key.kind != KeyEventKind::Press {
        parts.push(format!("{:?}", key.kind));
    }

    parts.join("+")
}

fn format_mouse_event(mouse: &MouseEvent) -> String {
    format!("Mouse({:?} @{}, {})", mouse.kind, mouse.x, mouse.y)
}

fn format_duration(duration: std::time::Duration) -> String {
    let ms = duration.as_millis();
    if ms < 1000 {
        return format!("{}ms", ms);
    }
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}s", secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftui_runtime::input_macro::{MacroMetadata, TimedEvent};

    fn key_event(c: char) -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: Modifiers::NONE,
            kind: KeyEventKind::Press,
        })
    }

    fn macro_with_delays(name: &str, items: &[(char, u64)]) -> InputMacro {
        let mut events = Vec::with_capacity(items.len());
        let mut total = std::time::Duration::ZERO;
        for (ch, delay_ms) in items {
            let delay = std::time::Duration::from_millis(*delay_ms);
            total += delay;
            events.push(TimedEvent::new(key_event(*ch), delay));
        }
        InputMacro::new(
            events,
            MacroMetadata {
                name: name.to_string(),
                terminal_size: (80, 24),
                total_duration: total,
            },
        )
    }

    #[test]
    fn playback_drains_events_in_order_for_zero_delay_macro() {
        let mut screen = MacroRecorderScreen::new();
        screen.macro_data = Some(InputMacro::from_events(
            "zero",
            vec![key_event('a'), key_event('b'), key_event('c')],
        ));

        screen.start_playback(0);
        screen.tick(0);

        let events = screen.drain_playback_events();
        assert_eq!(events, vec![key_event('a'), key_event('b'), key_event('c')]);
        assert_eq!(screen.state, UiState::Stopped);
        assert!(screen.playback.is_none());
    }

    #[test]
    fn playback_speed_affects_due_time_for_delayed_events() {
        // Two events: immediate 'a', then 'b' due at +1000ms.
        let mut screen = MacroRecorderScreen::new();
        screen.speed = 2.0;
        screen.macro_data = Some(macro_with_delays("delayed", &[('a', 0), ('b', 1000)]));

        screen.start_playback(0);

        // Tick 0 advances by at least one tick (100ms) scaled by speed (2x => 200ms),
        // so only the first event should be emitted.
        screen.tick(0);
        assert_eq!(screen.drain_playback_events(), vec![key_event('a')]);

        // Next event is due at 1000ms, which at 2x speed arrives on tick 4:
        // ticks 0..=4 => 5 steps * 100ms * 2 = 1000ms
        for t in 1..4 {
            screen.tick(t);
            assert!(screen.drain_playback_events().is_empty());
        }

        screen.tick(4);
        assert_eq!(screen.drain_playback_events(), vec![key_event('b')]);
        assert_eq!(screen.state, UiState::Stopped);
    }

    #[test]
    fn control_keys_can_be_filtered_from_recording() {
        let mut screen = MacroRecorderScreen::new();
        screen.start_recording();

        screen.record_event(&key_event('a'), true);
        screen.record_event(&key_event('p'), true); // control key -> ignored
        screen.record_event(&key_event('l'), true); // control key -> ignored

        screen.stop_recording();

        let recorded = screen
            .macro_data
            .as_ref()
            .expect("macro_data should be present after stop_recording")
            .bare_events();
        assert_eq!(recorded, vec![key_event('a')]);
    }
}
