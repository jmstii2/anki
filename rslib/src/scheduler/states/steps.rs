// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

const DEFAULT_SECS_IF_MISSING: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LearningSteps<'a> {
    /// The steps in minutes.
    steps: &'a [f32],
}

fn to_secs(v: f32) -> u32 {
    (v * 60.0) as u32
}

impl<'a> LearningSteps<'a> {
    /// Takes `steps` as minutes.
    pub(crate) fn new(steps: &[f32]) -> LearningSteps<'_> {
        LearningSteps { steps }
    }

    /// Strip off 'learning today', and ensure index is in bounds.
    fn get_index(self, remaining: u32) -> usize {
        let total = self.steps.len();
        total
            .saturating_sub((remaining % 1000) as usize)
            .min(total.saturating_sub(1))
    }

    fn secs_at_index(&self, index: usize) -> Option<u32> {
        self.steps.get(index).copied().map(to_secs)
    }

    /// Cards in learning must always have at least one learning step.
    pub(crate) fn again_delay_secs_learn(&self) -> u32 {
        self.secs_at_index(0).unwrap_or(DEFAULT_SECS_IF_MISSING)
    }

    pub(crate) fn again_delay_secs_relearn(&self) -> Option<u32> {
        self.secs_at_index(0)
    }

    pub(crate) fn hard_delay_secs(self, remaining: u32) -> Option<u32> {
        let idx = self.get_index(remaining);
        self.secs_at_index(idx)
            // if current is invalid, try first step
            .or_else(|| self.steps.first().copied().map(to_secs))
            .map(|current| {
                // special case to avoid Hard and Again showing same interval
                if idx == 0 {
                    // if there is no next step, simulate one with twice the interval of `current`
                    let next = self
                        .secs_at_index(idx + 1)
                        .unwrap_or_else(|| current.saturating_mul(2));
                    current.saturating_add(next) / 2
                } else {
                    current
                }
            })
    }

    pub(crate) fn good_delay_secs(self, remaining: u32) -> Option<u32> {
        let idx = self.get_index(remaining);
        self.secs_at_index(idx + 1)
    }

    pub(crate) fn current_delay_secs(self, remaining: u32) -> u32 {
        let idx = self.get_index(remaining);
        self.secs_at_index(idx).unwrap_or_default()
    }

    pub(crate) fn remaining_for_good(self, remaining: u32) -> u32 {
        let idx = self.get_index(remaining);
        self.steps.len().saturating_sub(idx + 1) as u32
    }

    pub(crate) fn remaining_for_failed(self) -> u32 {
        self.steps.len() as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_delay_secs {
        ($steps:expr, $remaining:expr, $again_delay:expr, $hard_delay:expr, $good_delay:expr) => {
            let steps = LearningSteps::new(&$steps);
            assert_eq!(steps.again_delay_secs_learn(), $again_delay);
            assert_eq!(steps.hard_delay_secs($remaining), $hard_delay);
            assert_eq!(steps.good_delay_secs($remaining), $good_delay);
        };
    }

    #[test]
    fn delay_secs() {
        assert_delay_secs!([10.0], 1, 600, Some(900), None);

        assert_delay_secs!([1.0, 10.0], 2, 60, Some(330), Some(600));
        assert_delay_secs!([1.0, 10.0], 1, 60, Some(600), None);

        assert_delay_secs!([1.0, 10.0, 100.0], 3, 60, Some(330), Some(600));
        assert_delay_secs!([1.0, 10.0, 100.0], 2, 60, Some(600), Some(6000));
        assert_delay_secs!([1.0, 10.0, 100.0], 1, 60, Some(6000), None);
    }
}
