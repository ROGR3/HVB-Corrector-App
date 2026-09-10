pub const MIN_EVENTS: u64 = 10;

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct Cell {
    pub target_exposed: u64,
    pub reference_exposed: u64,
    pub target_unexposed: u64,
    pub reference_unexposed: u64,
    pub population_exposed: Option<u64>,
    pub population_unexposed: Option<u64>,
}

impl Cell {
    pub fn apparent_ve(&self) -> Option<f64> {
        let pop_exposed = self.population_exposed?;
        let pop_unexposed = self.population_unexposed?;
        if pop_exposed == 0 || pop_unexposed == 0 || self.target_unexposed == 0 {
            return None;
        }
        let rate_exposed = self.target_exposed as f64 / pop_exposed as f64;
        let rate_unexposed = self.target_unexposed as f64 / pop_unexposed as f64;
        Some(1.0 - rate_exposed / rate_unexposed)
    }

    pub fn corrected_ve(&self) -> Option<f64> {
        if self.reference_exposed == 0 || self.target_unexposed == 0 {
            return None;
        }
        let odds_ratio = (self.target_exposed as f64 * self.reference_unexposed as f64)
            / (self.reference_exposed as f64 * self.target_unexposed as f64);
        Some(1.0 - odds_ratio)
    }

    pub fn is_reliable(&self) -> bool {
        [
            self.target_exposed,
            self.reference_exposed,
            self.target_unexposed,
            self.reference_unexposed,
        ]
        .into_iter()
        .min()
        .unwrap_or(0)
            >= MIN_EVENTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrected_ve_matches_odds_ratio_formula() {
        let cell = Cell {
            target_exposed: 580,
            reference_exposed: 2320,
            target_unexposed: 1040,
            reference_unexposed: 2080,
            population_exposed: None,
            population_unexposed: None,
        };
        let ve = cell.corrected_ve().unwrap();
        assert!((ve - 0.5).abs() < 1e-9, "got {ve}");
    }

    #[test]
    fn apparent_ve_is_none_without_population() {
        let cell = Cell {
            target_exposed: 10,
            reference_exposed: 10,
            target_unexposed: 10,
            reference_unexposed: 10,
            ..Default::default()
        };
        assert!(cell.apparent_ve().is_none());
        assert!(cell.corrected_ve().is_some());
    }

    #[test]
    fn reliability_uses_the_minimum_cell() {
        let cell = Cell {
            target_exposed: 100,
            reference_exposed: 100,
            target_unexposed: 100,
            reference_unexposed: 3,
            ..Default::default()
        };
        assert!(!cell.is_reliable());
    }

    #[test]
    fn corrected_ve_matches_the_data_md_verification_numbers() {
        let cell = Cell {
            target_exposed: 3792,
            reference_exposed: 109145,
            target_unexposed: 14965,
            reference_unexposed: 113807,
            population_exposed: None,
            population_unexposed: None,
        };
        let ve = cell.corrected_ve().unwrap();
        assert!((ve - 0.7360).abs() < 1e-3, "got {ve}");
    }
}
