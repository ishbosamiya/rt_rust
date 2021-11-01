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

#[derive(Debug, Clone)]
pub struct Mediums {
    mediums: Vec<Medium>,
}

impl Mediums {
    pub fn new() -> Self {
        Self {
            mediums: Vec::new(),
        }
    }

    pub fn with_air() -> Self {
        let mut res = Self::new();
        res.add_medium(Medium::air());
        res
    }

    pub fn add_medium(&mut self, medium: Medium) {
        self.mediums.push(medium);
    }

    pub fn remove_medium(&mut self) -> Option<Medium> {
        self.mediums.pop()
    }

    pub fn get_lastest_medium(&self) -> Option<&Medium> {
        self.mediums.first()
    }

    pub fn get_number_of_mediums(&self) -> usize {
        self.mediums.len()
    }
}

impl Default for Mediums {
    fn default() -> Self {
        Self::new()
    }
}
