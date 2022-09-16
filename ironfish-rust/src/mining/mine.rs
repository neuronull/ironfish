/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use byteorder::{BigEndian, WriteBytesExt};

/// returns true if a <= b when treating both as 32 byte big endian numbers.
pub(crate) fn bytes_lte(a: &[u8], b: &[u8]) -> bool {
    for i in 0..32 {
        if a[i] < b[i] {
            return true;
        }
        if a[i] > b[i] {
            return false;
        }
    }

    true
}

fn randomize_header(i: u64, mut header_bytes: &mut [u8]) {
    header_bytes.write_u64::<BigEndian>(i).unwrap();
}

pub(crate) fn mine_batch(
    header_bytes: &mut [u8],
    target: &[u8],
    start: u64,
    step_size: usize,
    batch_size: u64,
) -> Option<u64> {
    let end = start + batch_size - step_size as u64;
    for i in (start..=end).step_by(step_size) {
        randomize_header(i, header_bytes);
        let hash = blake3::hash(header_bytes);

        if bytes_lte(hash.as_bytes(), target) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use byteorder::{BigEndian, ReadBytesExt};

    use super::{bytes_lte, mine_batch};

    #[test]
    fn test_mine_batch_no_match() {
        let header_bytes = &mut [0, 1, 2, 4, 5, 6, 7, 8];
        let target = &[0u8; 32];
        let batch_size = 1;
        let start = 42;
        let step_size = 1;

        let result = mine_batch(header_bytes, target, start, step_size, batch_size);

        assert!(result.is_none())
    }

    #[test]
    fn test_mine_batch_match() {
        let header_bytes = &mut [0, 1, 2, 4, 5, 6, 7, 8];
        let batch_size = 3;
        let start = 42;
        let step_size = 1;

        // Hardcoded target value derived from a randomness of 43, which is lower than 42
        // This allows us to test the looping and target comparison a little better
        let target: &[u8; 32] = &[
            74, 52, 167, 52, 16, 135, 245, 240, 229, 92, 212, 133, 140, 231, 169, 56, 16, 105, 46,
            67, 145, 116, 198, 241, 183, 88, 140, 172, 79, 139, 210, 162,
        ];

        let result = mine_batch(header_bytes, target, start, step_size, batch_size);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), 43);
    }

    #[test]
    fn test_mine_batch_step_size() {
        let header_bytes_base = &mut (0..128).collect::<Vec<u8>>();
        let target = &[0u8; 32];
        let mut start = 0;
        let batch_size = 12;
        let step_size = 2;

        // Uses i values: 0, 2, 4, 6, 8, 10
        let header_bytes = &mut header_bytes_base.clone();
        let _ = mine_batch(header_bytes, target, start, step_size, batch_size);

        let mut cursor = Cursor::new(header_bytes);
        let end = cursor.read_u64::<BigEndian>().unwrap();
        assert_eq!(end, 10);

        // Uses i values: 1, 3, 5, 7, 9, 11
        let header_bytes = &mut header_bytes_base.clone();
        let _ = mine_batch(header_bytes, target, start + 1, step_size, batch_size);

        let mut cursor = Cursor::new(header_bytes);
        let end = cursor.read_u64::<BigEndian>().unwrap();
        assert_eq!(end, 11);

        // Second batch
        start += batch_size;

        // Uses i values: 12, 14, 16, 18, 20, 22
        let header_bytes = &mut header_bytes_base.clone();
        let _ = mine_batch(header_bytes, target, start, step_size, batch_size);

        let mut cursor = Cursor::new(header_bytes);
        let end = cursor.read_u64::<BigEndian>().unwrap();
        assert_eq!(end, 22);

        // Uses i values: 13, 15, 17, 19, 21, 23
        let header_bytes = &mut header_bytes_base.clone();
        let _ = mine_batch(header_bytes, target, start + 1, step_size, batch_size);

        let mut cursor = Cursor::new(header_bytes);
        let end = cursor.read_u64::<BigEndian>().unwrap();
        assert_eq!(end, 23);
    }

    #[test]
    fn test_mine_bytes_lte() {
        let big: &[u8; 32] = &[
            255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let small: &[u8; 32] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];

        assert!(bytes_lte(small, big));
        assert!(bytes_lte(small, small));
        assert!(!bytes_lte(big, small));
    }
}
