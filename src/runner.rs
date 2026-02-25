use crate::{scenario::Scenario, ConformanceReport, ConformanceTransport};

pub struct ConformanceRunner<T>
where
    T: ConformanceTransport,
{
    transport: T,
}

impl<T> ConformanceRunner<T>
where
    T: ConformanceTransport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn run(&self) -> ConformanceReport {
        let outcomes = Scenario::all()
            .iter()
            .map(|scenario| scenario.run(&self.transport))
            .collect();

        ConformanceReport::new(outcomes)
    }
}
