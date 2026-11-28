# Arabic Windows Edition

Stream Tube Desktop **1.0.0** Arabic Windows Edition is a **Windows 10/11 x64** build from the `ipapplus` fork of Stream Tube Desktop. It integrates upstream Stream Tube 1.0.0 with an Arabic (`ar`) interface and right-to-left (RTL) layout.

## Arabic localization

Arabic is registered as an application language and selects RTL layout when it is active. The Arabic catalog contains **962 keys**, including 33 additions for the 1.0.0 integration. The additions cover music, captions, settings, channel sorting, playlist actions, and player accessibility. The catalog keeps product names such as Stream Tube, FFmpeg, yt-dlp, and libmpv in their original form. Titles, account names, comments, and other user-generated text retain their original language.

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

## Release and installation

The [1.0.0 Arabic Windows release](https://github.com/ipapplus/Stream-Tube-Desktop/releases/tag/v1.0.0)
provides a per-user x64 installer and a ZIP that runs after extraction. Keep
`libmpv-2.dll` beside `streamtube.exe` and the Vulkan fallback loader in the
`vulkan` subdirectory. The ZIP has a marker for updating an extracted copy.
The app uses the shared profile by default. Set `STREAMTUBE_DATA_DIR` before
launch if you need a separate profile.

The updater selects the fork's Windows setup asset and checks its SHA-256
before running it. The [previous publication report](PUBLICATION-REPORT.md)
and [display matrix](docs/rtl-audit/public/report.md) document 0.7.5-alpha.

## Screenshots

These are sanitized copies from the published audit; screenshots tied to a real installed profile and local paths are retained outside the repository.

- [Expanded Arabic sidebar](docs/rtl-audit/public/screenshots/follow-up/expanded-sidebar-arabic.png), [collapsed rail](docs/rtl-audit/public/screenshots/follow-up/collapsed-sidebar-arabic.png), and [Shorts tooltip](docs/rtl-audit/public/screenshots/follow-up/shorts-navigation-tooltip.png)
- [Arabic settings](docs/rtl-audit/public/screenshots/after-settings.png) and [mixed-direction text](docs/rtl-audit/public/screenshots/follow-up/arabic-mixed-text.png)
- [Player gear menu](docs/rtl-audit/public/screenshots/follow-up/player-gear-menu-arabic.png), [captions](docs/rtl-audit/public/screenshots/follow-up/player-caption-submenu-arabic.png), and [equalizer](docs/rtl-audit/public/screenshots/follow-up/player-equalizer-submenu-arabic.png)
- [History compact counts](docs/rtl-audit/public/screenshots/follow-up/history-arabic-compact-counts.png) and [Watch dates and counts](docs/rtl-audit/public/screenshots/follow-up/watch-arabic-date-and-count.png)
- [English LTR regression](docs/rtl-audit/public/screenshots/follow-up/english-ltr-regression.png) and [English player menu](docs/rtl-audit/public/screenshots/follow-up/english-gear-ltr.png)
