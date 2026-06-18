use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone)]
pub enum HuffmanNode {
    Leaf { symbol: u8 },
    Internal {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

#[derive(Debug, Clone)]
struct HeapEntry {
    freq: u64,
    node: HuffmanNode,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.freq.cmp(&self.freq)
    }
}

#[derive(Debug, Clone)]
pub struct HuffmanTree {
    root: HuffmanNode,
    single_symbol: Option<u8>,
}

impl HuffmanTree {
    pub fn from_frequencies(freqs: &[u64; 256]) -> Option<Self> {
        let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();

        for (symbol, &freq) in freqs.iter().enumerate() {
            if freq > 0 {
                heap.push(HeapEntry {
                    freq,
                    node: HuffmanNode::Leaf {
                        symbol: symbol as u8,
                    },
                });
            }
        }

        if heap.is_empty() {
            return None;
        }

        if heap.len() == 1 {
            // Single-symbol edge case: the tree has only one leaf node at the root.
            // Normally Huffman coding assigns a 0-bit code to this symbol, which is
            // degenerate — we can't store "zero bits" in a bit stream. Instead, we
            // detect this case and handle it specially in encode/decode by storing
            // the symbol and the repeat count directly, bypassing bit-level encoding.
            let entry = heap.pop().unwrap();
            let symbol = match &entry.node {
                HuffmanNode::Leaf { symbol } => *symbol,
                _ => unreachable!(),
            };
            return Some(Self {
                root: entry.node,
                single_symbol: Some(symbol),
            });
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            let merged = HeapEntry {
                freq: left.freq + right.freq,
                node: HuffmanNode::Internal {
                    left: Box::new(left.node),
                    right: Box::new(right.node),
                },
            };
            heap.push(merged);
        }

        Some(Self {
            root: heap.pop().unwrap().node,
            single_symbol: None,
        })
    }

    /// Build a lookup table: byte -> (bit_pattern, bit_count)
    fn build_encode_table(&self) -> HashMap<u8, (u64, usize)> {
        let mut table = HashMap::new();
        // Iterative traversal using an explicit stack to avoid stack overflow
        // on degenerate (deeply unbalanced) trees. Each entry is (node, prefix, depth).
        let mut stack: Vec<(&HuffmanNode, u64, usize)> = Vec::new();
        stack.push((&self.root, 0, 0));

        while let Some((node, prefix, depth)) = stack.pop() {
            // Safety guard: Huffman codes for byte-aligned data should never
            // exceed 255 bits. If we hit something deeper, the tree is malformed.
            if depth > 255 {
                // In practice this should never happen with valid frequency tables,
                // but we guard against stack-smashing from adversarial inputs.
                break;
            }
            match node {
                HuffmanNode::Leaf { symbol } => {
                    table.insert(*symbol, (prefix, depth));
                }
                HuffmanNode::Internal { left, right } => {
                    // Push right first so left is processed first (stack = LIFO)
                    stack.push((right, prefix | (1 << depth), depth + 1));
                    stack.push((left, prefix, depth + 1));
                }
            }
        }
        table
    }

    pub fn encode(&self, input: &[u8]) -> Vec<u8> {
        use crate::core::bit_buffer::{BitBuffer, BitMode};

        if let Some(symbol) = self.single_symbol {
            // Single symbol: store symbol + count for efficient compression
            let mut result = Vec::with_capacity(9);
            result.push(symbol);
            result.extend_from_slice(&(input.len() as u64).to_le_bytes());
            return result;
        }

        let table = self.build_encode_table();
        let mut buf = BitBuffer::new(BitMode::MsbFirst);

        for &byte in input {
            let (pattern, count) = table[&byte];
            // The encode table stores codes LSB-first (bit 0 = first Huffman bit).
            // Write bits one at a time from bit 0 upward so the first Huffman bit
            // is written first, preserving leading zeros correctly.
            for i in 0..count {
                let bit = ((pattern >> i) & 1) as u8;
                buf.write_bits(bit as u64, 1);
            }
        }

        let total_bits = buf.bit_len();
        let data_bytes = buf.as_bytes();
        // Prepend 8 bytes of bit-length header
        let mut result = Vec::with_capacity(8 + data_bytes.len());
        result.extend_from_slice(&(total_bits as u64).to_le_bytes());
        result.extend_from_slice(&data_bytes);
        result
    }

    pub fn decode(&self, input: &[u8]) -> Vec<u8> {
        if let Some(symbol) = self.single_symbol {
            // Single symbol case: first byte is the symbol, next 8 are the count
            let count = u64::from_le_bytes(input[1..9].try_into().unwrap()) as usize;
            return vec![symbol; count];
        }

        if input.len() < 8 {
            return Vec::new();
        }

        let total_bits = u64::from_le_bytes(input[0..8].try_into().unwrap()) as usize;
        let mut output = Vec::new();
        let mut current = &self.root;
        let mut bits_read = 0;

        for &byte in input.iter().skip(8) {
            for bit_pos in (0..8).rev() {
                if bits_read >= total_bits {
                    return output;
                }
                let bit = (byte >> bit_pos) & 1;
                match current {
                    HuffmanNode::Internal { left, right } => {
                        current = if bit == 0 { left } else { right };
                    }
                    HuffmanNode::Leaf { .. } => {
                        // Shouldn't happen mid-bit if tree is correct, but restart
                    }
                }
                bits_read += 1;
                if let HuffmanNode::Leaf { symbol } = current {
                    output.push(*symbol);
                    current = &self.root;
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tree_from_frequencies() {
        let mut freqs = [0u64; 256];
        freqs[b'a' as usize] = 5;
        freqs[b'b' as usize] = 3;
        freqs[b'c' as usize] = 1;
        let tree = HuffmanTree::from_frequencies(&freqs);
        assert!(tree.is_some());
    }

    #[test]
    fn empty_input_returns_none() {
        let freqs = [0u64; 256];
        let tree = HuffmanTree::from_frequencies(&freqs);
        assert!(tree.is_none());
    }

    #[test]
    fn single_symbol_returns_some() {
        let mut freqs = [0u64; 256];
        freqs[b'x' as usize] = 10;
        let tree = HuffmanTree::from_frequencies(&freqs);
        assert!(tree.is_some());
    }

    #[test]
    fn encode_decode_roundtrip() {
        let input = b"hello world hello rust";
        let freqs = crate::core::frequency::FrequencyTable::from_bytes(input);
        let tree = HuffmanTree::from_frequencies(freqs.as_array()).unwrap();

        let encoded = tree.encode(input);
        let decoded = tree.decode(&encoded);

        assert_eq!(decoded, input.to_vec());
    }

    #[test]
    fn encode_decode_all_byte_values() {
        let input: Vec<u8> = (0..=255).collect();
        let freqs = crate::core::frequency::FrequencyTable::from_bytes(&input);
        let tree = HuffmanTree::from_frequencies(freqs.as_array()).unwrap();

        let encoded = tree.encode(&input);
        let decoded = tree.decode(&encoded);

        assert_eq!(decoded, input);
    }

    #[test]
    fn encode_decode_repetitive() {
        let input = vec![b'A'; 10000];
        let freqs = crate::core::frequency::FrequencyTable::from_bytes(&input);
        let tree = HuffmanTree::from_frequencies(freqs.as_array()).unwrap();

        let encoded = tree.encode(&input);
        let decoded = tree.decode(&encoded);

        assert_eq!(decoded, input);
        // Highly repetitive data should compress well
        assert!(encoded.len() < input.len());
    }
}
