#![forbid(unsafe_code)]

//! VOI overlay demo screen (Galaxy-Brain widget).

use std::time::Instant;

use ftui_core::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ftui_core::geometry::Rect;
use ftui_render::frame::Frame;
use ftui_runtime::{
    Cmd, InlineAutoRemeasureConfig, VoiLogEntry, VoiSampler, VoiSamplerSnapshot,
    inline_auto_voi_snapshot,
};
use ftui_style::Style;
use ftui_widgets::Widget;
use ftui_widgets::borders::BorderType;
use ftui_widgets::paragraph::Paragraph;
use ftui_widgets::voi_debug_overlay::{
    VoiDebugOverlay, VoiDecisionSummary, VoiLedgerEntry, VoiObservationSummary, VoiOverlayData,
    VoiOverlayStyle, VoiPosteriorSummary,
};

use super::{HelpEntry, Screen};
use crate::theme;

/// Tiny screen showcasing the VOI overlay widget.
pub struct VoiOverlayScreen {
    sampler: VoiSampler,
    tick: u64,
    start: Instant,
}

impl Default for VoiOverlayScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiOverlayScreen {
    pub fn new() -> Self {
        let mut config = InlineAutoRemeasureConfig::default().voi;
        config.enable_logging = true;
        config.max_log_entries = 96;
        let sampler = VoiSampler::new(config);
        Self {
            sampler,
            tick: 0,
            start: Instant::now(),
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn overlay_area(area: Rect, width: u16, height: u16) -> Rect {
        let w = width.min(area.width).max(1);
        let h = height.min(area.height).max(1);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        Rect::new(x, y, w, h)
    }

    fn data_from_snapshot(&self, snapshot: &VoiSamplerSnapshot, source: &str) -> VoiOverlayData {
        VoiOverlayData {
            title: "VOI Overlay".to_string(),
            tick: Some(self.tick),
            source: Some(source.to_string()),
            posterior: VoiPosteriorSummary {
                alpha: snapshot.alpha,
                beta: snapshot.beta,
                mean: snapshot.posterior_mean,
                variance: snapshot.posterior_variance,
                expected_variance_after: snapshot.expected_variance_after,
                voi_gain: snapshot.voi_gain,
            },
            decision: snapshot
                .last_decision
                .as_ref()
                .map(|decision| VoiDecisionSummary {
                    event_idx: decision.event_idx,
                    should_sample: decision.should_sample,
                    reason: decision.reason.to_string(),
                    score: decision.score,
                    cost: decision.cost,
                    log_bayes_factor: decision.log_bayes_factor,
                    e_value: decision.e_value,
                    e_threshold: decision.e_threshold,
                    boundary_score: decision.boundary_score,
                }),
            observation: snapshot
                .last_observation
                .as_ref()
                .map(|obs| VoiObservationSummary {
                    sample_idx: obs.sample_idx,
                    violated: obs.violated,
                    posterior_mean: obs.posterior_mean,
                    alpha: obs.alpha,
                    beta: obs.beta,
                }),
            ledger: Self::ledger_entries_from_logs(snapshot.recent_logs.iter().rev().take(6).rev()),
        }
    }

    fn data_from_sampler(&self, source: &str) -> VoiOverlayData {
        let (alpha, beta) = self.sampler.posterior_params();
        let variance = self.sampler.posterior_variance();
        let expected_after = self.sampler.expected_variance_after();
        VoiOverlayData {
            title: "VOI Overlay".to_string(),
            tick: Some(self.tick),
            source: Some(source.to_string()),
            posterior: VoiPosteriorSummary {
                alpha,
                beta,
                mean: self.sampler.posterior_mean(),
                variance,
                expected_variance_after: expected_after,
                voi_gain: (variance - expected_after).max(0.0),
            },
            decision: self
                .sampler
                .last_decision()
                .map(|decision| VoiDecisionSummary {
                    event_idx: decision.event_idx,
                    should_sample: decision.should_sample,
                    reason: decision.reason.to_string(),
                    score: decision.score,
                    cost: decision.cost,
                    log_bayes_factor: decision.log_bayes_factor,
                    e_value: decision.e_value,
                    e_threshold: decision.e_threshold,
                    boundary_score: decision.boundary_score,
                }),
            observation: self
                .sampler
                .last_observation()
                .map(|obs| VoiObservationSummary {
                    sample_idx: obs.sample_idx,
                    violated: obs.violated,
                    posterior_mean: obs.posterior_mean,
                    alpha: obs.alpha,
                    beta: obs.beta,
                }),
            ledger: Self::ledger_entries_from_logs(self.sampler.logs().iter().rev().take(6).rev()),
        }
    }

    fn ledger_entries_from_logs<'a, I>(logs: I) -> Vec<VoiLedgerEntry>
    where
        I: IntoIterator<Item = &'a VoiLogEntry>,
    {
        logs.into_iter()
            .map(|entry| match entry {
                VoiLogEntry::Decision(decision) => VoiLedgerEntry::Decision {
                    event_idx: decision.event_idx,
                    should_sample: decision.should_sample,
                    voi_gain: decision.voi_gain,
                    log_bayes_factor: decision.log_bayes_factor,
                },
                VoiLogEntry::Observation(obs) => VoiLedgerEntry::Observation {
                    sample_idx: obs.sample_idx,
                    violated: obs.violated,
                    posterior_mean: obs.posterior_mean,
                },
            })
            .collect()
    }
}

impl Screen for VoiOverlayScreen {
    type Message = ();

    fn update(&mut self, event: &Event) -> Cmd<Self::Message> {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            self.reset();
        }
        Cmd::None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        if area.is_empty() {
            return;
        }

        let hint = "runtime snapshot: inline-auto  |  fallback: local sampler  |  r:reset";
        Paragraph::new(hint)
            .style(Style::new().fg(theme::fg::MUTED))
            .render(
                Rect::new(area.x + 1, area.y, area.width.saturating_sub(2), 1),
                frame,
            );

        let overlay_area = Self::overlay_area(area, 68, 22);
        if overlay_area.width < 28 || overlay_area.height < 8 {
            return;
        }

        let data = if let Some(snapshot) = inline_auto_voi_snapshot() {
            self.data_from_snapshot(&snapshot, "runtime:inline-auto")
        } else {
            self.data_from_sampler("demo:fallback")
        };

        let style = VoiOverlayStyle {
            border: Style::new().fg(theme::accent::PRIMARY).bg(theme::bg::DEEP),
            text: Style::new().fg(theme::fg::PRIMARY),
            background: Some(theme::bg::DEEP.into()),
            border_type: BorderType::Rounded,
        };

        VoiDebugOverlay::new(data)
            .with_style(style)
            .render(overlay_area, frame);
    }

    fn keybindings(&self) -> Vec<HelpEntry> {
        vec![HelpEntry {
            key: "r",
            action: "Reset VOI sampler",
        }]
    }

    fn tick(&mut self, tick_count: u64) {
        self.tick = tick_count;
        let now = Instant::now();
        let decision = self.sampler.decide(now);
        if decision.should_sample {
            let violated = (tick_count % 17) < 3;
            self.sampler.observe_at(violated, now);
        }
    }

    fn title(&self) -> &'static str {
        "VOI Overlay"
    }

    fn tab_label(&self) -> &'static str {
        "VOI"
    }
}
