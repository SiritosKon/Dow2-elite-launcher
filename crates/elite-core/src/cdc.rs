//! Content-defined chunking (gear hash, FastCDC-style normalized chunking).
//!
//! Chunk boundaries depend only on nearby bytes, so inserting or shifting data inside a
//! file changes one or two chunks, not every chunk after it. The parameters, the gear
//! table seed and the chunk hash are part of the on-disk format: publisher and launcher
//! must produce identical chunks. Do not change them without bumping the manifest format.
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::OnceLock;

pub const MIN: u32 = 4096;
pub const AVG: u32 = 16384;
pub const MAX: u32 = 65536;
const MASK_S: u64 = 0xFFFF_0000_0000_0000; // before AVG: harder to cut (16 bits)
const MASK_L: u64 = 0xFFF0_0000_0000_0000; // after AVG: easier (12 bits)

fn gear() -> &'static [u64; 256] {
    static G: OnceLock<[u64; 256]> = OnceLock::new();
    G.get_or_init(|| {
        let mut g = [0u64; 256];
        let mut x: u64 = 0x456c_6974_6555_5044; // "EliteUPD"
        for v in g.iter_mut() {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            *v = z ^ (z >> 31);
        }
        g
    })
}

/// One chunk: its length and the first 8 bytes of SHA-256 of its content (little endian).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub len: u32,
    pub hash: u64,
}

/// Streaming chunker: feed any number of byte slices, then `finish`.
pub struct Chunker {
    h: u64,
    len: u32,
    sha: Sha256,
    pub chunks: Vec<Chunk>,
}

impl Default for Chunker {
    fn default() -> Self {
        Self { h: 0, len: 0, sha: Sha256::new(), chunks: Vec::new() }
    }
}

impl Chunker {
    fn emit(&mut self) {
        let d = std::mem::take(&mut self.sha).finalize();
        let mut b = [0u8; 8];
        b.copy_from_slice(&d[..8]);
        self.chunks.push(Chunk { len: self.len, hash: u64::from_le_bytes(b) });
        self.h = 0;
        self.len = 0;
    }

    pub fn feed(&mut self, data: &[u8]) {
        let g = gear();
        let mut start = 0;
        for (i, &b) in data.iter().enumerate() {
            self.h = (self.h << 1).wrapping_add(g[b as usize]);
            self.len += 1;
            if self.len < MIN {
                continue;
            }
            let m = if self.len < AVG { MASK_S } else { MASK_L };
            if self.h & m == 0 || self.len >= MAX {
                self.sha.update(&data[start..=i]);
                start = i + 1;
                self.emit();
            }
        }
        if start < data.len() {
            self.sha.update(&data[start..]);
        }
    }

    pub fn finish(mut self) -> Vec<Chunk> {
        if self.len > 0 {
            self.emit();
        }
        self.chunks
    }
}

/// Chunks + SHA-256 + size of everything readable from `r`.
pub fn scan(mut r: impl Read) -> std::io::Result<(Vec<Chunk>, [u8; 32], u64)> {
    let mut c = Chunker::default();
    let mut file_sha = Sha256::new();
    let mut buf = vec![0u8; 1 << 22];
    let mut size = 0u64;
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        c.feed(&buf[..n]);
        file_sha.update(&buf[..n]);
        size += n as u64;
    }
    Ok((c.finish(), file_sha.finalize().into(), size))
}

/// SHA-256 and size of a reader.
pub fn sha256(mut r: impl Read) -> std::io::Result<([u8; 32], u64)> {
    let mut s = Sha256::new();
    let mut buf = vec![0u8; 1 << 22];
    let mut size = 0u64;
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        s.update(&buf[..n]);
        size += n as u64;
    }
    Ok((s.finalize().into(), size))
}

pub fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}
