//! Version manifest: list of files with size, SHA-256 and chunk list.
//!
//! Binary, little endian (format "ELUPD001"):
//! magic[8] | u32 len + version | u32 nfiles |
//! per file: u16 len + path ('/'-separated, relative to the game folder), u64 size, sha256[32],
//!           u32 nchunks, nchunks * (u32 len, u64 hash)
//! File content is stored on mirrors as `blobs/<sha256 hex>` (identical files are stored once).
//! The manifest is signed: `<version>.manifest.sig` (see sign.rs).
use crate::cdc::Chunk;

pub const MAGIC: &[u8; 8] = b"ELUPD001";

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub sha: [u8; 32],
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<FileEntry>,
}

impl Manifest {
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(MAGIC);
        o.extend_from_slice(&(self.version.len() as u32).to_le_bytes());
        o.extend_from_slice(self.version.as_bytes());
        o.extend_from_slice(&(self.files.len() as u32).to_le_bytes());
        for f in &self.files {
            o.extend_from_slice(&(f.path.len() as u16).to_le_bytes());
            o.extend_from_slice(f.path.as_bytes());
            o.extend_from_slice(&f.size.to_le_bytes());
            o.extend_from_slice(&f.sha);
            o.extend_from_slice(&(f.chunks.len() as u32).to_le_bytes());
            for c in &f.chunks {
                o.extend_from_slice(&c.len.to_le_bytes());
                o.extend_from_slice(&c.hash.to_le_bytes());
            }
        }
        o
    }

    pub fn parse(b: &[u8]) -> Result<Manifest, String> {
        let mut p = 0usize;
        let take = |p: &mut usize, n: usize| -> Result<&[u8], String> {
            if b.len() < *p + n {
                return Err("manifest truncated".into());
            }
            let s = &b[*p..*p + n];
            *p += n;
            Ok(s)
        };
        if take(&mut p, 8)? != MAGIC {
            return Err("not an Elite manifest".into());
        }
        let u32le = |s: &[u8]| u32::from_le_bytes(s.try_into().unwrap());
        let vl = u32le(take(&mut p, 4)?) as usize;
        if vl > 64 {
            return Err("bad version".into());
        }
        let version = String::from_utf8(take(&mut p, vl)?.to_vec()).map_err(|e| e.to_string())?;
        let n = u32le(take(&mut p, 4)?) as usize;
        let mut files = Vec::with_capacity(n.min(100_000));
        for _ in 0..n {
            let pl = u16::from_le_bytes(take(&mut p, 2)?.try_into().unwrap()) as usize;
            let path = String::from_utf8(take(&mut p, pl)?.to_vec()).map_err(|e| e.to_string())?;
            if path.contains("..") || path.starts_with('/') || path.contains('\\') || path.contains(':') {
                return Err(format!("unsafe path in manifest: {path}"));
            }
            let size = u64::from_le_bytes(take(&mut p, 8)?.try_into().unwrap());
            let mut sha = [0u8; 32];
            sha.copy_from_slice(take(&mut p, 32)?);
            let nc = u32le(take(&mut p, 4)?) as usize;
            let raw = take(&mut p, nc * 12)?;
            let chunks = raw
                .chunks_exact(12)
                .map(|c| Chunk { len: u32le(&c[..4]), hash: u64::from_le_bytes(c[4..].try_into().unwrap()) })
                .collect();
            files.push(FileEntry { path, size, sha, chunks });
        }
        Ok(Manifest { version, files })
    }
}
