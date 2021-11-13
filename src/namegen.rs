pub struct NameGen {
    prefix: String,
    current_gen: usize,
}

impl NameGen {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            current_gen: 0,
        }
    }
}

impl Iterator for NameGen {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_gen += 1;

        Some(format!("{}_{}", self.prefix, self.current_gen))
    }
}
