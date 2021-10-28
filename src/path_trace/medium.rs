#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Medium {
    /// Index of refraction
    ior: f64,
}

impl Medium {
    pub fn new(ior: f64) -> Self {
        Self { ior }
    }

    pub fn air() -> Self {
        Self::new(1.0)
    }

    pub fn glass() -> Self {
        Self::new(1.5)
    }

    /// Get medium's ior.
    pub fn get_ior(&self) -> f64 {
        self.ior
    }
}
