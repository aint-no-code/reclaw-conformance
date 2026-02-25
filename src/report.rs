use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConformanceOutcome {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConformanceReport {
    pub total: usize,
    pub failed: usize,
    pub outcomes: Vec<ConformanceOutcome>,
}

impl ConformanceReport {
    pub fn new(outcomes: Vec<ConformanceOutcome>) -> Self {
        let total = outcomes.len();
        let failed = outcomes.iter().filter(|outcome| !outcome.passed).count();

        Self {
            total,
            failed,
            outcomes,
        }
    }

    pub fn is_passing(&self) -> bool {
        self.failed == 0
    }
}
