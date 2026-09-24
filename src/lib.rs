pub struct Elephant;

impl Elephant {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Elephant {
    fn default() -> Self {
        Self::new()
    }
}