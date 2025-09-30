use std::time::{SystemTime, UNIX_EPOCH};

/// Return current time in milliseconds since Unix epoch.
#[inline]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::now_ms;
    #[test]
    fn monotonic_nonzero() {
        let a = now_ms();
        let b = now_ms();
        assert!(b >= a);
        assert!(a > 0);
    }
}
