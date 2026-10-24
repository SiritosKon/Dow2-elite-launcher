//! elite-publish — tool for the Elite team.
//!
//!   elite-publish keygen <dir>                         new key pair: <dir>/private.key (keep secret!), <dir>/public.key
//!   elite-publish version <build_dir> <store> <version> <private.key> [file_list]
//!         Publishes a build: <store>/<version>.manifest(+.sig) and <store>/blobs/<sha256>.
//!         file_list: text file with paths relative to build_dir (default: every file in build_dir).
//!   elite-publish sign <file> <private.key>            writes <file>.sig (use for launcher.json after editing)
//!   elite-publish verify <store>                       checks launcher.json and all manifests against keys/public.key
//!
//! Upload <store> as-is to every mirror (any static hosting with HTTP Range support).
use elite_core::{cdc, manifest::{FileEntry, Manifest}, sign};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let r = match a.get(1).map(|s| s.as_str()) {
        Some("keygen") if a.len() == 3 => keygen(&a[2]),
        Some("version") if a.len() == 6 || a.len() == 7 => version(&a[2], &a[3], &a[4], &a[5], a.get(6)),
        Some("sign") if a.len() == 4 => sign_file(&a[2], &a[3]),
        Some("verify") if a.len() == 3 => verify(&a[2]),
        _ => Err("usage: see the comment at the top of crates/elite-publish/src/main.rs (keygen | version | sign | verify)".into()),
    };
    if let Err(e) = r {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn keygen(dir: &str) -> Result<(), String> {
    let d = Path::new(dir);
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    if d.join("private.key").exists() {
        return Err("private.key already exists, refusing to overwrite".into());
    }
    let (sk, pk) = sign::keygen();
    fs::write(d.join("private.key"), sk + "\n").map_err(|e| e.to_string())?;
    fs::write(d.join("public.key"), pk + "\n").map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(d.join("private.key"), fs::Permissions::from_mode(0o600));
    }
    println!("keys written to {dir}. Put public.key into keys/ of the launcher source and rebuild.");
    Ok(())
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            walk(base, &p, out);
        } else if let Ok(rel) = p.strip_prefix(base) {
            out.push(rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect::<Vec<_>>().join("/"));
        }
    }
}

fn version(build: &str, store: &str, ver: &str, key: &str, list: Option<&String>) -> Result<(), String> {
    if ver.is_empty() || ver.contains(['/', '\\']) || ver.starts_with('.') {
        return Err("bad version id".into());
    }
    let secret = fs::read_to_string(key).map_err(|e| format!("{key}: {e}"))?;
    let build = PathBuf::from(build);
    let store = PathBuf::from(store);
    let blobs = store.join("blobs");
    fs::create_dir_all(&blobs).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    match list {
        Some(l) => files = fs::read_to_string(l).map_err(|e| e.to_string())?.lines().map(|s| s.trim().replace('\\', "/")).filter(|s| !s.is_empty()).collect(),
        None => walk(&build, &build, &mut files),
    }
    let mut man = Manifest { version: ver.into(), files: Vec::new() };
    let (mut total, mut chunks, mut new_blobs) = (0u64, 0usize, 0u64);
    for rel in files {
        let src = build.join(&rel);
        let f = fs::File::open(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        let (ch, sha, size) = cdc::scan(f).map_err(|e| e.to_string())?;
        let blob = blobs.join(cdc::hex(&sha));
        if !blob.exists() {
            let tmp = blobs.join(".incoming");
            fs::copy(&src, &tmp).map_err(|e| e.to_string())?;
            fs::rename(&tmp, &blob).map_err(|e| e.to_string())?;
            new_blobs += size;
        }
        println!("{:>10.1} MB {:>6} chunks  {}", size as f64 / 1e6, ch.len(), rel);
        total += size;
        chunks += ch.len();
        man.files.push(FileEntry { path: rel, size, sha, chunks: ch });
    }
    let bytes = man.to_bytes();
    let sig = sign::sign(&secret, &bytes)?;
    fs::write(store.join(format!("{ver}.manifest")), &bytes).map_err(|e| e.to_string())?;
    fs::write(store.join(format!("{ver}.manifest.sig")), sig + "\n").map_err(|e| e.to_string())?;
    println!(
        "version {ver}: {} files, {:.1} MB, {chunks} chunks, new blob data {:.1} MB, manifest {:.1} MB",
        man.files.len(),
        total as f64 / 1e6,
        new_blobs as f64 / 1e6,
        bytes.len() as f64 / 1e6
    );
    println!("Now point a channel to it in {}/launcher.json and run: elite-publish sign {}/launcher.json {key}", store.display(), store.display());
    Ok(())
}

fn sign_file(file: &str, key: &str) -> Result<(), String> {
    let secret = fs::read_to_string(key).map_err(|e| format!("{key}: {e}"))?;
    let data = fs::read(file).map_err(|e| format!("{file}: {e}"))?;
    if file.ends_with(".json") {
        serde_json::from_slice::<serde_json::Value>(&data).map_err(|e| format!("{file} is not valid JSON: {e}"))?;
    }
    fs::write(format!("{file}.sig"), sign::sign(&secret, &data)? + "\n").map_err(|e| e.to_string())?;
    println!("signed {file}");
    Ok(())
}

fn verify(store: &str) -> Result<(), String> {
    let mut bad = 0;
    let rd = fs::read_dir(store).map_err(|e| e.to_string())?;
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name.ends_with(".manifest") || name == "launcher.json" {
            let ok = fs::read(&p).ok().zip(fs::read_to_string(format!("{}.sig", p.display())).ok()).map(|(d, s)| sign::verify(&d, &s)).unwrap_or(false);
            println!("{} {name}", if ok { "OK " } else { "BAD" });
            bad += (!ok) as i32;
        }
    }
    if bad > 0 { Err(format!("{bad} file(s) failed")) } else { Ok(()) }
}
