#![forbid(unsafe_code)]

//! Shakespeare Library screen — complete works with search and virtualized scroll.

use ftui_core::event::{Event, KeyCode, KeyEventKind, Modifiers};
use ftui_core::geometry::Rect;
use ftui_extras::text_effects::{ColorGradient, Direction, RevealMode, StyledText, TextEffect};
use ftui_layout::{Constraint, Flex};
use ftui_render::cell::PackedRgba;
use ftui_render::frame::Frame;
use ftui_runtime::Cmd;
use ftui_style::{Style, StyleFlags};
use ftui_text::search::search_ascii_case_insensitive;
use ftui_widgets::StatefulWidget;
use ftui_widgets::Widget;
use ftui_widgets::block::{Alignment, Block};
use ftui_widgets::borders::{BorderType, Borders};
use ftui_widgets::input::TextInput;
use ftui_widgets::paragraph::Paragraph;
use ftui_widgets::scrollbar::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ftui_widgets::tree::{Tree, TreeGuides, TreeNode};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{HelpEntry, Screen};
use crate::theme;

/// Embedded complete works of Shakespeare.
const SHAKESPEARE_TEXT: &str = include_str!("../../data/shakespeare.txt");

/// A table-of-contents entry (play/section title and its line number).
struct TocEntry {
    title: String,
    line: usize,
}

/// Shakespeare Library screen state.
pub struct Shakespeare {
    /// All lines of the text, as static string slices.
    lines: Vec<&'static str>,
    /// Current scroll offset (top visible line).
    scroll_offset: usize,
    /// Table of contents entries.
    toc_entries: Vec<TocEntry>,
    /// Tree widget for TOC display.
    toc_tree: Tree,
    /// Search input widget.
    search_input: TextInput,
    /// Whether search bar is focused/visible.
    search_active: bool,
    /// Line indices matching the current search query.
    search_matches: Vec<usize>,
    /// Index into search_matches for current highlighted match.
    current_match: usize,
    /// Viewport height (lines visible), updated each render.
    viewport_height: u16,
    /// Animation tick counter.
    tick_count: u64,
    /// Animation time (seconds).
    time: f64,
}

impl Default for Shakespeare {
    fn default() -> Self {
        Self::new()
    }
}

impl Shakespeare {
    pub fn new() -> Self {
        let lines: Vec<&'static str> = SHAKESPEARE_TEXT.lines().collect();
        let toc_entries = Self::build_toc(&lines);
        let toc_tree = Self::build_tree(&toc_entries);

        let mut state = Self {
            lines,
            scroll_offset: 0,
            toc_entries,
            toc_tree,
            search_input: TextInput::new()
                .with_placeholder("Search... (/ to focus, Esc to close)")
                .with_style(
                    Style::new()
                        .fg(theme::fg::PRIMARY)
                        .bg(theme::alpha::SURFACE),
                )
                .with_placeholder_style(Style::new().fg(theme::fg::MUTED)),
            search_active: false,
            search_matches: Vec::new(),
            current_match: 0,
            viewport_height: 20,
            tick_count: 0,
            time: 0.0,
        };
        state.apply_theme();
        state
    }

    pub fn apply_theme(&mut self) {
        let input_style = Style::new()
            .fg(theme::fg::PRIMARY)
            .bg(theme::alpha::SURFACE);
        let placeholder_style = Style::new().fg(theme::fg::MUTED);
        self.search_input = self
            .search_input
            .clone()
            .with_style(input_style)
            .with_placeholder_style(placeholder_style);
    }

    /// Build table of contents from the text by detecting play titles.
    /// Titles are lines that appear in ALL CAPS and match known patterns.
    fn build_toc(lines: &[&str]) -> Vec<TocEntry> {
        let mut entries = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip empty lines and very short lines
            if trimmed.len() < 5 {
                continue;
            }
            // Detect section headers: lines that are all uppercase letters/spaces/punctuation
            // and appear after a blank line, typical of Gutenberg Shakespeare formatting.
            // Also check for "THE SONNETS", "ACT" headers, play titles, etc.
            if i > 0
                && trimmed.len() > 8
                && trimmed.len() < 80
                && is_title_line(trimmed)
                && (i == 0
                    || lines
                        .get(i.wrapping_sub(1))
                        .is_some_and(|l| l.trim().is_empty()))
            {
                entries.push(TocEntry {
                    title: trimmed.to_owned(),
                    line: i,
                });
            }
        }
        entries
    }

    /// Build a Tree widget from TOC entries.
    fn build_tree(entries: &[TocEntry]) -> Tree {
        let mut root = TreeNode::new("Complete Works").with_expanded(true);
        for entry in entries.iter().take(50) {
            // Truncate long titles for the tree
            let label = if entry.title.len() > 35 {
                format!("{}...", &entry.title[..32])
            } else {
                entry.title.clone()
            };
            root = root.child(TreeNode::new(label));
        }
        Tree::new(root)
            .with_show_root(true)
            .with_guides(TreeGuides::Unicode)
            .with_guide_style(Style::new().fg(theme::fg::MUTED))
            .with_label_style(Style::new().fg(theme::fg::SECONDARY))
            .with_root_style(
                Style::new()
                    .fg(theme::accent::PRIMARY)
                    .attrs(StyleFlags::BOLD),
            )
    }

    fn total_lines(&self) -> usize {
        self.lines.len()
    }

    fn scroll_by(&mut self, delta: i32) {
        let max_offset = self
            .total_lines()
            .saturating_sub(self.viewport_height as usize);
        if delta < 0 {
            self.scroll_offset = self
                .scroll_offset
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.scroll_offset = (self.scroll_offset + delta as usize).min(max_offset);
        }
    }

    fn scroll_to(&mut self, line: usize) {
        let max_offset = self
            .total_lines()
            .saturating_sub(self.viewport_height as usize);
        self.scroll_offset = line.min(max_offset);
    }

    fn perform_search(&mut self) {
        let query = self.search_input.value().to_owned();
        self.search_matches.clear();
        self.current_match = 0;
        if query.len() < 2 {
            return;
        }
        for (i, line) in self.lines.iter().enumerate() {
            let results = search_ascii_case_insensitive(line, &query);
            if !results.is_empty() {
                self.search_matches.push(i);
            }
        }
        // Jump to first match
        if let Some(&first) = self.search_matches.first() {
            self.scroll_to(first.saturating_sub(3));
        }
    }

    fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        let line = self.search_matches[self.current_match];
        self.scroll_to(line.saturating_sub(3));
    }

    fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match =
            (self.current_match + self.search_matches.len() - 1) % self.search_matches.len();
        let line = self.search_matches[self.current_match];
        self.scroll_to(line.saturating_sub(3));
    }

    /// Determine the current play/section name based on scroll position.
    fn current_section(&self) -> &str {
        let mut section = "Preamble";
        for entry in &self.toc_entries {
            if entry.line <= self.scroll_offset {
                section = &entry.title;
            } else {
                break;
            }
        }
        section
    }
}

/// Check if a line looks like a title (mostly uppercase with allowed punctuation).
fn is_title_line(s: &str) -> bool {
    let alpha_count = s.chars().filter(|c| c.is_alphabetic()).count();
    if alpha_count < 4 {
        return false;
    }
    let upper_count = s.chars().filter(|c| c.is_uppercase()).count();
    // At least 80% uppercase letters
    upper_count * 100 / alpha_count.max(1) >= 80
}

impl Screen for Shakespeare {
    type Message = Event;

    fn update(&mut self, event: &Event) -> Cmd<Self::Message> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return Cmd::None;
            }

            // Search mode input handling
            if self.search_active {
                match (key.code, key.modifiers) {
                    (KeyCode::Escape, _) => {
                        self.search_active = false;
                        self.search_input.set_focused(false);
                        return Cmd::None;
                    }
                    (KeyCode::Enter, _) | (KeyCode::Down, _) => {
                        if self.search_matches.is_empty() {
                            self.perform_search();
                        } else {
                            self.next_match();
                        }
                        return Cmd::None;
                    }
                    (KeyCode::Up, _) => {
                        self.prev_match();
                        return Cmd::None;
                    }
                    (KeyCode::Tab, _) => {
                        self.next_match();
                        return Cmd::None;
                    }
                    _ => {
                        let handled = self.search_input.handle_event(event);
                        if handled {
                            // Live search on each keystroke
                            self.perform_search();
                        }
                        return Cmd::None;
                    }
                }
            }

            // Normal mode keybindings
            match (key.code, key.modifiers) {
                (KeyCode::Char('/'), Modifiers::NONE) => {
                    self.search_active = true;
                    self.search_input.set_focused(true);
                }
                (KeyCode::Char('n'), Modifiers::NONE) => self.next_match(),
                (KeyCode::Char('N'), Modifiers::NONE) | (KeyCode::Char('n'), Modifiers::SHIFT) => {
                    self.prev_match();
                }
                (KeyCode::Char('j'), Modifiers::NONE) | (KeyCode::Down, Modifiers::NONE) => {
                    self.scroll_by(1);
                }
                (KeyCode::Char('k'), Modifiers::NONE) | (KeyCode::Up, Modifiers::NONE) => {
                    self.scroll_by(-1);
                }
                (KeyCode::Char('d'), Modifiers::CTRL) | (KeyCode::PageDown, _) => {
                    self.scroll_by(self.viewport_height as i32 / 2);
                }
                (KeyCode::Char('u'), Modifiers::CTRL) | (KeyCode::PageUp, _) => {
                    self.scroll_by(-(self.viewport_height as i32 / 2));
                }
                (KeyCode::Home, _) | (KeyCode::Char('g'), Modifiers::NONE) => {
                    self.scroll_to(0);
                }
                (KeyCode::End, _) | (KeyCode::Char('G'), Modifiers::NONE) => {
                    self.scroll_to(self.total_lines());
                }
                _ => {}
            }
        }
        Cmd::None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        if area.height < 5 || area.width < 30 {
            Paragraph::new("Terminal too small")
                .style(theme::muted())
                .render(area, frame);
            return;
        }

        // Vertical layout: search bar (if active) + body + status
        let v_chunks = Flex::vertical()
            .constraints(if self.search_active {
                vec![
                    Constraint::Fixed(3),
                    Constraint::Min(4),
                    Constraint::Fixed(1),
                ]
            } else {
                vec![Constraint::Min(4), Constraint::Fixed(1)]
            })
            .split(area);

        let (body_area, status_area) = if self.search_active {
            self.render_search_bar(frame, v_chunks[0]);
            (v_chunks[1], v_chunks[2])
        } else {
            (v_chunks[0], v_chunks[1])
        };

        // Body: left (text) + right (TOC)
        let h_chunks = Flex::horizontal()
            .constraints([Constraint::Percentage(72.0), Constraint::Percentage(28.0)])
            .split(body_area);

        self.render_text_panel(frame, h_chunks[0]);
        self.render_toc_panel(frame, h_chunks[1]);
        self.render_status_bar(frame, status_area);
    }

    fn keybindings(&self) -> Vec<HelpEntry> {
        vec![
            HelpEntry {
                key: "/",
                action: "Search",
            },
            HelpEntry {
                key: "Enter/↓/Tab",
                action: "Next match (while searching)",
            },
            HelpEntry {
                key: "↑",
                action: "Prev match (while searching)",
            },
            HelpEntry {
                key: "n/N",
                action: "Next/prev match",
            },
            HelpEntry {
                key: "j/k",
                action: "Scroll up/down",
            },
            HelpEntry {
                key: "g/G",
                action: "Top/bottom",
            },
            HelpEntry {
                key: "PgUp/PgDn",
                action: "Page scroll",
            },
        ]
    }

    fn tick(&mut self, tick_count: u64) {
        self.tick_count = tick_count;
        self.time = tick_count as f64 * 0.1;
    }

    fn title(&self) -> &'static str {
        "Shakespeare Library"
    }

    fn tab_label(&self) -> &'static str {
        "Shakespeare"
    }
}

impl Shakespeare {
    fn render_search_bar(&self, frame: &mut Frame, area: Rect) {
        if area.height < 2 {
            self.search_input.render(area, frame);
            return;
        }

        let rows = Flex::vertical()
            .constraints([
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(1),
            ])
            .split(area);

        // Header row: animated title + controls
        let header_cols = Flex::horizontal()
            .constraints([Constraint::Min(10), Constraint::Fixed(32)])
            .split(rows[0]);

        let title = StyledText::new("LIVE SEARCH")
            .effect(TextEffect::AnimatedGradient {
                gradient: ColorGradient::cyberpunk(),
                speed: 0.6,
            })
            .effect(TextEffect::PulsingGlow {
                color: theme::accent::ACCENT_7.into(),
                speed: 2.2,
            })
            .bold()
            .time(self.time);
        title.render(header_cols[0], frame);

        let hint = truncate_to_width(
            "↑/↓ jump · Enter/Tab next · Esc close",
            header_cols[1].width,
        );
        let hint_fx = StyledText::new(hint)
            .effect(TextEffect::ColorWave {
                color1: theme::accent::PRIMARY.into(),
                color2: theme::accent::ACCENT_8.into(),
                speed: 1.0,
                wavelength: 10.0,
            })
            .time(self.time);
        hint_fx.render(header_cols[1], frame);

        // Input row: label + input + match count
        let input_cols = Flex::horizontal()
            .constraints([
                Constraint::Fixed(10),
                Constraint::Min(10),
                Constraint::Fixed(22),
            ])
            .split(rows[1]);

        let label = StyledText::new("Query")
            .effect(TextEffect::Pulse {
                speed: 1.4,
                min_alpha: 0.35,
            })
            .bold()
            .time(self.time);
        label.render(input_cols[0], frame);
        self.search_input.render(input_cols[1], frame);

        let match_info = if self.search_matches.is_empty() {
            if self.search_input.value().len() >= 2 {
                "No matches".to_owned()
            } else {
                "Type to search".to_owned()
            }
        } else {
            format!(
                "{}/{} matches",
                self.current_match + 1,
                self.search_matches.len()
            )
        };
        let match_info = truncate_to_width(&match_info, input_cols[2].width);
        let match_fx = StyledText::new(match_info)
            .effect(TextEffect::AnimatedGradient {
                gradient: ColorGradient::sunset(),
                speed: 0.5,
            })
            .effect(TextEffect::Glow {
                color: theme::accent::WARNING.into(),
                intensity: 0.45,
            })
            .time(self.time);
        match_fx.render(input_cols[2], frame);

        // Status row: mode + current line info
        let status_cols = Flex::horizontal()
            .constraints([Constraint::Min(10), Constraint::Fixed(24)])
            .split(rows[2]);
        let status = if self.search_input.value().len() >= 2 {
            "Instant highlight active"
        } else {
            "Search updates as you type"
        };
        Paragraph::new(status)
            .style(theme::muted())
            .render(status_cols[0], frame);

        let jump_hint = StyledText::new("n/N outside search")
            .effect(TextEffect::Reveal {
                mode: RevealMode::CenterOut,
                progress: ((self.time * 0.6).sin() * 0.5 + 0.5).clamp(0.0, 1.0),
                seed: 21,
            })
            .time(self.time);
        jump_hint.render(status_cols[1], frame);
    }

    fn render_text_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Heavy)
            .title("Complete Works of William Shakespeare")
            .title_alignment(Alignment::Center)
            .style(theme::content_border());
        let inner = block.inner(area);
        block.render(area, frame);

        if inner.height == 0 || inner.width < 5 {
            return;
        }

        // Reserve 1 col for scrollbar on the right
        let text_width = inner.width.saturating_sub(1);
        let text_area = Rect::new(inner.x, inner.y, text_width, inner.height);
        let scrollbar_area = Rect::new(inner.x + text_width, inner.y, 1, inner.height);

        // Update viewport height (conceptual - we'd need &mut self, but we can use the value
        // from the area directly)
        let vh = inner.height as usize;

        // Render visible lines
        let query = self.search_input.value();
        let has_query = query.len() >= 2;

        for row in 0..vh {
            let line_idx = self.scroll_offset + row;
            if line_idx >= self.lines.len() {
                break;
            }

            let line = self.lines[line_idx];
            let line_area = Rect::new(
                text_area.x,
                text_area.y.saturating_add(row as u16),
                text_area.width,
                1,
            );

            let matches = if has_query {
                search_ascii_case_insensitive(line, query)
            } else {
                Vec::new()
            };

            // Determine style: highlight if it's a search match
            let is_current_match = has_query
                && !self.search_matches.is_empty()
                && self.search_matches.get(self.current_match) == Some(&line_idx);
            let is_any_match = !matches.is_empty();

            // Line number prefix with animated marker
            let num_width = 8u16.min(text_area.width);
            let content_width = text_area.width.saturating_sub(num_width);
            let marker_area = Rect::new(line_area.x, line_area.y, 1.min(num_width), 1);
            let num_area = Rect::new(line_area.x + 1, line_area.y, num_width.saturating_sub(1), 1);
            let content_area = Rect::new(line_area.x + num_width, line_area.y, content_width, 1);

            if marker_area.width > 0 {
                if is_current_match {
                    let marker = StyledText::new("▶")
                        .effect(TextEffect::PulsingGlow {
                            color: theme::accent::WARNING.into(),
                            speed: 2.0,
                        })
                        .time(self.time);
                    marker.render(marker_area, frame);
                } else if is_any_match {
                    Paragraph::new("•")
                        .style(Style::new().fg(theme::accent::INFO))
                        .render(marker_area, frame);
                } else {
                    Paragraph::new(" ")
                        .style(Style::new().fg(theme::fg::MUTED))
                        .render(marker_area, frame);
                }
            }

            let line_num = format!("{:>6} ", line_idx + 1);
            let num_style = if is_current_match {
                Style::new()
                    .fg(theme::accent::WARNING)
                    .attrs(StyleFlags::BOLD)
            } else if is_any_match {
                Style::new().fg(theme::accent::INFO)
            } else {
                Style::new().fg(theme::fg::MUTED)
            };
            Paragraph::new(line_num)
                .style(num_style)
                .render(num_area, frame);

            if content_area.width == 0 {
                continue;
            }

            if !is_any_match || query.is_empty() {
                Paragraph::new(line)
                    .style(Style::new().fg(theme::fg::SECONDARY))
                    .render(content_area, frame);
                continue;
            }

            let mut cursor_x = content_area.x;
            let line_y = content_area.y;
            let max_x = content_area.right();
            let mut last = 0usize;

            for result in &matches {
                let start = result.range.start;
                let end = result.range.end.min(line.len());
                if cursor_x >= max_x {
                    break;
                }
                if start < last || start >= line.len() {
                    continue;
                }
                if !line.is_char_boundary(start) || !line.is_char_boundary(end) {
                    continue;
                }

                let before = &line[last..start];
                if !before.is_empty() && cursor_x < max_x {
                    let remaining = max_x.saturating_sub(cursor_x);
                    let clipped = truncate_to_width(before, remaining);
                    let width = UnicodeWidthStr::width(clipped.as_str()) as u16;
                    if width > 0 {
                        let area = Rect::new(cursor_x, line_y, width.min(remaining), 1);
                        Paragraph::new(clipped)
                            .style(Style::new().fg(theme::fg::SECONDARY))
                            .render(area, frame);
                        cursor_x = cursor_x.saturating_add(width);
                    }
                }

                if cursor_x >= max_x {
                    break;
                }

                let matched = &line[start..end];
                let remaining = max_x.saturating_sub(cursor_x);
                let clipped = truncate_to_width(matched, remaining);
                let width = UnicodeWidthStr::width(clipped.as_str()) as u16;
                if width == 0 {
                    break;
                }

                if is_current_match {
                    let glow = StyledText::new(clipped)
                        .base_color(theme::accent::WARNING.into())
                        .bg_color(theme::alpha::HIGHLIGHT.into())
                        .bold()
                        .effect(TextEffect::AnimatedGradient {
                            gradient: ColorGradient::sunset(),
                            speed: 0.7,
                        })
                        .effect(TextEffect::PulsingGlow {
                            color: PackedRgba::rgb(255, 200, 120),
                            speed: 2.2,
                        })
                        .effect(TextEffect::ChromaticAberration {
                            offset: 1,
                            direction: Direction::Right,
                            animated: true,
                            speed: 0.5,
                        })
                        .time(self.time)
                        .seed(self.tick_count);
                    glow.render(Rect::new(cursor_x, line_y, width.min(remaining), 1), frame);
                } else {
                    Paragraph::new(clipped)
                        .style(
                            Style::new()
                                .fg(theme::fg::PRIMARY)
                                .bg(theme::alpha::HIGHLIGHT)
                                .attrs(StyleFlags::UNDERLINE),
                        )
                        .render(Rect::new(cursor_x, line_y, width.min(remaining), 1), frame);
                }
                cursor_x = cursor_x.saturating_add(width);
                last = end;
            }

            if cursor_x < max_x && last < line.len() {
                let tail = &line[last..];
                let remaining = max_x.saturating_sub(cursor_x);
                let clipped = truncate_to_width(tail, remaining);
                let width = UnicodeWidthStr::width(clipped.as_str()) as u16;
                if width > 0 {
                    Paragraph::new(clipped)
                        .style(Style::new().fg(theme::fg::SECONDARY))
                        .render(Rect::new(cursor_x, line_y, width.min(remaining), 1), frame);
                }
            }
        }

        // Scrollbar
        let mut scrollbar_state = ScrollbarState::new(self.total_lines(), self.scroll_offset, vh);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::new().fg(theme::accent::PRIMARY))
            .track_style(Style::new().fg(theme::bg::SURFACE));
        StatefulWidget::render(&scrollbar, scrollbar_area, frame, &mut scrollbar_state);
    }

    fn render_toc_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Contents")
            .title_alignment(Alignment::Center)
            .style(theme::content_border());
        let inner = block.inner(area);
        block.render(area, frame);

        if inner.height == 0 || inner.width < 5 {
            return;
        }

        self.toc_tree.render(inner, frame);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let section = self.current_section();
        let total = self.total_lines();
        let pos = self.scroll_offset + 1;
        let pct = (self.scroll_offset * 100).checked_div(total).unwrap_or(0);

        let status = format!(
            " Line {pos}/{total} ({pct}%) | {}",
            if section.len() > 40 {
                &section[..40]
            } else {
                section
            }
        );

        Paragraph::new(status)
            .style(Style::new().fg(theme::fg::MUTED).bg(theme::alpha::SURFACE))
            .render(area, frame);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shakespeare_initial_state() {
        let s = Shakespeare::new();
        assert_eq!(s.scroll_offset, 0);
        assert!(s.total_lines() > 100_000, "Should have 100K+ lines");
        assert!(!s.toc_entries.is_empty(), "Should have TOC entries");
    }

    #[test]
    fn shakespeare_search_to_be() {
        let mut s = Shakespeare::new();
        s.search_input.set_value("To be, or not to be");
        s.perform_search();
        assert!(
            !s.search_matches.is_empty(),
            "Should find 'To be or not to be' in Shakespeare"
        );
    }

    #[test]
    fn shakespeare_scroll_to_end() {
        let mut s = Shakespeare::new();
        s.viewport_height = 40;
        s.scroll_to(s.total_lines());
        let max_offset = s.total_lines().saturating_sub(s.viewport_height as usize);
        assert_eq!(s.scroll_offset, max_offset);
    }

    #[test]
    fn shakespeare_scroll_navigation() {
        let mut s = Shakespeare::new();
        s.viewport_height = 40;
        s.scroll_by(10);
        assert_eq!(s.scroll_offset, 10);
        s.scroll_by(-5);
        assert_eq!(s.scroll_offset, 5);
        // Can't scroll below 0
        s.scroll_by(-100);
        assert_eq!(s.scroll_offset, 0);
    }

    #[test]
    fn shakespeare_toc_has_plays() {
        let s = Shakespeare::new();
        let titles: Vec<&str> = s.toc_entries.iter().map(|e| e.title.as_str()).collect();
        // Should find major plays
        assert!(
            titles.iter().any(|t| t.contains("HAMLET")),
            "TOC should contain Hamlet, found: {:?}",
            &titles[..titles.len().min(20)]
        );
    }
}
