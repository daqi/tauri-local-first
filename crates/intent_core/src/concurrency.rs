pub fn compute_concurrency(logical: usize) -> usize {
    if logical == 0 { return 1; }
    let half = (logical + 1) / 2; // ceil(logical/2)
    std::cmp::min(half, 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_examples() {
        assert_eq!(compute_concurrency(2), 1);
        assert_eq!(compute_concurrency(4), 2);
        assert_eq!(compute_concurrency(6), 3);
        assert_eq!(compute_concurrency(8), 4);
        assert_eq!(compute_concurrency(16), 4);
    }
}
