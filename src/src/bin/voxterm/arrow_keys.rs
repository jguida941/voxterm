#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrowKey {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) fn parse_arrow_keys(bytes: &[u8]) -> Vec<ArrowKey> {
    let mut keys = Vec::new();
    let mut idx: usize = 0;
    while idx.checked_add(2).is_some_and(|next| next < bytes.len()) {
        if bytes[idx] == 0x1b && (bytes[idx + 1] == b'[' || bytes[idx + 1] == b'O') {
            match bytes[idx + 2] {
                b'A' => keys.push(ArrowKey::Up),
                b'B' => keys.push(ArrowKey::Down),
                b'C' => keys.push(ArrowKey::Right),
                b'D' => keys.push(ArrowKey::Left),
                _ => {}
            }
            idx = idx.saturating_add(3);
        } else {
            idx = idx.saturating_add(1);
        }
    }
    keys
}

pub(crate) fn parse_arrow_keys_only(bytes: &[u8]) -> Option<Vec<ArrowKey>> {
    if bytes.is_empty() {
        return None;
    }
    let mut keys = Vec::new();
    let mut idx: usize = 0;
    while idx.checked_add(2).is_some_and(|next| next < bytes.len()) {
        if bytes[idx] == 0x1b && (bytes[idx + 1] == b'[' || bytes[idx + 1] == b'O') {
            let key = match bytes[idx + 2] {
                b'A' => ArrowKey::Up,
                b'B' => ArrowKey::Down,
                b'C' => ArrowKey::Right,
                b'D' => ArrowKey::Left,
                _ => return None,
            };
            keys.push(key);
            idx = idx.saturating_add(3);
        } else {
            return None;
        }
    }
    if idx == bytes.len() {
        Some(keys)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_arrow_keys_reads_sequences() {
        let bytes = [
            0x1b, b'[', b'A', 0x1b, b'[', b'B', b'x', 0x1b, b'O', b'C', 0x1b, b'[', b'D',
        ];
        let keys = parse_arrow_keys(&bytes);
        assert_eq!(
            keys,
            vec![
                ArrowKey::Up,
                ArrowKey::Down,
                ArrowKey::Right,
                ArrowKey::Left
            ]
        );
        assert!(parse_arrow_keys(&[0x1b, b'[']).is_empty());
    }
}
