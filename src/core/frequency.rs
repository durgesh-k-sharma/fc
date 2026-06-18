use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct FrequencyTable {
    counts: [u64; 256],
}

impl FrequencyTable {
    pub fn from_bytes(input: &[u8]) -> Self {
        let mut counts = [0u64; 256];
        for &byte in input {
            counts[byte as usize] += 1;
        }
        Self { counts }
    }

    pub fn from_bytes_par(input: &[u8]) -> Self {
        const PARALLEL_THRESHOLD: usize = 64 * 1024; // 64KB

        if input.len() < PARALLEL_THRESHOLD {
            return Self::from_bytes(input);
        }

        let counts = input
            .par_chunks(4096)
            .map(|chunk| {
                let mut local = [0u64; 256];
                for &byte in chunk {
                    local[byte as usize] += 1;
                }
                local
            })
            .reduce(
                || [0u64; 256],
                |mut a, b| {
                    for i in 0..256 {
                        a[i] += b[i];
                    }
                    a
                },
            );

        Self { counts }
    }

    pub fn merge(&mut self, other: &Self) {
        for i in 0..256 {
            self.counts[i] += other.counts[i];
        }
    }

    pub fn as_array(&self) -> &[u64; 256] {
        &self.counts
    }

    pub fn total(&self) -> u64 {
        self.counts.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_simple_string() {
        let table = FrequencyTable::from_bytes(b"aabbc");
        assert_eq!(table.as_array()[b'a' as usize], 2);
        assert_eq!(table.as_array()[b'b' as usize], 2);
        assert_eq!(table.as_array()[b'c' as usize], 1);
        assert_eq!(table.as_array()[0], 0);
    }

    #[test]
    fn count_empty() {
        let table = FrequencyTable::from_bytes(b"");
        for i in 0..256 {
            assert_eq!(table.as_array()[i], 0);
        }
    }

    #[test]
    fn parallel_matches_sequential() {
        let input = vec![0xABu8; 100_000];
        let seq = FrequencyTable::from_bytes(&input);
        let par = FrequencyTable::from_bytes_par(&input);
        assert_eq!(seq.as_array(), par.as_array());
    }

    #[test]
    fn merge_two_tables() {
        let a = FrequencyTable::from_bytes(b"aab");
        let b = FrequencyTable::from_bytes(b"bbc");
        let mut merged = a;
        merged.merge(&b);
        assert_eq!(merged.as_array()[b'a' as usize], 2);
        assert_eq!(merged.as_array()[b'b' as usize], 3);
        assert_eq!(merged.as_array()[b'c' as usize], 1);
    }

    #[test]
    fn parallel_large_input() {
        let input: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
        let seq = FrequencyTable::from_bytes(&input);
        let par = FrequencyTable::from_bytes_par(&input);
        assert_eq!(seq.as_array(), par.as_array());
    }
}
