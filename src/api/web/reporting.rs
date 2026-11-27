//! Reports playback to the account's watch history the way YouTube's own
//! players do: a playback ping at start, then watchtime pings as it goes. The
//! account is the browser session when connected, otherwise the TV sign-in.
use super::{ORIGIN, http};
use crate::api::history::Account;
use anyhow::{Result, anyhow};
use parking_lot::Mutex;
use serde_json::Value;
use std::sync::{OnceLock, mpsc};
use std::time::{Duration, Instant};

/// The web player's `signatureTimestamp`, for player requests through a browser session.
pub(in crate::api) fn sts() -> Option<u64> {
    static STS: Mutex<Option<(u64, Instant)>> = Mutex::new(None);
    if let Some((v, at)) = *STS.lock() {
        if at.elapsed() < Duration::from_secs(6 * 3600) {
            return Some(v);
        }
    }
    let api =
        String::from_utf8(http::get_bytes("https://www.youtube.com/iframe_api", &[]).ok()?).ok()?;
    let id: String = api
        .split("/s/player/")
        .nth(1)
        .or_else(|| api.split(r"player\/").nth(1))?
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();
    let js = String::from_utf8(
        http::get_bytes(
            &format!("https://www.youtube.com/s/player/{id}/player_ias.vflset/en_US/base.js"),
            &[],
        )
        .ok()?,
    )
    .ok()?;
    let v: u64 = ["signatureTimestamp:", "sts:"].iter().find_map(|k| {
        js.split(k).nth(1).and_then(|t| {
            t.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        })
    })?;
    *STS.lock() = Some((v, Instant::now()));
    Some(v)
}

struct Watch {
    account: Account,
    recording_revision: u64,
    id: String,
    playback: String,
    watchtime: String,
    cpn: String,
    len: f64,
    started: Instant,
    last_ping: Instant,
    last_cmt: f64,
    pings: u32,
    playing: bool,
}

impl Watch {
    fn begin(start: &Start) -> Result<Self> {
        let Start {
            account,
            id,
            len,
            position,
            ..
        } = start;
        anyhow::ensure!(
            enabled(start.recording_revision),
            "history reporting is inactive"
        );
        let r = account.player_response(id)?;
        let url = |k: &str| {
            r.pointer(&format!("/playbackTracking/{k}/baseUrl"))
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        let playback =
            url("videostatsPlaybackUrl").ok_or_else(|| anyhow!("no playback tracking"))?;
        let watchtime =
            url("videostatsWatchtimeUrl").ok_or_else(|| anyhow!("no watchtime tracking"))?;
        let len = r
            .pointer("/videoDetails/lengthSeconds")
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok())
            .filter(|l: &f64| *l > 0.0)
            .unwrap_or(*len);
        let cpn: String = {
            use rand::Rng;
            const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            let mut rng = rand::rng();
            (0..16)
                .map(|_| A[rng.random_range(0..A.len())] as char)
                .collect()
        };
        let now = Instant::now();
        let w = Watch {
            account: account.clone(),
            recording_revision: start.recording_revision,
            id: id.clone(),
            playback,
            watchtime,
            cpn,
            len,
            started: now,
            last_ping: now,
            last_cmt: *position,
            pings: 0,
            playing: true,
        };
        w.get(
            &w.playback,
            &[
                ("cmt", format!("{position:.3}")),
                ("rtn", "0".into()),
                ("state", "playing".into()),
            ],
        )?;
        Ok(w)
    }

    /// Sends a stats ping: the base URL from the player response with the fields
    /// youtube.com's own player adds. Fields already in the base URL are replaced,
    /// never repeated.
    fn get(&self, base: &str, fields: &[(&str, String)]) -> Result<()> {
        // the ping carries the sign-in cookies: only ever to YouTube itself
        if !is_youtube_https(base) {
            return Err(anyhow!("unexpected stats host"));
        }
        let rt = self.started.elapsed().as_secs_f64();
        let (client, version, platform) = self.account.ping_client();
        let mut add: Vec<(&str, String)> = vec![
            ("ver", "2".into()),
            ("cpn", self.cpn.clone()),
            ("c", client.into()),
            ("cver", version.into()),
            ("cplayer", "UNIPLAYER".into()),
            ("cplatform", platform.into()),
        ];
        if client == "WEB" {
            add.extend([
                ("cbr", "Chrome".into()),
                ("cbrver", "140.0.0.0".into()),
                ("cos", "Windows".into()),
                ("cosver", "10.0".into()),
            ]);
        } else {
            add.extend([("cos", "Tizen".into()), ("cosver", "2.4.0".into())]);
        }
        add.extend([
            ("fs", "0".into()),
            ("volume", "100".into()),
            ("muted", "0".into()),
            ("lact", "1200".into()),
            ("len", format!("{:.3}", self.len)),
            ("rt", format!("{rt:.3}")),
        ]);
        add.extend(fields.iter().cloned());
        let (path, query) = base.split_once('?').unwrap_or((base, ""));
        let mut pairs: Vec<(String, String)> = query
            .split('&')
            .filter(|kv| !kv.is_empty())
            .map(|kv| {
                let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
                (k.to_string(), v.to_string())
            })
            .collect();
        for (k, v) in add {
            match pairs.iter_mut().find(|(pk, _)| pk == k) {
                Some(p) => p.1 = v,
                None => pairs.push((k.to_string(), v)),
            }
        }
        let url = format!(
            "{path}?{}",
            pairs
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        );
        let referer = format!("https://www.youtube.com/watch?v={}", self.id);
        let account_headers = self.account.ping_headers()?;
        let mut headers: Vec<(&str, &str)> = account_headers
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();
        headers.push(("Referer", referer.as_str()));
        headers.push(("Origin", ORIGIN));
        let send = || {
            anyhow::ensure!(
                self.account.if_current(|| ()).is_some() && enabled(self.recording_revision),
                "history reporting is inactive"
            );
            Ok(http::tracking_status(&url, &headers))
        };
        let response = match send()? {
            Ok(500..=599) | Err(_) => send()?,
            response => response,
        };
        match response {
            Ok(200..=299) => Ok(()),
            Ok(status) => Err(anyhow!("HTTP {status}")),
            Err(_) => Err(anyhow!("network request failed")),
        }
    }

    /// Reports the stretch watched since the last report, ending at `cmt`.
    fn report(&mut self, cmt: f64, playing: bool, last: bool) -> Result<()> {
        let st = if cmt >= self.last_cmt && cmt - self.last_cmt < 120.0 {
            self.last_cmt
        } else {
            cmt
        };
        let rtn = self.started.elapsed().as_secs_f64();
        let mut fields = vec![
            ("cmt", format!("{cmt:.3}")),
            ("st", format!("{st:.3}")),
            ("et", format!("{cmt:.3}")),
            (
                "state",
                if playing { "playing" } else { "paused" }.to_string(),
            ),
            ("rtn", format!("{:.0}", rtn.max(1.0))),
            ("rti", format!("{:.0}", rtn.max(1.0))),
            ("idpj", "-1".into()),
            ("ldpj", "-1".into()),
        ];
        if last {
            fields.push(("final", "1".into()));
        }
        self.last_ping = Instant::now();
        self.playing = playing;
        self.get(&self.watchtime, &fields)?;
        self.last_cmt = cmt;
        self.pings += 1;
        Ok(())
    }

    /// youtube.com reports at 10, 20 and 30 seconds, then every 40 seconds.
    fn due(&self) -> bool {
        let every = if self.pings < 3 { 10 } else { 40 };
        self.last_ping.elapsed() >= Duration::from_secs(every)
    }
}

struct Start {
    account: Account,
    recording_revision: u64,
    id: String,
    serial: u64,
    len: f64,
    position: f64,
}

enum Event {
    Start(Start),
    Progress {
        serial: u64,
        position: f64,
        playing: bool,
        force: bool,
    },
    Seek {
        serial: u64,
        position: f64,
    },
    Stop {
        serial: u64,
        position: f64,
    },
    /// Forget the playback without a final report.
    Abandon {
        serial: u64,
    },
    Flush(mpsc::Sender<()>),
}

struct ActivePlayback {
    start: Start,
    watch: Option<Watch>,
    last_start_attempt: Instant,
}

static REPORTER: OnceLock<mpsc::Sender<Event>> = OnceLock::new();

fn enabled(revision: u64) -> bool {
    crate::store::history_recording_revision() == Some(revision)
}

fn report(watch: &mut Watch, position: f64, playing: bool, final_report: bool) {
    if watch.account.if_current(|| ()).is_some() && enabled(watch.recording_revision) {
        if let Err(error) = watch.report(position, playing, final_report) {
            eprintln!("streamtube: YouTube watch progress failed: {error}");
        }
    }
}

fn worker(rx: mpsc::Receiver<Event>) {
    let mut active: Option<ActivePlayback> = None;
    while let Ok(event) = rx.recv() {
        match event {
            Event::Start(start) => {
                let watch = begin(&start);
                active = Some(ActivePlayback {
                    start,
                    watch,
                    last_start_attempt: Instant::now(),
                });
            }
            Event::Progress {
                serial,
                position,
                playing,
                force,
            } => {
                let Some(playback) = active.as_mut().filter(|p| p.start.serial == serial) else {
                    continue;
                };
                if playback.watch.is_none()
                    && playback.last_start_attempt.elapsed() >= Duration::from_secs(10)
                {
                    playback.watch = begin(&playback.start);
                    playback.last_start_attempt = Instant::now();
                }
                if let Some(watch) = &mut playback.watch {
                    if force || watch.due() || playing != watch.playing {
                        report(watch, position, playing, false);
                    }
                }
            }
            Event::Seek { serial, position } => {
                if let Some(playback) = active.as_mut().filter(|p| p.start.serial == serial) {
                    playback.start.position = position;
                    if let Some(watch) = &mut playback.watch {
                        watch.last_cmt = position;
                    }
                }
            }
            Event::Stop { serial, position } => {
                if active.as_ref().is_some_and(|p| p.start.serial == serial) {
                    if let Some(mut watch) = active.take().and_then(|p| p.watch) {
                        report(&mut watch, position, false, true);
                    }
                }
            }
            Event::Abandon { serial } => {
                if active.as_ref().is_some_and(|p| p.start.serial == serial) {
                    active = None;
                }
            }
            Event::Flush(done) => {
                let _ = done.send(());
            }
        }
    }
}

fn begin(start: &Start) -> Option<Watch> {
    if !enabled(start.recording_revision) || start.account.if_current(|| ()).is_none() {
        return None;
    }
    match Watch::begin(start) {
        Ok(watch) => Some(watch),
        // The error can contain details from an authenticated request.
        Err(_) => {
            eprintln!("streamtube: could not start YouTube watch reporting");
            None
        }
    }
}

fn reporter() -> &'static mpsc::Sender<Event> {
    REPORTER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("watch-report".into())
            .spawn(move || worker(rx));
        tx
    })
}

pub fn pending_reports() -> Option<mpsc::Receiver<()>> {
    let (done, receipt) = mpsc::channel();
    REPORTER.get()?.send(Event::Flush(done)).ok()?;
    Some(receipt)
}

/// A single playback, tied to the account selected when it was opened.
pub struct PlaybackReport {
    account: Account,
    recording_revision: u64,
    video_id: String,
    serial: u64,
    started: bool,
}

impl PlaybackReport {
    pub fn new(video_id: String, serial: u64) -> Option<Self> {
        Some(Self {
            account: Account::current()?,
            recording_revision: crate::store::history_recording_revision()?,
            video_id,
            serial,
            started: false,
        })
    }

    pub fn progress(&self, position: f64, playing: bool, force: bool) {
        if self.started
            && enabled(self.recording_revision)
            && position.is_finite()
            && position >= 0.0
        {
            let _ = reporter().send(Event::Progress {
                serial: self.serial,
                position,
                playing,
                force,
            });
        }
    }

    pub fn seek(&self, position: f64) {
        if self.started
            && enabled(self.recording_revision)
            && position.is_finite()
            && position >= 0.0
        {
            let _ = reporter().send(Event::Seek {
                serial: self.serial,
                position,
            });
        }
    }

    pub fn start(&mut self, position: f64, len: f64) {
        if self.started
            || !enabled(self.recording_revision)
            || !position.is_finite()
            || position <= 0.0
        {
            return;
        }
        self.started = reporter()
            .send(Event::Start(Start {
                account: self.account.clone(),
                recording_revision: self.recording_revision,
                id: self.video_id.clone(),
                serial: self.serial,
                len,
                position,
            }))
            .is_ok();
    }

    /// Drops the report without a final ping, when the account already has a
    /// newer position from another device than this one would report.
    pub fn abandon(self) {
        if self.started {
            let _ = reporter().send(Event::Abandon {
                serial: self.serial,
            });
        }
    }

    pub fn finish(self, position: f64) {
        if self.started
            && enabled(self.recording_revision)
            && position.is_finite()
            && position >= 0.0
        {
            let _ = reporter().send(Event::Stop {
                serial: self.serial,
                position,
            });
        }
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;

    /// Reports a short watch of VID with the profile in STREAMTUBE_DATA_DIR, then
    /// checks the account's history page.
    #[test]
    #[ignore = "needs a signed-in profile in STREAMTUBE_DATA_DIR and network"]
    fn reports_to_the_account() {
        let id = std::env::var("VID").unwrap_or_else(|_| "aqz-KE-bpKQ".into());
        let account = Account::current().expect("a signed-in account");
        let start = Start {
            account: account.clone(),
            recording_revision: crate::store::history_recording_revision().unwrap_or(0),
            id: id.clone(),
            serial: 1,
            len: 600.0,
            position: 3.0,
        };
        let mut w = Watch::begin(&start).expect("playback ping");
        std::thread::sleep(Duration::from_secs(2));
        w.report(9.0, true, false).expect("watchtime ping");
        w.report(12.0, false, true).expect("final ping");
        let page = account.page(None).expect("history page");
        let first: Vec<_> = page
            .entries
            .iter()
            .take(3)
            .map(|e| (e.video.id.clone(), e.progress.map(|p| p.seconds)))
            .collect();
        println!("history head: {first:?}");
        assert!(
            page.entries.iter().take(3).any(|e| e.video.id == id),
            "watched video not at the top of history"
        );
    }
}

/// `https://youtube.com/...` or `https://<anything>.youtube.com/...`, nothing else.
fn is_youtube_https(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    // no user info or port tricks ("youtube.com@evil.example")
    !host.contains(['@', ':']) && (host == "youtube.com" || host.ends_with(".youtube.com"))
}

#[cfg(test)]
mod host_tests {
    use super::is_youtube_https;

    #[::core::prelude::v1::test]
    fn only_youtube() {
        assert!(is_youtube_https(
            "https://s.youtube.com/api/stats/playback?x=1"
        ));
        assert!(is_youtube_https(
            "https://www.youtube.com/api/stats/watchtime"
        ));
        assert!(!is_youtube_https("http://s.youtube.com/api"));
        assert!(!is_youtube_https("https://youtube.com.evil.example/api"));
        assert!(!is_youtube_https(
            "https://www.youtube.com@evil.example/api"
        ));
        assert!(!is_youtube_https("https://evilyoutube.com/api"));
    }
}
