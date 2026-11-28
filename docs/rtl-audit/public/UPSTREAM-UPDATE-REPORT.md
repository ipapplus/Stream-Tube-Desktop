# Stream Tube Desktop 1.0.0 Arabic Windows integration

## Source integration

| Item | Verified value |
| --- | --- |
| Starting fork and local `main` | `0d93db3f1fb9aa128fba49ce4f9da380da6b3d9c` |
| Previous installed version | `0.7.5-alpha` |
| Previous release tag | `v0.7.5-alpha-arabic.1` |
| Common ancestor | `ea472fa78d22025637f4ea3cd9bafe3df10ef797` |
| Upstream version and head | `1.0.0`, `7da65e55ae5a6d4c56b1df4df5e0ec57c65ba69e` |
| Merge | `c5c805e5f039e17a1a2a35749dcfcaebbb828317` |
| Arabic source commit | `49c378021d1757de0141dfe9bc43483f546631c2` |
| Changes from starting fork | 30 upstream commits integrated, 63 files changed |
| Previous fork commits retained | `821c104`, `4de7be6`, and `0d93db3` |

The merge had 21 path conflicts. A three-way comparison after normalizing Rust
formatting resolved formatting overlaps. The behavior conflicts combined the
redesigned player menu with Arabic row direction and bidi handling, retained
clickable channel names and localized metadata in playlists, and kept the new
YouTube save and sign-in behavior with RTL layout. Watch and Shorts retained
their localized count paths. Cargo retained the Windows GPUI patch and added
the new caption dependencies and vendored Linux input decoder.

The merge includes the redesigned player, Auto quality, captions, Watch Later
and YouTube playlist membership changes, channel sorting, music mixes and Now
Playing, HLS recovery, signed-in playback and download paths, safer database
startup, and Linux desktop media integration. The [upstream release notes](https://github.com/Avaneesh-Inamdar/Stream-Tube-Desktop/releases/tag/v1.0.0)
describe the original release.

## Arabic interface

The catalog contains 962 Arabic entries, with 33 additions and zero changed or
removed entries relative to the starting fork. Additions cover music mode, Now
Playing, captions, channel sorting, automatic quality, settings, playlist
actions, and accessible media-control names. Across the other ten catalogs,
30 duplicate definitions were removed while preserving their effective
translations.

The music bar and queue, channel sort row, save dialog, and player submenus
use direction-aware layout. Playback order, seek, and volume run left to
right. DirectWrite retains its native RTL and mixed-paragraph implementation.
The shared metrics formatter retains its Arabic plural categories and Western
digits. Storage and copyable values retain their source text.

The updater uses the [fork's releases](https://github.com/ipapplus/Stream-Tube-Desktop/releases).
On Windows it accepts the release's setup filename and matching fork asset
URL, requires a valid SHA-256 digest in GitHub metadata, and hashes the
download before running it. Missing or malformed assets leave the release
page available. Tests reject the original upstream and the former migration
destination as installer sources.

## Local checks

| Check | Result |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo build --release --locked` | Passed with 17 dead-code warnings retained from existing declarations |
| `cargo test --release --locked -q` | 132 passed, 0 failed, 29 ignored |
| `cargo test --release --locked -q -p gpui-pre-windows` | 22 passed, 0 failed, 0 ignored |
| `git diff --check` and staged patch review | Passed |
| JSON, duplicate keys, missing source keys, placeholder compatibility | Passed for Arabic and all ten other embedded catalogs |

The local executable SHA-256 is
`D491A44E35D8A9E72FB14FEC12EF151DC44A39C9680B9C5318A09C807F3BEE61`.
The test suites include Arabic direction, bidi isolation, pluralization,
DirectWrite shaping and mixed paragraphs, upstream regression cases, and
updater release selection. Separate live tests passed for verified yt-dlp
retrieval and resuming an HLS segment after pausing the reader. The first
children’s video download completed, but its reported filename omitted a
Unicode character. Reading yt-dlp’s JSON-encoded output path addresses the
observed mismatch. The same live download passed after the fix. Each of\nthe three separately run live tests passed once. A startup smoke test of the\nfinal executable displayed the Arabic home page and sidebar.

## GUI checks

The first successful Arabic candidate was run with a separate unsigned-in
profile. Screenshots and Windows UI Automation corroborated Arabic navigation,
right-aligned expanded and collapsed sidebars, Arabic settings, and the English
LTR layout. Searching “theater” in English and “السينما” in Arabic displayed
only the matched Player rows. The public Watch test rendered mixed Arabic and English
metadata, Western digits and localized comment counts, and loaded replies.
Video playback, pause, seek, mute, fullscreen entry and exit, the updated gear
menu, Auto quality selection, and the caption picker were exercised. An
English subtitle track was selected and displayed with the Arabic interface.
The Music feed loaded, a public track played, and Now Playing rendered cover
art, related content, and the direction-aware controls. The bar stayed visible
while navigating to Settings and Home. Changing the sort to Oldest refreshed
the video grid for a public channel. The About page showed 1.0.0 and the updater
completed its check. Both window-fill context menu labels were confirmed in
Arabic. The checks used a 1920×1200 display at
125% scaling (120 logical DPI), including windowed and fullscreen views.

## GitHub release

The first push was rejected because the CLI credential lacked the `workflow`
scope. After authorization the public fork's `main` pointed to
`49c378021d1757de0141dfe9bc43483f546631c2`. The annotated `v1.0.0` tag resolves
to that commit. GitHub had no automatic run for the tag push, so the unchanged
upstream Release workflow was dispatched on `v1.0.0`.

The [workflow run](https://github.com/ipapplus/Stream-Tube-Desktop/actions/runs/46097299328)
completed successfully. The tag, Windows, Linux, and publish jobs all passed.
Windows CI reported 132 passing tests, zero failures, and 29 ignored. It
verified the pinned libmpv archive and confirmed that libmpv loads with the
bundled Vulkan loader. Linux packaged the archive, AppImage, Debian package,
and RPM, and passed the Debian installation check.

[Stream Tube Desktop 1.0.0 — Arabic Windows Edition](https://github.com/ipapplus/Stream-Tube-Desktop/releases/tag/v1.0.0) is published
as the fork's latest release with an Arabic description. All seven assets were
downloaded and verified before publication and fetched again after publication.
Every SHA-256 matched the GitHub metadata, and the six package hashes matched
`SHA256SUMS.txt`. All seven build attestations verified against the fork's
Release workflow, the tag ref, the tagged source digest, and GitHub-hosted
runners. The Windows ZIP contained the executable, libmpv, the Vulkan loader,
the expected documentation, and the portable-update marker. The portable
build uses the shared data profile by default.

| Verified asset | SHA-256 |
| --- | --- |
| [SHA256SUMS.txt](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/SHA256SUMS.txt) | `4b3456e246012642485626f651976439359eaccf371948f1285277d7acaaaef1` |
| [StreamTube-1.0.0-linux-x86_64.tar.gz](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/StreamTube-1.0.0-linux-x86_64.tar.gz) | `8e7c732289bf26af8f10b8ba9a1bb98bb7a51d625a337a73f06ae12d3366138e` |
| [StreamTube-1.0.0-setup.exe](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/StreamTube-1.0.0-setup.exe) | `c96ea661056846570bac4b8db798f9d864d28c4303b00967e232a9254bd86a71` |
| [StreamTube-1.0.0-windows-x64.zip](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/StreamTube-1.0.0-windows-x64.zip) | `4d10549d550e705b97b91e5bda14c6f780607f63a91ac0036e5dceb17685a621` |
| [StreamTube-1.0.0-x86_64.AppImage](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/StreamTube-1.0.0-x86_64.AppImage) | `164484266f7e2faaf3c94131303416923c15fd306e995d4132c55b60587add71` |
| [streamtube-1.0.0.x86_64.rpm](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/streamtube-1.0.0.x86_64.rpm) | `e8ff6df3fcf765639b022e1f9c803ab4dc022fd7dfc235d574531ac37a180be7` |
| [streamtube_1.0.0_amd64.deb](https://github.com/ipapplus/Stream-Tube-Desktop/releases/download/v1.0.0/streamtube_1.0.0_amd64.deb) | `f756bd5d6cf20875dde005710449afb21e8b4e89ddf8d643c58a610b5665eb89` |

## Installed Windows build and profile checks

The registered per-user installation uses the expected Inno Setup identity and
the original location, `%LOCALAPPDATA%/Programs/Stream Tube`. The original
0.7.5-alpha executable and the existing 1.0.0 installation observed on
resumption were backed up locally. The second observed executable initially
contained the upstream release URL. It had already changed by the time the
verified installer could be run. The registered installation then reported
1.0.0, and its executable, libmpv, and Vulkan loader all matched the attested
Windows ZIP. No redundant installer invocation was performed. Both existing
shortcuts still resolve to the registered executable.

The installed executable SHA-256 is
`d4667b0e45872a3f84bb2b854174f4c1afc7950761fcb8868e6a20de20438173`.
The installed binary contains the fork's release URL. Updater tests cover
acceptance of fork assets and rejection of assets from the original project.

The installed program started with the existing Arabic profile. SQLite
integrity checks passed on local copies. A before-and-after startup comparison
found no changes to the settings fields or the playlist, playlist-item,
subscription, profile, and saved-playlist records. History and its sync table
gained entries during startup. The earlier and current history snapshots differ,
so the report does not certify a byte-for-byte history match across the
intervening installation. Settings and application databases were never
restored, reset, or copied into the worktree.

In an isolated profile, the same installed executable displayed the Arabic
sidebar on the right and loaded the Music feed. A public video played, paused
and resumed, the localized player menu opened, and Arabic captions rendered.
The test video was opened only in the isolated profile.

## Final source and audit state

The release tag remains on the Arabic source commit. The two report copies are
the only changes in the follow-up documentation commit on the fork's `main`.
The checkout retains the original untracked audit material. All 53 recorded
fingerprints of that local audit material matched their starting values. Only
the fork received pushes. The original project was accessed read-only. Its
branch advanced independently after the integration. No upstream PR or issue
was created and no force push was used. No build outputs, profile data, or
credentials were committed.

## Limitations

The previous Windows display scaling matrix, account-only flows, and Linux GUI
behavior have yet to be verified for this release. The previous publication
report and screenshots remain available as historical records.
