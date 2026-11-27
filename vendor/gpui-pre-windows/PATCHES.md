# gpui-pre-windows 0.3.12, patched

Copy of [gpui-pre-windows 0.3.12](https://crates.io/crates/gpui-pre-windows)
(Zed's `gpui_windows` crate, Apache-2.0) with one fix in `src/direct_write.rs`:
right-to-left text (Arabic, Hebrew, ...) needs visual glyph ordering.

- `DrawGlyphRun`: DirectWrite returns a right-to-left run (odd `bidiLevel`) in
  logical order. The patch reverses it before assigning its left-to-right visual
  positions. This keeps Arabic and Hebrew inside their layout bounds instead of
  placing the full run offscreen to the left.
- Mixed paragraphs use DirectWrite's first-strong paragraph direction and
  native run origins. The physical layout origin stays on the left, and runs
  are delivered to GPUI in visual order.
- Tabs are placed where the layout's hit test puts them.
- `StringIndexConverter::advance_to_utf16_ix` can move backwards.
- Tests: `rtl_runs_are_laid_out_right_to_left` and
  `mixed_arabic_paragraph_keeps_brand_and_date_in_reading_order` are new; the
  caret checks in `tabs_preserve_native_advances_and_cursor` skip right-to-left
  strings, since they assumed the old left-to-right layout.

The Windows event handler also translates native touchscreen pans into pixel scroll events. It keeps the pan attached to the control where the gesture started.

Every change is marked with "Stream Tube patch". Drop this folder and the
`[patch.crates-io]` entry in `Cargo.toml` once upstream handles bidi text and touchscreen scrolling.
