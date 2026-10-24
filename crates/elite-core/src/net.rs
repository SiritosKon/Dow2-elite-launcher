//! HTTP with several mirrors. A mirror is any static host that supports Range requests.
//! Content is verified by SHA-256 and signatures, so mirrors do not need to be trusted.
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct Mirrors {
    bases: Vec<String>,
    dead: Vec<AtomicBool>,
    current: std::sync::atomic::AtomicUsize,
    agent: ureq::Agent,
    pub downloaded: Arc<AtomicU64>,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum NetError {
    Cancelled,
    Failed(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetError::Cancelled => write!(f, "cancelled"),
            NetError::Failed(s) => write!(f, "{s}"),
        }
    }
}

impl Mirrors {
    pub fn new(bases: Vec<String>) -> Self {
        let mut uniq: Vec<String> = Vec::new();
        for b in bases {
            let b = if b.ends_with('/') { b } else { format!("{b}/") };
            if (b.starts_with("https://") || b.starts_with("http://")) && !uniq.contains(&b) {
                uniq.push(b);
            }
        }
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(8))
            .timeout_read(Duration::from_secs(30))
            .user_agent(concat!("EliteLauncher/", env!("CARGO_PKG_VERSION")))
            .try_proxy_from_env(true)
            .build();
        Mirrors {
            dead: uniq.iter().map(|_| AtomicBool::new(false)).collect(),
            bases: uniq,
            current: 0.into(),
            agent,
            downloaded: Arc::new(AtomicU64::new(0)),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Base URL of the mirror that answered last (to try it first next time).
    pub fn current_base(&self) -> String {
        self.bases.get(self.current.load(Ordering::Relaxed)).cloned().unwrap_or_default()
    }

    pub fn current_host(&self) -> String {
        self.bases.get(self.current.load(Ordering::Relaxed)).map(|b| host_of(b)).unwrap_or_default()
    }

    pub fn count(&self) -> usize {
        self.bases.len()
    }

    /// Try mirrors in order starting from the last one that worked.
    fn each<T>(&self, mut f: impl FnMut(&str) -> Result<T, (bool, String)>) -> Result<T, NetError> {
        let n = self.bases.len();
        if n == 0 {
            return Err(NetError::Failed("no mirrors configured".into()));
        }
        let start = self.current.load(Ordering::Relaxed);
        let mut last = String::new();
        for k in 0..n {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(NetError::Cancelled);
            }
            let i = (start + k) % n;
            if self.dead[i].load(Ordering::Relaxed) && k < n - 1 {
                continue;
            }
            match f(&self.bases[i]) {
                Ok(v) => {
                    self.current.store(i, Ordering::Relaxed);
                    self.dead[i].store(false, Ordering::Relaxed);
                    return Ok(v);
                }
                Err((not_found, e)) => {
                    last = format!("{}: {e}", host_of(&self.bases[i]));
                    if !not_found {
                        self.dead[i].store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        Err(NetError::Failed(last))
    }

    /// Whole small file (configs, manifests).
    pub fn get(&self, rel: &str) -> Result<Vec<u8>, NetError> {
        self.each(|base| {
            let r = self.agent.get(&format!("{base}{rel}")).call().map_err(|e| (matches!(e, ureq::Error::Status(404, _)), e.to_string()))?;
            let mut v = Vec::new();
            r.into_reader().take(64 << 20).read_to_end(&mut v).map_err(|e| (false, e.to_string()))?;
            self.downloaded.fetch_add(v.len() as u64, Ordering::Relaxed);
            Ok(v)
        })
    }

    /// Bytes [from, to) of `rel` appended to `out`. On a broken transfer the partial write
    /// is rolled back and the range is fetched from the next mirror.
    pub fn get_range<W: Write + Seek>(&self, rel: &str, from: u64, to: u64, out: &mut W) -> Result<(), NetError> {
        let start = out.stream_position().map_err(|e| NetError::Failed(e.to_string()))?;
        self.each(|base| {
            out.seek(SeekFrom::Start(start)).map_err(|e| (false, e.to_string()))?;
            let r = self
                .agent
                .get(&format!("{base}{rel}"))
                .set("Range", &format!("bytes={}-{}", from, to - 1))
                .call()
                .map_err(|e| (matches!(e, ureq::Error::Status(404, _)), e.to_string()))?;
            if r.status() != 206 {
                return Err((false, format!("no range support (HTTP {})", r.status())));
            }
            let mut rd = r.into_reader();
            let mut buf = vec![0u8; 1 << 18];
            let mut got = 0u64;
            loop {
                if self.cancel.load(Ordering::Relaxed) {
                    return Err((true, "cancelled".into()));
                }
                let n = rd.read(&mut buf).map_err(|e| (false, e.to_string()))?;
                if n == 0 {
                    break;
                }
                out.write_all(&buf[..n]).map_err(|e| (true, e.to_string()))?;
                got += n as u64;
                self.downloaded.fetch_add(n as u64, Ordering::Relaxed);
            }
            if got != to - from {
                return Err((false, format!("short read {got}/{}", to - from)));
            }
            Ok(())
        })
        .map_err(|e| if self.cancel.load(Ordering::Relaxed) { NetError::Cancelled } else { e })
    }
}

fn host_of(b: &str) -> String {
    b.split("://").nth(1).unwrap_or(b).split('/').next().unwrap_or("").to_string()
}
