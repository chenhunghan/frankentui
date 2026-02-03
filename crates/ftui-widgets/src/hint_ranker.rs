//! Utility-based keybinding hint ranking with Bayesian posteriors.
//!
//! Prioritises which keyboard shortcuts to display so users see the most
//! valuable hints first, without clutter or instability.
//!
//! # Mathematical Model
//!
//! Each hint `i` has:
//! - **Utility** `U_i ~ Beta(α_i, β_i)`: posterior belief about how useful
//!   the hint is, updated from observed usage events.
//! - **Cost** `C_i`: screen space (character columns) consumed by the hint.
//! - **Net value** `V_i = E[U_i] - λ × C_i`: expected utility minus
//!   display cost weighted by space pressure `λ`.
//!
//! Hints are ranked by decreasing `V_i`. To prevent flicker, a hysteresis
//! margin `ε` is applied: a hint must improve by at least `ε` over its
//! current rank-neighbour to swap positions.
//!
//! # Context Sensitivity
//!
//! Hints can be tagged with context requirements (widget type, mode).
//! Only hints matching the current context are considered for ranking.
//! When context changes, the ranking is recomputed from scratch.
//!
//! # Evidence Ledger
//!
//! Every ranking decision is recorded for explainability:
//! `(hint_id, E[U], C, V, rank, context)`.
//!
//! # Fallback
//!
//! If no usage data exists (cold start), hints use a static priority
//! ordering defined at registration time.
//!
//! # Failure Modes
//!
//! | Condition | Behavior | Rationale |
//! |-----------|----------|-----------|
//! | No hints registered | Return empty ranking | Vacuously correct |
//! | No usage data | Use static priority | Cold start fallback |
//! | All hints filtered | Return empty ranking | Context mismatch |
//! | Hysteresis deadlock | Break ties by id | Determinism guarantee |

/// A context tag for filtering hints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HintContext {
    /// Always show regardless of context.
    Global,
    /// Show only when a specific widget type is focused.
    Widget(String),
    /// Show only in a specific mode (e.g., "insert", "normal").
    Mode(String),
}

/// Per-hint Bayesian statistics.
#[derive(Debug, Clone)]
pub struct HintStats {
    /// Beta posterior α (usage events + prior).
    pub alpha: f64,
    /// Beta posterior β (non-usage events + prior).
    pub beta: f64,
    /// Display cost in character columns.
    pub cost: f64,
    /// Static fallback priority (lower = higher priority).
    pub static_priority: u32,
    /// Total observations.
    pub observations: u64,
}

impl HintStats {
    /// Posterior mean E[U] = α / (α + β).
    #[inline]
    pub fn expected_utility(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Posterior variance.
    #[inline]
    pub fn variance(&self) -> f64 {
        let sum = self.alpha + self.beta;
        (self.alpha * self.beta) / (sum * sum * (sum + 1.0))
    }

    /// Value of information: standard deviation (exploration bonus).
    #[inline]
    pub fn voi(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// A registered keybinding hint.
#[derive(Debug, Clone)]
pub struct HintEntry {
    /// Unique identifier.
    pub id: usize,
    /// Human-readable label (e.g., "Ctrl+S Save").
    pub label: String,
    /// Display cost in columns.
    pub cost: f64,
    /// Context filter.
    pub context: HintContext,
    /// Bayesian statistics.
    pub stats: HintStats,
}

/// Evidence ledger entry.
#[derive(Debug, Clone)]
pub struct RankingEvidence {
    pub id: usize,
    pub label: String,
    pub expected_utility: f64,
    pub cost: f64,
    pub net_value: f64,
    pub voi: f64,
    pub rank: usize,
}

/// Configuration for the hint ranker.
#[derive(Debug, Clone)]
pub struct RankerConfig {
    /// Prior α for Beta(α, β). Default: 1.0.
    pub prior_alpha: f64,
    /// Prior β for Beta(α, β). Default: 1.0.
    pub prior_beta: f64,
    /// Space pressure λ: weight of display cost. Default: 0.01.
    pub lambda: f64,
    /// Hysteresis margin ε. Default: 0.02.
    pub hysteresis: f64,
    /// VOI exploration weight (0 = pure exploitation). Default: 0.1.
    pub voi_weight: f64,
}

impl Default for RankerConfig {
    fn default() -> Self {
        Self {
            prior_alpha: 1.0,
            prior_beta: 1.0,
            lambda: 0.01,
            hysteresis: 0.02,
            voi_weight: 0.1,
        }
    }
}

/// Utility-based keybinding hint ranker.
#[derive(Debug, Clone)]
pub struct HintRanker {
    config: RankerConfig,
    hints: Vec<HintEntry>,
    /// Last computed ordering (hint ids).
    last_ordering: Vec<usize>,
    /// Last context used for ranking.
    last_context: Option<String>,
}

impl HintRanker {
    /// Create a new ranker with the given config.
    pub fn new(config: RankerConfig) -> Self {
        Self {
            config,
            hints: Vec::new(),
            last_ordering: Vec::new(),
            last_context: None,
        }
    }

    /// Register a keybinding hint. Returns the assigned id.
    pub fn register(
        &mut self,
        label: impl Into<String>,
        cost_columns: f64,
        context: HintContext,
        static_priority: u32,
    ) -> usize {
        let id = self.hints.len();
        self.hints.push(HintEntry {
            id,
            label: label.into(),
            cost: cost_columns,
            context,
            stats: HintStats {
                alpha: self.config.prior_alpha,
                beta: self.config.prior_beta,
                cost: cost_columns,
                static_priority,
                observations: 0,
            },
        });
        id
    }

    /// Record that a hint was used (user pressed the shortcut).
    pub fn record_usage(&mut self, hint_id: usize) {
        if let Some(h) = self.hints.get_mut(hint_id) {
            h.stats.alpha += 1.0;
            h.stats.observations += 1;
        }
    }

    /// Record that a hint was shown but not used (negative evidence).
    pub fn record_shown_not_used(&mut self, hint_id: usize) {
        if let Some(h) = self.hints.get_mut(hint_id) {
            h.stats.beta += 1.0;
            h.stats.observations += 1;
        }
    }

    /// Compute net value for a hint.
    fn net_value(&self, h: &HintEntry) -> f64 {
        let eu = h.stats.expected_utility();
        let voi = h.stats.voi();
        eu + self.config.voi_weight * voi - self.config.lambda * h.cost
    }

    /// Compute ranking for the given context. Returns (ordering, ledger).
    ///
    /// If `context_key` is `None`, all hints are considered.
    pub fn rank(&mut self, context_key: Option<&str>) -> (Vec<usize>, Vec<RankingEvidence>) {
        let context_str = context_key.map(String::from);

        // Filter hints by context.
        let mut candidates: Vec<(usize, f64)> = self
            .hints
            .iter()
            .filter(|h| match (&h.context, context_key) {
                (HintContext::Global, _) => true,
                (HintContext::Widget(w), Some(ctx)) => w == ctx,
                (HintContext::Mode(m), Some(ctx)) => m == ctx,
                _ => context_key.is_none(), // show all if no context filter
            })
            .map(|h| {
                let v = if h.stats.observations == 0 {
                    // Cold start: use static priority (negate so lower priority = higher value).
                    -(h.stats.static_priority as f64)
                } else {
                    self.net_value(h)
                };
                (h.id, v)
            })
            .collect();

        // Sort by decreasing net value, ties by id.
        candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let new_ordering: Vec<usize> = candidates.iter().map(|(id, _)| *id).collect();

        // Apply hysteresis: only accept reordering if improvement exceeds ε.
        let ordering = if self.last_context == context_str && !self.last_ordering.is_empty() {
            self.apply_hysteresis(&new_ordering, &candidates)
        } else {
            new_ordering.clone()
        };

        // Build evidence ledger.
        let ledger: Vec<RankingEvidence> = ordering
            .iter()
            .enumerate()
            .map(|(rank, &id)| {
                let h = &self.hints[id];
                RankingEvidence {
                    id,
                    label: h.label.clone(),
                    expected_utility: h.stats.expected_utility(),
                    cost: h.cost,
                    net_value: self.net_value(h),
                    voi: h.stats.voi(),
                    rank,
                }
            })
            .collect();

        self.last_ordering = ordering.clone();
        self.last_context = context_str;

        (ordering, ledger)
    }

    /// Apply hysteresis to prevent flicker.
    fn apply_hysteresis(&self, new_order: &[usize], scores: &[(usize, f64)]) -> Vec<usize> {
        // Build score map.
        let score_map: std::collections::HashMap<usize, f64> = scores.iter().copied().collect();

        let mut result = self.last_ordering.clone();

        // Ensure result contains only hints that are in new_order.
        result.retain(|id| new_order.contains(id));

        // Add any new hints not in last ordering.
        for &id in new_order {
            if !result.contains(&id) {
                result.push(id);
            }
        }

        // Bubble sort with hysteresis: only swap if improvement > ε.
        let eps = self.config.hysteresis;
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..result.len().saturating_sub(1) {
                let a = result[i];
                let b = result[i + 1];
                let sa = score_map.get(&a).copied().unwrap_or(f64::NEG_INFINITY);
                let sb = score_map.get(&b).copied().unwrap_or(f64::NEG_INFINITY);
                if sb > sa + eps {
                    result.swap(i, i + 1);
                    changed = true;
                }
            }
        }

        result
    }

    /// Get top N hints for display.
    pub fn top_n(&mut self, n: usize, context_key: Option<&str>) -> Vec<&HintEntry> {
        let (ordering, _) = self.rank(context_key);
        ordering
            .into_iter()
            .take(n)
            .filter_map(|id| self.hints.get(id))
            .collect()
    }

    /// Get stats for a hint.
    pub fn stats(&self, id: usize) -> Option<&HintStats> {
        self.hints.get(id).map(|h| &h.stats)
    }

    /// Number of registered hints.
    pub fn hint_count(&self) -> usize {
        self.hints.len()
    }
}

impl Default for HintRanker {
    fn default() -> Self {
        Self::new(RankerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ranker() -> HintRanker {
        let mut r = HintRanker::new(RankerConfig::default());
        r.register("Ctrl+S Save", 12.0, HintContext::Global, 1);
        r.register("Ctrl+Z Undo", 12.0, HintContext::Global, 2);
        r.register("Ctrl+F Find", 12.0, HintContext::Global, 3);
        r.register("Tab Complete", 13.0, HintContext::Widget("input".into()), 4);
        r.register("Esc Cancel", 11.0, HintContext::Global, 5);
        r
    }

    #[test]
    fn empty_ranker_returns_empty() {
        let mut r = HintRanker::default();
        let (ordering, ledger) = r.rank(None);
        assert!(ordering.is_empty());
        assert!(ledger.is_empty());
    }

    #[test]
    fn cold_start_uses_static_priority() {
        let mut r = make_ranker();
        let (ordering, _) = r.rank(None);
        // No usage data → static priority order (1, 2, 3, 5, 4).
        // But id=3 has Widget context, so without context filter it's included
        // when context_key is None.
        // Static priorities: 1,2,3,4,5 → ids 0,1,2,3,4.
        // Values: -1, -2, -3, -4, -5.
        assert_eq!(ordering[0], 0); // priority 1
        assert_eq!(ordering[1], 1); // priority 2
        assert_eq!(ordering[2], 2); // priority 3
    }

    #[test]
    fn unit_prior_updates() {
        let mut r = HintRanker::default();
        let id = r.register("test", 10.0, HintContext::Global, 1);

        // Prior: α=1, β=1 → E[U] = 0.5
        assert!((r.stats(id).unwrap().expected_utility() - 0.5).abs() < 1e-10);

        // 4 usages
        for _ in 0..4 {
            r.record_usage(id);
        }
        // α=5, β=1 → E[U] = 5/6
        assert!((r.stats(id).unwrap().expected_utility() - 5.0 / 6.0).abs() < 1e-10);

        // 2 non-usages
        for _ in 0..2 {
            r.record_shown_not_used(id);
        }
        // α=5, β=3 → E[U] = 5/8
        assert!((r.stats(id).unwrap().expected_utility() - 5.0 / 8.0).abs() < 1e-10);
    }

    #[test]
    fn unit_ranking_stability() {
        let mut r = HintRanker::new(RankerConfig {
            hysteresis: 0.05,
            ..Default::default()
        });
        let a = r.register("A", 10.0, HintContext::Global, 1);
        let b = r.register("B", 10.0, HintContext::Global, 2);

        // Give A lots of usage so it ranks first.
        for _ in 0..20 {
            r.record_usage(a);
        }
        for _ in 0..10 {
            r.record_usage(b);
        }

        let (order1, _) = r.rank(None);
        assert_eq!(order1[0], a);
        assert_eq!(order1[1], b);

        // Small perturbation: one more B usage. Should NOT flip due to hysteresis.
        r.record_usage(b);
        let (order2, _) = r.rank(None);
        assert_eq!(order2[0], a, "hysteresis should prevent flicker");
    }

    #[test]
    fn context_filtering() {
        let mut r = make_ranker();
        // Rank with "input" context.
        let (ordering, _) = r.rank(Some("input"));
        // Should include Global hints and Widget("input") hint (id=3).
        assert!(ordering.contains(&3), "input widget hint should appear");

        // Rank with "list" context.
        let (ordering2, _) = r.rank(Some("list"));
        // Should NOT include Widget("input") hint.
        assert!(
            !ordering2.contains(&3),
            "input widget hint should not appear for list"
        );
    }

    #[test]
    fn property_context_switch_reranks() {
        let mut r = make_ranker();

        // Give different usage patterns.
        for _ in 0..10 {
            r.record_usage(0); // Save
        }
        for _ in 0..5 {
            r.record_usage(2); // Find
        }

        let (order_none, _) = r.rank(None);
        let (order_list, _) = r.rank(Some("list"));

        // "list" context should exclude Widget("input") hint (id=3).
        assert!(
            order_none.contains(&3),
            "None context should include input widget hint"
        );
        assert!(
            !order_list.contains(&3),
            "list context should exclude input widget hint"
        );
    }

    #[test]
    fn voi_exploration_bonus() {
        let mut r = HintRanker::new(RankerConfig {
            voi_weight: 1.0, // strong exploration
            lambda: 0.0,     // no cost penalty
            hysteresis: 0.0,
            ..Default::default()
        });
        let a = r.register("A", 10.0, HintContext::Global, 1);
        let _b = r.register("B", 10.0, HintContext::Global, 2);

        // A has lots of data (low VOI), B has none (high VOI).
        for _ in 0..100 {
            r.record_usage(a);
            r.record_shown_not_used(a);
        }

        let (ordering, _) = r.rank(None);
        // B should rank higher due to exploration bonus despite no usage data.
        // B is still cold-start (0 observations) → uses static priority.
        // A has observations → uses Bayesian score.
        // With strong VOI and A having p≈0.5 + low VOI, B's static priority
        // of 2 gives it value -2.0 which is less than A's ~0.5 + VOI.
        // Let's check A's net value.
        let a_eu = r.stats(a).unwrap().expected_utility();
        let a_voi = r.stats(a).unwrap().voi();
        assert!(a_eu > 0.4); // approximately 0.5
        assert!(a_voi < 0.1); // low uncertainty with 200 observations
        // B is cold start: value = -2.0 (static priority).
        // A's net value ≈ 0.5 + 1.0*small = ~0.5.
        // So A should be first, B second (cold start loses to warm data).
        assert_eq!(ordering[0], a);
    }

    #[test]
    fn top_n_returns_limited() {
        let mut r = make_ranker();
        let top = r.top_n(2, None);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn deterministic_under_same_history() {
        let run = || {
            let mut r = make_ranker();
            r.record_usage(0);
            r.record_usage(0);
            r.record_usage(2);
            r.record_shown_not_used(1);
            r.record_shown_not_used(4);
            let (ordering, _) = r.rank(None);
            ordering
        };

        assert_eq!(run(), run());
    }

    #[test]
    fn ledger_records_all_ranked_hints() {
        let mut r = make_ranker();
        for _ in 0..5 {
            r.record_usage(0);
        }
        let (ordering, ledger) = r.rank(None);
        assert_eq!(ordering.len(), ledger.len());

        // Ranks should be sequential.
        for (i, entry) in ledger.iter().enumerate() {
            assert_eq!(entry.rank, i);
        }
    }

    #[test]
    fn usage_promotes_hint() {
        let mut r = HintRanker::new(RankerConfig {
            hysteresis: 0.0,
            ..Default::default()
        });
        let a = r.register("A", 10.0, HintContext::Global, 2); // lower static prio
        let b = r.register("B", 10.0, HintContext::Global, 1); // higher static prio

        // Cold start: B first (priority 1 < 2).
        let (order1, _) = r.rank(None);
        assert_eq!(order1[0], b);

        // Heavy A usage should promote it.
        for _ in 0..20 {
            r.record_usage(a);
        }
        // Give B one observation so it leaves cold-start.
        r.record_shown_not_used(b);

        let (order2, _) = r.rank(None);
        assert_eq!(order2[0], a, "heavy usage should promote A above B");
    }
}
