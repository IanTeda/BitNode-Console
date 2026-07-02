pub struct Direction {
    pub unspecified: bool,
    pub forward: bool,
    pub backward: bool,
}

impl Default for Direction {
    fn default() -> Self {
        Self {
            unspecified: true,
            forward: false,
            backward: false,
        }
    }
}
