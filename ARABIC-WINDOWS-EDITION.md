# Arabic Windows Edition

Stream Tube Desktop **0.7.5-alpha** Arabic Windows Edition is a **Windows 10/11 x64** build from the `ipapplus` fork of Stream Tube Desktop. It adds an Arabic (`ar`) interface and right-to-left (RTL) layout to the existing Rust/GPUI application. This is a personal fork release; the original project distributes its own builds separately.

## Arabic localization

Arabic is registered as an application language and selects RTL layout when it is active. The Arabic catalog contains **929 keys**. Against the first audit baseline, the follow-up added **15** keys, corrected **24**, and removed **none**. Changes cover navigation, player presets, settings descriptions, and feedback messages. The catalog keeps product names such as Stream Tube, FFmpeg, yt-dlp, and libmpv in their original form. Titles, account names, comments, and other user-generated text retain their original language.

Regression tests validate Arabic catalog parsing, placeholder compatibility, directional isolation, and count formatting. Untranslated interface strings fall back to the source text.

## Right-to-left layout

The application resolves layout direction from the interface language and shares helpers for row order, logical spacing, leading and trailing alignment, and submenu chevrons. The expanded Arabic sidebar is on the right with full labels. The collapsed rail uses icons, with full Arabic tooltips and Windows accessibility names.

The same direction rules apply to settings navigation and controls, search, video cards, channel and playlist layouts, history and library, Shorts, comments and replies, and inspected popups. Settings descriptions wrap as Arabic prose, and groups that can outgrow a row can wrap without changing their logical order. The dark theme and red branding are retained.

## Bidirectional text

Shared application code uses Unicode isolates for display substitutions such as URLs, file paths, versions, and other Latin tokens within Arabic sentences. Dynamic placeholders can request an explicit direction or let the value determine its own direction. Selectable variants omit display-only controls. Watch renders the RTL date and recognized view count in separate elements, keeping the source date value and date semantics.

On Windows, the vendored GPUI DirectWrite backend sets paragraph reading direction from the first strong character. It uses DirectWrite's native glyph runs and positions so Arabic shaping and mixed-direction text follow the Windows layout. This is distinct from the application's layout helpers. The timeline, seek bar, playback buttons, volume, and other media controls retain their intentional left-to-right behavior.

## Statistics and number formatting

A shared formatter handles recognized view, subscriber, like, dislike, comment, reply, video, and playlist counts. It uses CLDR-derived Arabic compact units at thousand, million, and billion scales and accounts for the zero, one, two, few, many, and other cardinal categories. Western digits remain the default; display-only isolation keeps decimal tokens in order inside Arabic text.

The formatter does not guess the meaning of strings such as episode labels. Some prelocalized provider count strings are not recognized and may be omitted from the compact metadata row. Stored provider and history data are preserved.

## Player menus

The player gear menu and its quality, playback-speed, caption, aspect-ratio, equalizer, and audio submenus use direction-aware rows and submenu indicators. The transcript caption picker follows the same layout rules. The equalizer presets, screenshot actions, and external-player options use localized interface labels; technical identifiers retain their original spelling. The changes apply to presentation and accessibility, while existing playback and media-control actions retain their semantics.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo build --release --locked` | Passed; 20 compiler warnings |
| `cargo test --release --locked -q` | 127 passed, 0 failed, 28 ignored |
| `cargo test --release --locked -q -p gpui-pre-windows` | 22 passed, 0 failed |
| `git -c core.safecrlf=false diff --check` | Passed |
| Follow-up GUI checks A–F | Passed as recorded in the audit, with scope and evidence in the [follow-up report](docs/rtl-audit/public/follow-up-report.md) |

The [initial audit](docs/rtl-audit/public/report.md) checked Settings and Watch at **1366×768** and **1920×1080**, each at **100%, 125%, and 150%** Windows display scaling. The full display matrix was not repeated for the follow-up.

The locked release build produced `streamtube.exe` with SHA-256 `DD150993E9B27B1322ECF142DDC3A4184E964FDDB3CEB5219502C8CC3E809ED5`.

## Installation and limitations

Extract the Windows runtime ZIP and run `streamtube.exe` from the extracted folder. Keep `libmpv-2.dll` beside the executable and the Vulkan fallback loader in its `vulkan` subdirectory. The executable-only download requires the same runtime files. The app's existing in-app updater is tied to the original project; obtain Arabic edition updates from the fork's release page.

Manual clipboard testing of raw values and paths, account-only operations, multi-monitor per-monitor DPI, and non-Windows runtime behavior were outside the audit. The runtime ZIP does not create a separate data profile by default.

## Screenshots

These are sanitized copies from the published audit; screenshots tied to a real installed profile and local paths are retained outside the repository.

- [Expanded Arabic sidebar](docs/rtl-audit/public/screenshots/follow-up/expanded-sidebar-arabic.png), [collapsed rail](docs/rtl-audit/public/screenshots/follow-up/collapsed-sidebar-arabic.png), and [Shorts tooltip](docs/rtl-audit/public/screenshots/follow-up/shorts-navigation-tooltip.png)
- [Arabic settings](docs/rtl-audit/public/screenshots/after-settings.png) and [mixed-direction text](docs/rtl-audit/public/screenshots/follow-up/arabic-mixed-text.png)
- [Player gear menu](docs/rtl-audit/public/screenshots/follow-up/player-gear-menu-arabic.png), [captions](docs/rtl-audit/public/screenshots/follow-up/player-caption-submenu-arabic.png), and [equalizer](docs/rtl-audit/public/screenshots/follow-up/player-equalizer-submenu-arabic.png)
- [History compact counts](docs/rtl-audit/public/screenshots/follow-up/history-arabic-compact-counts.png) and [Watch dates and counts](docs/rtl-audit/public/screenshots/follow-up/watch-arabic-date-and-count.png)
- [English LTR regression](docs/rtl-audit/public/screenshots/follow-up/english-ltr-regression.png) and [English player menu](docs/rtl-audit/public/screenshots/follow-up/english-gear-ltr.png)
