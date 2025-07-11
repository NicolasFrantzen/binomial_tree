use statrs::distribution::ContinuousCDF;

pub(crate) struct Normal {
    normal: statrs::distribution::Normal,
}

impl Normal {
    pub(crate) fn new() -> Self {
        Self {
            normal: statrs::distribution::Normal::standard(),
        }
    }

    pub(crate) fn cdf(&self, x: f64) -> f64 {
        self.normal.cdf(x)
    }
}