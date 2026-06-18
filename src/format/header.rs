use crate::core::error::CompressionError;

#[derive(Debug, Clone)]
pub struct GzipHeader {
    pub mtime: u32,
    pub os: u8,
}

impl GzipHeader {
    pub fn encode(&self) -> Vec<u8> {
        let mut header = vec![0u8; 10];
        header[0] = 0x1f;       // Magic ID1
        header[1] = 0x8b;       // Magic ID2
        header[2] = 0x08;       // CM = DEFLATE
        header[3] = 0x00;       // FLG = no flags
        header[4..8].copy_from_slice(&self.mtime.to_le_bytes()); // MTIME
        header[8] = 0x00;       // XFL
        header[9] = self.os;    // OS
        header
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, CompressionError> {
        if bytes.len() < 10 {
            return Err(CompressionError::Truncated {
                expected: 10,
                actual: bytes.len(),
            });
        }
        if bytes[0] != 0x1f || bytes[1] != 0x8b {
            return Err(CompressionError::InvalidData(
                "invalid gzip magic bytes".into(),
            ));
        }
        if bytes[2] != 0x08 {
            return Err(CompressionError::UnsupportedMethod(bytes[2]));
        }

        let mtime = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let os = bytes[9];

        Ok(Self { mtime, os })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let header = GzipHeader {
            mtime: 1700000000,
            os: 0xFF,
        };
        let bytes = header.encode();
        assert_eq!(bytes.len(), 10);
        assert_eq!(bytes[0], 0x1f);
        assert_eq!(bytes[1], 0x8b);
        assert_eq!(bytes[2], 0x08); // DEFLATE

        let decoded = GzipHeader::decode(&bytes).unwrap();
        assert_eq!(decoded.mtime, 1700000000);
        assert_eq!(decoded.os, 0xFF);
    }

    #[test]
    fn decode_invalid_magic() {
        let mut bytes = vec![0u8; 10];
        bytes[0] = 0x00; // wrong magic
        assert!(GzipHeader::decode(&bytes).is_err());
    }

    #[test]
    fn decode_wrong_length() {
        let bytes = vec![0x1f, 0x8b, 0x08];
        assert!(GzipHeader::decode(&bytes).is_err());
    }
}
