# Stream Tube Desktop — Arabic RTL follow-up audit

**Date:** 27 November 2026
**Repository:** `ipapplus/Stream-Tube-Desktop` (personal fork)
**Branch state:** published from `main`; see [PUBLICATION-REPORT.md](PUBLICATION-REPORT.md)
**Build tested and installed:** SHA-256 `DD150993E9B27B1322ECF142DDC3A4184E964FDDB3CEB5219502C8CC3E809ED5`

## Executive summary

This follow-up fixed the remaining Arabic RTL/localization defects found after the first audit. The changes are shared-cause fixes: collapsed navigation is now an icon-only Arabic rail with complete accessible names and tooltips; player JSON-style menus and inspected popups follow interface direction while media controls stay intentionally LTR; all recognized numeric counts go through one formatter; and Watch metadata separates localized counts from untouched dates.

All five code-side gates passed after the final source changes. All six GUI checks A–F were executed against the final tested executable. The final release was backed up, installed, launched from the Start menu, and confirmed to have the same SHA-256 as the tested release. Verification used an isolated local test profile; no real user profile, settings file, or database is included in this publication.

The commit, push, tag, and release are recorded in the publication report.

## Scope and baseline

| Item | Before | After |
| --- | --- | --- |
| Release executable | Earlier local build | `DD150993E9B27B1322ECF142DDC3A4184E964FDDB3CEB5219502C8CC3E809ED5` |
| Install backup | Earlier local build | Kept outside the published tree |
| User settings | Not read for publication | No settings file, database, or user profile is published |
| GUI test profile | Isolated local test profile | Used only for tests; not published |
| Arabic catalog | 914 keys baseline | 929 keys; 15 added, 24 changed, 0 removed |

## Issue-by-issue findings

### 1. Collapsed Arabic navigation word order

| Stage | Detail |
| --- | --- |
| Evidence | Supplied before screenshots showed reordered/truncated Arabic labels in the collapsed rail. Final UIA exposed `r-shorts` as `الفيديوهات القصيرة`; the tooltip was `165x34` at `x=1356,y=232` and OCR recognized `الفيديوهات القصيرة`. |
| Finding | The collapsed rail was trying to render full Arabic labels in a narrow strip, which produced ambiguous word order. Expanded navigation was already correct and had to remain unchanged. |
| Path | `src/ui/app.rs` now renders the collapsed rail as icon-only, with the full translated string in `aria_label` and `Tooltip`. Shared direction helpers in `src/ui/mod.rs` keep expanded and collapsed branches consistent. |
| Verification | [collapsed-sidebar-arabic.png](screenshots/follow-up/collapsed-sidebar-arabic.png), [expanded-sidebar-arabic.png](screenshots/follow-up/expanded-sidebar-arabic.png), [shorts-navigation-tooltip.png](screenshots/follow-up/shorts-navigation-tooltip.png); GUI test A. |

English navigation was checked separately and remained left-to-right with the original item order.

### 2. Player menus and inspected popups

| Stage | Detail |
| --- | --- |
| Evidence | Supplied before screenshot `supplied-before-player-gear-menu.png` showed the gear menu inheriting page direction and mixing media-control geometry with interface labels. Final UIA showed Arabic gear rows bounded at `x=1171`, width `375`, `y=160..710`; English gear rows were at `x=639`, width `375`. |
| Finding | Interface menus such as Quality, Playback speed, Subtitles/CC, Aspect ratio, Equalizer, Loop, Stats, Screenshot save, and External playback needed direction-aware layout. The timeline, seek bar, playback buttons, volume, and fullscreen controls are deliberately LTR and must not mirror. |
| Path | `src/player/view.rs` uses shared direction helpers, logical padding, `text_start`/`text_end`, and direction-aware chevrons for gear and submenu rows. `src/ui/transcript_menu.rs` applies the same treatment to the transcript caption popup. Accessibility IDs were added without changing rendering: `c-settings`, `m-q`, `m-s`, `m-c`, `m-ar`, `m-eq`, `m-ao`, `m-amb`, `m-l`, `m-stats`, `m-shot`, `m-ext`, `menu-back`, `cap-off`, and `eq-*`. |
| Verification | [player-gear-menu-arabic.png](screenshots/follow-up/player-gear-menu-arabic.png), [player-caption-submenu-arabic.png](screenshots/follow-up/player-caption-submenu-arabic.png), [player-equalizer-submenu-arabic.png](screenshots/follow-up/player-equalizer-submenu-arabic.png), [english-gear-ltr.png](screenshots/follow-up/english-gear-ltr.png); GUI tests B and F. |

Intentional LTR player controls were confirmed by UIA bounds: `c-play` x587, `c-next` x642, `c-mute` x697, `c-shot` x1221, `c-settings` x1276, `c-pip` x1331, `c-theater` x1386, `c-wfs` x1441, `c-fs` x1496, all at y650.

### 3. Remaining user-facing labels

| Stage | Detail |
| --- | --- |
| Evidence | The final Arabic catalog contains previously missing labels such as `Flat`, `Bass boost`, `Voice`, `Treble boost`, and `Warm`; 24 existing strings were corrected. |
| Finding | Several player-menu and settings strings were either untranslated or had punctuation/spacing that broke mixed-direction reading. Technical and product identifiers (`libmpv`, `FFmpeg`, `yt-dlp`, file extensions, version tokens, URLs, paths) had to remain in their original form but be directionally isolated. |
| Path | `src/i18n/locales/ar.json` plus `src/i18n.rs` (`{0:ltr}`, `{0:rtl}`, `isolate_ltr`, `isolate_ascii`) and the call sites listed in the exact-change tables below. |
| Verification | Catalog validation and unit tests; GUI tests B, D, E, and F. |

### 4. Numeric localization

| Stage | Detail |
| --- | --- |
| Evidence | Unit tests cover six CLDR Arabic plural categories and compact forms such as `1 ألف`, `1.1 ألف`, `3 آلاف`, `10 ألف`, `1 مليون`, `3 ملايين`, `10 ملايين`, `1 مليار`, `3 مليار`, and `1 ترليون`. In an isolated profile, history rows with numeric raw labels produced OCR text `23.6 مليون مشاهدة`, `19.9 مليون مشاهدة`, and `5.8 ملايين مشاهدة`. Watch vote counts rendered as `111 ألف` and `18.7 ألف`. |
| Finding | Counts were previously assembled per page from preformatted YouTube strings, producing mixed English/Arabic labels and inconsistent plurals. |
| Path | New shared module `src/metrics.rs` derives values from raw numeric counts, uses one CLDR-derived Arabic formatter, keeps Western digits, and provides display variants with LRI/PDI and selectable variants without isolates. Integrations cover views, likes, dislikes, comments, replies, video/playlist/download/saved/history counts, subscribers, and settings stats through `src/ui/cards.rs`, `src/ui/pages/watch.rs`, `src/ui/pages/shorts.rs`, `src/ui/pages/channel.rs`, `src/ui/pages/playlist.rs`, and `src/ui/pages/settings.rs`. |
| Safety | Only known numeric labels are converted. Semantic labels such as `7 episodes` and already-localized values are left untouched. Cached history and copied values remain raw. |
| Verification | [history-arabic-compact-counts.png](screenshots/follow-up/history-arabic-compact-counts.png), [watch-arabic-date-and-count.png](screenshots/follow-up/watch-arabic-date-and-count.png); GUI tests C and D. |

For GUI test C, the isolated test database was temporarily given numeric English `views_text` values to exercise the formatter. The database was restored from its pre-seed copy after the screenshot. No real-profile data was changed.

### 5. Dates and Watch metadata

| Stage | Detail |
| --- | --- |
| Evidence | The final Watch screenshot OCR shows the numeric date `10/11/2014` and localized vote counts in the same metadata region without the date being absorbed into the Arabic count. |
| Finding | Combining a numeric date and an Arabic count in one text node allowed bidi reordering and made it impossible to isolate the two values independently. |
| Path | `src/ui/pages/watch.rs` now emits separate `watch-date-*` and `watch-views-*` elements in RTL, with display-only `i18n::isolate_ltr(&w.date_text)` around the date. Date parsing, stored strings, and copy semantics are unchanged. LTR keeps the original combined selectable text. |
| Verification | [watch-arabic-date-and-count.png](screenshots/follow-up/watch-arabic-date-and-count.png); GUI test D. |

When the API supplies an already-localized Arabic views string, the formatter intentionally leaves it unchanged instead of guessing; the vote counts still demonstrate the shared formatter in the same screenshot.

### 6. Mixed-direction text

| Stage | Detail |
| --- | --- |
| Evidence | [arabic-mixed-text.png](screenshots/follow-up/arabic-mixed-text.png) shows Arabic navigation and controls alongside English titles, URLs, channel names, dates, and Western digits. |
| Finding | Mixed strings need isolation only for display. Raw values and copyable text must not gain invisible bidi control characters. |
| Path | Existing `{0:ltr}` / `{0:rtl}` mechanisms were reused and extended with `i18n::isolate_ltr` and `i18n::isolate_ascii`. The exact catalog changes are listed below. |
| Verification | GUI test E and the `display_numbers_are_isolated_but_selectable_numbers_are_not` unit test. |

## Exact translation changes

Catalog comparison against `original-ar-from-baseline.json`: `old=914 new=929 added=15 removed=0 changed=24`. The machine-readable diff is retained outside the published tree.

### Added keys

| Key | Arabic value |
| --- | --- |
| `Bass boost` | `تعزيز الجهير` |
| `Expand description` | `توسيع الوصف` |
| `Flat` | `مسطّح` |
| `Navigation menu` | `قائمة التنقل` |
| `No watched videos in {0}` | `لم تشاهد أي فيديو في «{0}» بعد` |
| `Remove watched videos?` | `إزالة الفيديوهات التي شاهدتها؟` |
| `Removed {0} watched video` | `أُزيل الفيديو الذي شاهدته (عدد الفيديوهات: {0})` |
| `Removed {0} watched videos` | `أُزيلت الفيديوهات التي شاهدتها (عدد الفيديوهات: {0})` |
| `Search` | `بحث` |
| `Treble boost` | `تعزيز الطبقات العالية` |
| `Voice` | `الصوت` |
| `Warm` | `دافئ` |
| `browser` | `المتصفح` |
| `{0} watched video will be removed from {1}.` | `سيُزال الفيديو الذي شاهدته من «{1}» (عدد الفيديوهات: {0}).` |
| `{0} watched videos will be removed from {1}.` | `ستُزال الفيديوهات التي شاهدتها من «{1}» (عدد الفيديوهات: {0}).` |

### Changed keys

| Key | Before | After |
| --- | --- | --- |
| `Buffer health` | `مخزون التشغيل المؤقت` | `مدة التشغيل المخزّنة مؤقتًا` |
| `Copy link {0}` | `نسخ الرابط {0}` | `نسخ الرابط {0:ltr}` |
| `Copy link {0}…` | `نسخ الرابط {0}…` | `نسخ الرابط {0:ltr}…` |
| `Downloading… {0}%` | `جارٍ التنزيل… {0}%` | `جارٍ التنزيل… {0:ltr}%` |
| `Exported to {0}` | `صُدّر إلى {0}` | `صُدّر إلى {0:ltr}` |
| `Get FFmpeg ({0})` | `تنزيل FFmpeg ‏({0})` | `تنزيل FFmpeg ({0})` |
| `Get yt-dlp ({0})` | `تنزيل yt-dlp ‏({0})` | `تنزيل yt-dlp ({0})` |
| `Heat line above the seek bar` | `يوضح المقاطع الأكثر إعادة فوق شريط التقدم` | `يظهر المخطط فوق شريط التقدم.` |
| `Most replayed graph` | `مخطط المقاطع الأكثر إعادة` | `مخطط المقاطع الأكثر إعادة مشاهدة` |
| `On another device, go to {0} and enter the code` | `افتح {0} على جهاز آخر وأدخل الرمز` | `افتح {0:ltr} على جهاز آخر وأدخل الرمز` |
| `Saved to {0}` | `حُفظ في {0}` | `حُفظ في {0:ltr}` |
| `Saving to {0}. Engine {1}` | `الحفظ في {0} باستخدام {1}` | `الحفظ في {0:ltr} باستخدام {1}` |
| `Screenshot saved to {0}` | `حُفظت لقطة الشاشة في {0}` | `حُفظت لقطة الشاشة في {0:ltr}` |
| `Source: {0}, libmpv` | `المصدر: {0}، المشغّل: libmpv` | `المصدر: {0:ltr}، المشغّل: libmpv` |
| `Speed {0}x` | `السرعة {0}×` | `السرعة: {0:ltr}×` |
| `Stream Tube {0} is the latest version` | `لديك أحدث إصدار من Stream Tube ‏({0})` | `لديك أحدث إصدار من Stream Tube ({0:ltr})` |
| `Subtitles (.srt)` | `الترجمة (‎.srt‎)` | `الترجمة (.srt)` |
| `Subtitles saved · {0}` | `حُفظت الترجمة · {0}` | `حُفظت الترجمة · {0:ltr}` |
| `Transcript (.txt)` | `النص (‎.txt‎)` | `النص المكتوب (.txt)` |
| `Transcript saved · {0}` | `حُفظ النص · {0}` | `حُفظ النص المكتوب · {0:ltr}` |
| `Version {0}` | `الإصدار {0}` | `الإصدار {0:ltr}` |
| `{0} is available` | `الإصدار {0} متاح` | `الإصدار {0:ltr} متاح` |
| `{0} s` | `{0} ث` | `{0:ltr} ث` |
| `{0}, format {1}` | `{0}، الصيغة: {1}` | `{0}، الصيغة: {1:ltr}` |

## Verification outputs

All commands were run from the repository root after the last source edit.

### Formatting

```text
$ cargo fmt --check
(no output; exit code 0)
```

### Release build

A full build after the final source edits finished in `4m 07s`. The last locked release build was a no-op and produced:

```text
warning: `streamtube` (bin "streamtube") generated 20 warnings (run `cargo fix --bin "streamtube" -p streamtube` to
apply 1 suggestion)
    Finished `release` profile [optimized] target(s) in 0.42s
```

Exit code was `0`. The 20 warnings are pre-existing and did not change in this follow-up.

### Application tests

```text
$ cargo test --release --locked -q
running 155 tests
test result: ok. 127 passed; 0 failed; 28 ignored; 0 measured; 0 filtered out; finished in 1.06s
```

Exit code was `0`.

### Windows GPUI pre-window tests

```text
$ cargo test --release --locked -q -p gpui-pre-windows
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Exit code was `0`.

### Whitespace check

```text
$ git -c core.safecrlf=false diff --check
(no output; exit code 0)
```

### Hashes

```text
target/release/streamtube.exe  DD150993E9B27B1322ECF142DDC3A4184E964FDDB3CEB5219502C8CC3E809ED5
```

## GUI tests A–F

| Test | Scope | Status | Evidence |
| --- | --- | --- | --- |
| A | Arabic navigation and collapsed rail | PASSED | UIA `r-shorts` = `الفيديوهات القصيرة`; tooltip `165x34` at `x=1356,y=232`; OCR recognized the full label; expanded sidebar unchanged. [collapsed-sidebar-arabic.png](screenshots/follow-up/collapsed-sidebar-arabic.png), [shorts-navigation-tooltip.png](screenshots/follow-up/shorts-navigation-tooltip.png) |
| B | Player gear, caption, and equalizer menus | PASSED | Arabic rows localized and bounded `x=1171`, width `375`, `y=160..710`; caption IDs `menu-back` / `cap-off`; equalizer IDs `eq-flat`, `eq-bass`, `eq-voice`, `eq-treble`, `eq-warm`; controls stayed LTR. [player-gear-menu-arabic.png](screenshots/follow-up/player-gear-menu-arabic.png), [player-caption-submenu-arabic.png](screenshots/follow-up/player-caption-submenu-arabic.png), [player-equalizer-submenu-arabic.png](screenshots/follow-up/player-equalizer-submenu-arabic.png) |
| C | Arabic compact counts in history | PASSED | Isolated numeric labels rendered as `23.6 مليون مشاهدة`, `19.9 مليون مشاهدة`, and `5.8 ملايين مشاهدة`; unit tests cover compact forms and six plural categories. [history-arabic-compact-counts.png](screenshots/follow-up/history-arabic-compact-counts.png) |
| D | Watch localized counts and date metadata | PASSED | Watch metadata OCR shows `10/11/2014`, `111 ألف`, and `18.7 ألف`; source date and count are separate elements with display-only isolation. [watch-arabic-date-and-count.png](screenshots/follow-up/watch-arabic-date-and-count.png) |
| E | Mixed Arabic and LTR text | PASSED | Arabic UI with untouched English titles, URL, channel names, dates, and Western digits; isolation covered by unit tests. [arabic-mixed-text.png](screenshots/follow-up/arabic-mixed-text.png) |
| F | English LTR regression | PASSED | English UIA names `Home`, `Shorts`, `Subscriptions`, `Settings`; gear rows x639 versus Arabic x1171; English player labels intact. [english-ltr-regression.png](screenshots/follow-up/english-ltr-regression.png), [english-gear-ltr.png](screenshots/follow-up/english-gear-ltr.png) |

## Screenshots

All follow-up screenshots are in `screenshots/follow-up` relative to this report (`docs\rtl-audit\screenshots\follow-up` in the repository):

- [collapsed-sidebar-arabic.png](screenshots/follow-up/collapsed-sidebar-arabic.png)
- [expanded-sidebar-arabic.png](screenshots/follow-up/expanded-sidebar-arabic.png)
- [shorts-navigation-tooltip.png](screenshots/follow-up/shorts-navigation-tooltip.png)
- [player-gear-menu-arabic.png](screenshots/follow-up/player-gear-menu-arabic.png)
- [player-caption-submenu-arabic.png](screenshots/follow-up/player-caption-submenu-arabic.png)
- [player-equalizer-submenu-arabic.png](screenshots/follow-up/player-equalizer-submenu-arabic.png)
- [history-arabic-compact-counts.png](screenshots/follow-up/history-arabic-compact-counts.png)
- [watch-arabic-date-and-count.png](screenshots/follow-up/watch-arabic-date-and-count.png)
- [arabic-mixed-text.png](screenshots/follow-up/arabic-mixed-text.png)
- [english-ltr-regression.png](screenshots/follow-up/english-ltr-regression.png)
- [english-gear-ltr.png](screenshots/follow-up/english-gear-ltr.png)
- The screenshot from the real installed profile is retained locally only.

## Installation

1. Confirmed that no Stream Tube process was running before replacement.
2. Backed up the previous local executable outside the source tree.
3. Copied the tested release build to the local installation path.
4. Verified that the installed executable hash matched the release build.
5. Confirmed that the local settings file and Arabic language preference remained unchanged; no settings content or hash is published.
6. Launched through the Start menu entry point and confirmed the Arabic UI. The screenshot tied to the real installed profile is retained locally only.

The Start menu shell surface itself did not open through the automation helper, so the shortcut was invoked directly. This exercised the same Start menu entry point.

## Limitations

- Direct visual inspection of PNG pixels was unavailable in this model. Verification used UIA trees, OCR, bounds, and automated tests; a vision-capable reviewer can still make the final aesthetic judgment.
- OCR at small font sizes misread some Arabic words; for example `ملايين` was read as `ملان` in one history row. The formatter itself is covered by unit tests.
- The Watch API sometimes returns an already-localized Arabic views string. By design, the shared formatter leaves such values unchanged rather than guessing, so the Watch screenshot demonstrates compact vote counts and the separated date, not necessarily a compact views label.
- DPI/resolution matrix checks from the earlier audit were not re-run in this follow-up. No new dependency was introduced and `--locked` builds passed.
- No pixel-level automated diff was added; screenshots are retained as manual evidence.
- The source changes are recorded in the publication report.

## Git status

The source commit and final working-tree state are recorded in [PUBLICATION-REPORT.md](PUBLICATION-REPORT.md).
