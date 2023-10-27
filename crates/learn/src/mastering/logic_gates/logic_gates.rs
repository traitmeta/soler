pub fn and(a: u8, b: u8) -> u8 {
    match (a, b) {
        (1, 1) => 1,
        _ => 0,
    }
}

pub fn xor(a: u8, b: u8) -> u8 {
    match (a, b) {
        (1, 0) | (0, 1) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::logic_gates::{and, xor};

    #[test]
    fn test_and() {
        assert_eq!(1, and(1, 1));
        assert_eq!(0, and(0, 1));
        assert_eq!(0, and(1, 0));
        assert_eq!(0, and(0, 0));
    }

    #[test]
    fn test_xor() {
        assert_eq!(0, xor(1, 1));
        assert_eq!(1, xor(0, 1));
        assert_eq!(1, xor(1, 0));
        assert_eq!(0, xor(0, 0));
    }
}
