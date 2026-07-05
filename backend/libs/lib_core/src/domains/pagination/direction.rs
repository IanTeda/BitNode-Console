/// Direction controls which side of the pagination cursor to read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Read items after the cursor (chronologically newer for logs).
    #[default]
    Forward,
    /// Read items before the cursor (chronologically older for logs).
    Backward,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_defaults_to_forward() {
        assert_eq!(Direction::default(), Direction::Forward);
    }
}
