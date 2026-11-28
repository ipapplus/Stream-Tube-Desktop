//! In-app updates from the project's GitHub releases (pre-releases included).
use crate::api::http;
use crate::i18n::tf;
use crate::store;
use crate::ui::{blocking, toast};
use anyhow::{Context as _, Result, anyhow};
use gpui_kit::*;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};

const REPO: &str = "ipapplus/Stream-Tube-Desktop";
pub const RELEASES_PAGE: &str = "https://github.com/ipapplus/Stream-Tube-Desktop/releases";

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub name: String,
    pub notes: String,
    pub page: String,
    pub prerelease: bool,
    /// installer (Windows) or archive (Linux) for this platform
    pub asset: Option<Asset>,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
    /// SHA-256 that GitHub reports for the file (lowercase hex)
    pub sha256: String,
}

#[derive(Clone, Debug, Default)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(Release),
    Downloading(Release, f32),
    Error(String),
}

#[derive(Default)]
pub struct Updates {
    pub status: Status,
}
impl Global for Updates {}

static BUSY: AtomicBool = AtomicBool::new(false);

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// (major, minor, patch, stage, stage number). A plain release ranks above any
/// alpha / beta / rc of the same number.
fn parse_version(v: &str) -> (u64, u64, u64, u32, u64) {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let (core, pre) = v.split_once('-').unwrap_or((v, ""));
    let mut n = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    let (a, b, c) = (
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
    );
    let pre = pre.to_ascii_lowercase();
    let stage = if pre.is_empty() {
        9
    } else if pre.starts_with("rc") {
        3
    } else if pre.starts_with("beta") {
        2
    } else if pre.starts_with("alpha") {
        1
    } else {
        0
    };
    let num = pre
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    (a, b, c, stage, num)
}

pub fn is_newer(tag: &str, current: &str) -> bool {
    parse_version(tag) > parse_version(current)
}

fn pick_asset(tag: &str, assets: &[Value]) -> Option<Asset> {
    let version = tag.strip_prefix('v')?;
    let filename = format!("StreamTube-{version}-setup.exe");
    assets.iter().find_map(|a| {
        let name = a.get("name")?.as_str()?;
        let want = if cfg!(windows) {
            name == filename
        } else {
            name.contains("linux") && (name.ends_with(".tar.gz") || name.ends_with(".AppImage"))
        };
        if !want
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return None;
        }
        let url = a.get("browser_download_url")?.as_str()?;
        if url != format!("{RELEASES_PAGE}/download/{tag}/{name}") {
            return None;
        }
        let hash = a.get("digest")?.as_str()?.strip_prefix("sha256:")?;
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        Some(Asset {
            name: name.to_string(),
            url: url.to_string(),
            size: a.get("size").and_then(Value::as_u64).unwrap_or(0),
            sha256: hash.to_ascii_lowercase(),
        })
    })
}

/// SHA-256 of a file, as lowercase hex.
pub(crate) fn file_sha256(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The newest published release, if it is newer than this build.
pub fn check() -> Result<Option<Release>> {
    let v = http::get_json(&format!(
        "https://api.github.com/repos/{REPO}/releases?per_page=15"
    ))?;
    let list = v
        .as_array()
        .ok_or_else(|| anyhow!("unexpected reply from GitHub"))?;
    let newest = list
        .iter()
        .filter(|r| !r.get("draft").and_then(Value::as_bool).unwrap_or(false))
        .filter_map(|r| {
            let tag = r.get("tag_name")?.as_str()?.strip_prefix('v')?;
            let core = tag.split('-').next()?;
            if core.split('.').count() != 3
                || !core.split('.').all(|part| part.parse::<u64>().is_ok())
                || !tag
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-.".contains(&b))
            {
                return None;
            }
            Some((format!("v{tag}"), r))
        })
        .max_by(|a, b| parse_version(&a.0).cmp(&parse_version(&b.0)));
    let Some((tag, r)) = newest else {
        return Ok(None);
    };
    if !is_newer(&tag, current_version()) {
        return Ok(None);
    }
    let s = |k: &str| {
        r.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    Ok(Some(Release {
        name: Some(s("name"))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| tag.clone()),
        notes: s("body"),
        page: format!("{RELEASES_PAGE}/tag/{tag}"),
        prerelease: r
            .get("prerelease")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        asset: r
            .get("assets")
            .and_then(Value::as_array)
            .and_then(|a| pick_asset(&tag, a)),
        tag,
    }))
}

fn set_status(st: Status, cx: &mut App) {
    cx.default_global::<Updates>().status = st;
    cx.refresh_windows();
}

/// Checks GitHub. `manual` reports "up to date" and errors; the startup check stays
/// quiet unless there is something new (and not skipped).
pub fn check_now(manual: bool, cx: &mut App) {
    if matches!(
        cx.default_global::<Updates>().status,
        Status::Downloading(_, _)
    ) || BUSY.swap(true, Ordering::SeqCst)
    {
        return;
    }
    set_status(Status::Checking, cx);
    cx.spawn(async move |cx| {
        let res = blocking(check).await;
        BUSY.store(false, Ordering::SeqCst);
        let _ = cx.update(|cx| match res {
            Ok(Some(rel)) => {
                let skipped = store::kv_get("update_skip").as_deref() == Some(rel.tag.as_str());
                set_status(Status::Available(rel.clone()), cx);
                if manual || !skipped {
                    show_dialog(cx);
                }
            }
            Ok(None) => {
                set_status(Status::UpToDate, cx);
                if manual {
                    toast(
                        cx,
                        tf!("Stream Tube {0} is the latest version", current_version()),
                    );
                }
            }
            Err(e) => {
                set_status(
                    if manual {
                        Status::Error(e.to_string())
                    } else {
                        Status::Idle
                    },
                    cx,
                );
                if manual {
                    toast(cx, tf!("Could not check for updates: {0}", e));
                }
            }
        });
    })
    .detach();
}

/// Startup check, a few seconds after launch.
pub fn check_on_startup(cx: &mut App) {
    if !store::settings_snapshot().check_updates {
        return;
    }
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(std::time::Duration::from_secs(6))
            .await;
        let _ = cx.update(|cx| check_now(false, cx));
    })
    .detach();
}

fn show_dialog(cx: &mut App) {
    if let Some(w) = cx
        .active_window()
        .or_else(|| cx.windows().into_iter().next())
    {
        let _ = w.update(cx, |_, window, cx| {
            crate::ui::app::open_dialog(crate::ui::app::Dialog::Update, window, cx)
        });
    }
}

pub fn skip(cx: &mut App) {
    if let Status::Available(r) = &cx.default_global::<Updates>().status {
        store::kv_set("update_skip", &r.tag);
    }
}

/// Downloads the new version and starts installing it. On Windows the installer
/// runs silently, replaces the app and opens it again.
pub fn install(cx: &mut App) {
    let rel = match &cx.default_global::<Updates>().status {
        Status::Available(r) => r.clone(),
        _ => return,
    };
    let Some(Asset {
        name,
        url,
        size,
        sha256,
    }) = rel.asset.clone().filter(|_| cfg!(windows))
    else {
        // no installer for this platform: the release page has the downloads
        let _ = open::that(&rel.page);
        return;
    };
    set_status(Status::Downloading(rel.clone(), 0.0), cx);
    let progress = std::sync::Arc::new(parking_lot::Mutex::new(0.0f32));
    let p2 = progress.clone();
    let result: std::sync::Arc<parking_lot::Mutex<Option<Result<std::path::PathBuf>>>> =
        Default::default();
    let r2 = result.clone();
    std::thread::spawn(move || {
        let res = (|| -> Result<std::path::PathBuf> {
            // the app's own folder, not a predictable path in a shared temp folder
            let dir = crate::store::data_dir().join("update");
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(&name);
            let mut f = std::fs::File::create(&path)?;
            http::download_to(&url, &mut f, |done, total| {
                let total = total.unwrap_or(size).max(1);
                *p2.lock() = (done as f32 / total as f32).min(1.0);
                true
            })
            .context("download failed")?;
            drop(f);
            if size > 0 && std::fs::metadata(&path)?.len() != size {
                anyhow::bail!("the download is incomplete");
            }
            // the installer runs only if it is exactly the file GitHub has on the release
            if file_sha256(&path)? != sha256 {
                let _ = std::fs::remove_file(&path);
                anyhow::bail!("the download doesn't match the release's checksum");
            }
            Ok(path)
        })();
        *r2.lock() = Some(res);
    });
    cx.spawn(async move |cx| {
        let res = loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(250))
                .await;
            if let Some(r) = result.lock().take() {
                break r;
            }
            let p = *progress.lock();
            let rel2 = rel.clone();
            cx.update(|cx| set_status(Status::Downloading(rel2, p), cx));
        };
        cx.update(|cx| match res {
            Ok(path) => run_installer(&path, rel, cx),
            Err(e) => {
                set_status(Status::Available(rel), cx);
                toast(cx, tf!("Update failed: {0}", e));
            }
        });
    })
    .detach();
}

fn installer_command(
    path: &std::path::Path,
    executable: Option<&std::path::Path>,
) -> std::process::Command {
    let mut command = std::process::Command::new(path);
    command.args([
        "/SP-",
        "/SILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART",
        "/CLOSEAPPLICATIONS",
        "/RESTARTAPPLICATIONS",
    ]);
    if let Some(dir) = executable.and_then(portable_dir) {
        let mut destination = std::ffi::OsString::from("/DIR=");
        destination.push(dir);
        command.arg("/PORTABLEUPDATE=1").arg(destination);
    }
    command
}

fn portable_dir(executable: &std::path::Path) -> Option<&std::path::Path> {
    let dir = executable.parent()?;
    let name = executable.file_name()?.to_str()?;
    let portable = dir.join("streamtube.portable").is_file() || dir.join("README.md").is_file();
    (name.eq_ignore_ascii_case("streamtube.exe") && portable && dir.join("libmpv-2.dll").is_file())
        .then_some(dir)
}

fn run_installer(path: &std::path::Path, rel: Release, cx: &mut App) {
    let executable = std::env::current_exe().ok();
    let mut command = installer_command(path, executable.as_deref());
    match command.spawn() {
        Ok(_) => {
            store::kv_set("update_skip", "");
            crate::flush_watch_history(cx);
            cx.quit();
        }
        Err(e) => {
            set_status(Status::Available(rel), cx);
            toast(cx, tf!("Could not start the installer: {0}", e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{installer_command, is_newer, pick_asset};

    #[test]
    fn portable_update_targets_the_running_copy() {
        let dir = crate::store::data_dir().join("Папка Stream Tube");
        std::fs::create_dir_all(&dir).unwrap();
        let executable = dir.join("streamtube.exe");
        let setup = std::path::Path::new("update-setup.exe");
        std::fs::write(dir.join("libmpv-2.dll"), []).unwrap();
        assert!(
            !installer_command(setup, Some(&executable))
                .get_args()
                .any(|arg| arg == "/PORTABLEUPDATE=1")
        );

        // ZIPs released before the marker shipped include the README.
        std::fs::write(dir.join("README.md"), []).unwrap();
        let command = installer_command(setup, Some(&executable));
        let mut destination = std::ffi::OsString::from("/DIR=");
        destination.push(dir);
        assert!(command.get_args().any(|arg| arg == "/PORTABLEUPDATE=1"));
        assert!(command.get_args().any(|arg| arg == destination));
    }
    #[test]
    fn versions() {
        assert!(is_newer("v0.1.0", "0.1.0-alpha"));
        assert!(is_newer("v0.1.0-beta", "0.1.0-alpha"));
        assert!(is_newer("v0.1.0-alpha.2", "0.1.0-alpha"));
        assert!(is_newer("v0.2.0-alpha", "0.1.0"));
        assert!(!is_newer("v0.1.0-alpha", "0.1.0-alpha"));
        assert!(!is_newer("v0.1.0-alpha", "0.1.0"));
    }

    #[test]
    fn assets_come_from_the_arabic_fork_with_a_checksum() {
        let tag = "v1.0.0";
        let good = if cfg!(windows) {
            "StreamTube-1.0.0-setup.exe"
        } else {
            "StreamTube-1.0.0-linux-x86_64.tar.gz"
        };
        let official_url = format!("{}/download/{tag}/{good}", super::RELEASES_PAGE);
        let asset = |name: &str, url: &str, digest: serde_json::Value| serde_json::json!({ "name": name, "browser_download_url": url, "size": 5, "digest": digest });
        let digest = serde_json::json!(format!("sha256:{}", "AB".repeat(32)));
        let ok = pick_asset(tag, &[asset(good, &official_url, digest.clone())]).unwrap();
        assert_eq!(ok.sha256, "ab".repeat(32));
        for repo in [
            "Avaneesh-Inamdar/Stream-Tube-Desktop",
            "StreamTubeProject/Stream-Tube-Desktop",
            "ipapplus/Stream-Tube-Desktop-fork",
        ] {
            let url = format!("https://github.com/{repo}/releases/download/{tag}/{good}");
            assert!(pick_asset(tag, &[asset(good, &url, digest.clone())]).is_none());
        }
        assert!(pick_asset("v1.0.1", &[asset(good, &official_url, digest.clone())]).is_none());
        assert!(pick_asset(tag, &[asset(&format!("../{good}"), &official_url, digest)]).is_none());
        for invalid in [serde_json::Value::Null, serde_json::json!("sha256:wrong")] {
            assert!(pick_asset(tag, &[asset(good, &official_url, invalid)]).is_none());
        }
    }
}
