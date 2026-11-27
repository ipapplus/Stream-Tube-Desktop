//! Locale-aware count formatting from numeric values.
//!
//! The application deliberately keeps YouTube's original display strings in
//! the model so copied values and cached history stay byte-for-byte intact.
//! This module converts a recognised numeric display string to a number and
//! renders it with CLDR-derived Arabic compact forms and plural categories.
//! Western digits are preserved; there is no implicit Arabic-Indic conversion.

use crate::i18n::AppLanguage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricKind {
    Views,
    Likes,
    Dislikes,
    Comments,
    Replies,
    Videos,
    PlaylistVideos,
    Downloads,
    SavedVideos,
    HistoryEntries,
    Subscribers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// CLDR Arabic cardinal categories: zero, one, two, few, many, other.
fn category(n: u64) -> Category {
    if n == 0 {
        Category::Zero
    } else if n == 1 {
        Category::One
    } else if n == 2 {
        Category::Two
    } else if (3..=10).contains(&(n % 100)) {
        Category::Few
    } else if (11..=99).contains(&(n % 100)) {
        Category::Many
    } else {
        Category::Other
    }
}

fn compact_decimal(value: f64) -> String {
    if value >= 100.0 {
        format!("{value:.0}")
    } else {
        let text = format!("{value:.1}");
        let text = text.trim_end_matches('0');
        let text = text.trim_end_matches('.');
        text.to_string()
    }
}

fn isolate_ltr(value: &str) -> String {
    format!("\u{2066}{value}\u{2069}")
}

/// Arabic compact unit words from CLDR long forms. The decimal token remains
/// Western; the unit form follows the compact pattern for its scale rather than
/// the ordinary cardinal rules (CLDR uses `10 ألف` but `10 ملايين`).
fn compact_unit(value: f64) -> (String, &'static str) {
    let (divisor, word) = if value >= 1e12 {
        (1e12, "ترليون")
    } else if value >= 1e9 {
        (1e9, "مليار")
    } else if value >= 1e6 {
        let compacted = value / 1e6;
        let word = if (3..=10).contains(&(compacted.floor() as u64)) {
            "ملايين"
        } else {
            "مليون"
        };
        return (compact_decimal(compacted), word);
    } else {
        let compacted = value / 1e3;
        let word = if (3..=9).contains(&(compacted.floor() as u64)) {
            "آلاف"
        } else {
            "ألف"
        };
        return (compact_decimal(compacted), word);
    };
    (compact_decimal(value / divisor), word)
}

fn noun_forms(kind: MetricKind) -> (&'static str, &'static str, &'static str, &'static str) {
    match kind {
        MetricKind::Views => ("مشاهدة", "مشاهدتان", "مشاهدات", "مشاهدة"),
        MetricKind::Likes => ("إعجاب", "إعجابان", "إعجابات", "إعجاب"),
        MetricKind::Dislikes => ("عدم إعجاب", "عدم إعجابين", "عدم إعجابات", "عدم إعجاب"),
        MetricKind::Comments => ("تعليق", "تعليقان", "تعليقات", "تعليق"),
        MetricKind::Replies => ("رد", "ردان", "ردود", "رد"),
        MetricKind::Videos | MetricKind::PlaylistVideos | MetricKind::Downloads => {
            ("فيديو", "فيديوهان", "فيديوهات", "فيديو")
        }
        MetricKind::SavedVideos => (
            "فيديو محفوظ",
            "فيديوهان محفوظان",
            "فيديوهات محفوظة",
            "فيديو محفوظ",
        ),
        MetricKind::HistoryEntries => (
            "فيديو شوهدته",
            "فيديوهان شوهدا",
            "فيديوهات شاهدتها",
            "فيديو شوهدته",
        ),
        MetricKind::Subscribers => ("مشترك", "مشتركان", "مشتركون", "مشترك"),
    }
}

fn zero_noun(kind: MetricKind) -> &'static str {
    match kind {
        MetricKind::Views => "لا مشاهدات",
        MetricKind::Likes => "لا إعجابات",
        MetricKind::Dislikes => "لا عدم إعجابات",
        MetricKind::Comments => "لا تعليقات",
        MetricKind::Replies => "لا ردود",
        MetricKind::Videos | MetricKind::PlaylistVideos | MetricKind::Downloads => "لا فيديوهات",
        MetricKind::SavedVideos => "لا فيديوهات محفوظة",
        MetricKind::HistoryEntries => "لا فيديوهات شوهدت",
        MetricKind::Subscribers => "لا مشتركين",
    }
}

fn arabic_quantity(value: u64, isolate: bool) -> String {
    let wrap = |token: &str| {
        if isolate {
            isolate_ltr(token)
        } else {
            token.to_string()
        }
    };
    if value >= 1_000 {
        let (token, unit) = compact_unit(value as f64);
        format!("{} {}", wrap(&token), unit)
    } else {
        wrap(&value.to_string())
    }
}

fn arabic_metric(kind: MetricKind, value: u64, isolate: bool) -> String {
    let (one, two, few, other) = noun_forms(kind);
    if value == 0 {
        return zero_noun(kind).to_string();
    }
    if value >= 1_000 {
        // A compact quantity is followed by the singular noun in Arabic, e.g.
        // "3 ملايين مشاهدة" and "10 ألف مشاهدة".
        return format!("{} {}", arabic_quantity(value, isolate), one);
    }
    let number = if isolate {
        isolate_ltr(&value.to_string())
    } else {
        value.to_string()
    };
    match category(value) {
        Category::Zero => zero_noun(kind).to_string(),
        Category::One => format!("{number} {one}"),
        Category::Two => format!("{number} {two}"),
        Category::Few => format!("{number} {few}"),
        Category::Many | Category::Other => format!("{number} {other}"),
    }
}

fn count_for(language: AppLanguage, kind: MetricKind, value: u64, isolate: bool) -> String {
    if language == AppLanguage::Arabic {
        arabic_metric(kind, value, isolate)
    } else {
        value.to_string()
    }
}

/// Formats a numeric count for the current interface language. Display strings
/// isolate numeric tokens so mixed-direction text keeps the intended order.
pub fn count(kind: MetricKind, value: u64) -> String {
    count_for(crate::i18n::language(), kind, value, true)
}

/// Like [`count`] without direction isolates, for selectable/copyable text.
pub fn count_selectable(kind: MetricKind, value: u64) -> String {
    count_for(crate::i18n::language(), kind, value, false)
}

fn quantity_for(language: AppLanguage, value: u64, isolate: bool) -> String {
    if language == AppLanguage::Arabic {
        arabic_quantity(value, isolate)
    } else {
        crate::api::json::compact_number(value)
    }
}

/// Formats just the numeric part, for translated labels that already supply the
/// noun (for example `الاشتراكات: {0}`). `kind` keeps call sites explicit about
/// what they are counting even though the compact scale is value-driven.
pub fn number(kind: MetricKind, value: u64) -> String {
    let _ = kind;
    quantity_for(crate::i18n::language(), value, true)
}

/// Like [`number`] without direction isolates, for selectable/copyable text.
pub fn number_selectable(kind: MetricKind, value: u64) -> String {
    let _ = kind;
    quantity_for(crate::i18n::language(), value, false)
}

/// Only accepts a numeric token with a known English metric label. Unrecognised
/// text (including already-localised labels and show/episode names) is left to
/// the caller so its meaning is never changed by guessing.
fn recognized_count(kind: MetricKind, text: &str) -> Option<u64> {
    let lower = text.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    let numeric_body = |body: &str| {
        let body = body.trim();
        !body.is_empty()
            && body.chars().all(|ch| {
                ch.is_ascii_digit()
                    || matches!(ch, '.' | ',' | 'k' | 'm' | 'b')
                    || ch.is_ascii_whitespace()
            })
    };
    if numeric_body(&lower) {
        return crate::api::json::parse_count(&lower);
    }
    let suffixes: &[&str] = match kind {
        MetricKind::Views => &["views", "view"],
        MetricKind::Likes => &["likes", "like"],
        MetricKind::Dislikes => &["dislikes", "dislike"],
        MetricKind::Comments => &["comments", "comment"],
        MetricKind::Replies => &["replies", "reply"],
        MetricKind::Videos | MetricKind::PlaylistVideos => &["videos", "video"],
        MetricKind::Downloads => &["downloads", "download"],
        MetricKind::SavedVideos => &["saved videos", "saved video"],
        MetricKind::HistoryEntries => &["history entries", "history entry"],
        MetricKind::Subscribers => &["subscribers", "subscriber"],
    };
    for suffix in suffixes {
        if let Some(body) = lower.strip_suffix(suffix) {
            if numeric_body(body) {
                return crate::api::json::parse_count(body);
            }
        }
    }
    None
}

fn from_text_for(
    language: AppLanguage,
    kind: MetricKind,
    text: Option<&str>,
    isolate: bool,
) -> Option<String> {
    let text = text?;
    let value = recognized_count(kind, text)?;
    if language == AppLanguage::Arabic {
        Some(count_for(language, kind, value, isolate))
    } else {
        Some(text.to_string())
    }
}

/// Converts a recognised display string such as `3.9M views` into an Arabic
/// count. Other labels, including already-localised values, are unchanged.
pub fn from_text(kind: MetricKind, text: Option<&str>) -> Option<String> {
    from_text_for(crate::i18n::language(), kind, text, true)
}

/// Selectable version of [`from_text`]: no display isolates are inserted.
pub fn from_text_selectable(kind: MetricKind, text: Option<&str>) -> Option<String> {
    from_text_for(crate::i18n::language(), kind, text, false)
}

/// Uses the shared formatter for Arabic and keeps the caller's original value
/// for every other language.
pub fn arabic_or(kind: MetricKind, value: u64, fallback: impl Into<String>) -> String {
    let language = crate::i18n::language();
    if language == AppLanguage::Arabic {
        count_for(language, kind, value, true)
    } else {
        fallback.into()
    }
}

/// Selectable version of [`arabic_or`].
pub fn arabic_or_selectable(kind: MetricKind, value: u64, fallback: impl Into<String>) -> String {
    let language = crate::i18n::language();
    if language == AppLanguage::Arabic {
        count_for(language, kind, value, false)
    } else {
        fallback.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cldr_arabic_has_six_plural_categories() {
        assert_eq!(category(0), Category::Zero);
        assert_eq!(category(1), Category::One);
        assert_eq!(category(2), Category::Two);
        assert_eq!(category(3), Category::Few);
        assert_eq!(category(10), Category::Few);
        assert_eq!(category(11), Category::Many);
        assert_eq!(category(99), Category::Many);
        assert_eq!(category(100), Category::Other);
        assert_eq!(category(101), Category::Other);
        assert_eq!(category(1_001), Category::Other);
    }

    #[test]
    fn arabic_compact_forms_match_cldr_long_examples() {
        let views = |n| arabic_metric(MetricKind::Views, n, false);
        assert_eq!(views(1_000), "1 ألف مشاهدة");
        assert_eq!(views(1_100), "1.1 ألف مشاهدة");
        assert_eq!(views(3_000), "3 آلاف مشاهدة");
        assert_eq!(views(10_000), "10 ألف مشاهدة");
        assert_eq!(views(1_000_000), "1 مليون مشاهدة");
        assert_eq!(views(3_000_000), "3 ملايين مشاهدة");
        assert_eq!(views(10_000_000), "10 ملايين مشاهدة");
        assert_eq!(views(1_000_000_000), "1 مليار مشاهدة");
        assert_eq!(views(3_000_000_000), "3 مليار مشاهدة");
        assert_eq!(views(1_000_000_000_000), "1 ترليون مشاهدة");
    }

    #[test]
    fn numeric_only_formatter_keeps_translated_labels_intact() {
        let quantity = |n| quantity_for(AppLanguage::Arabic, n, false);
        assert_eq!(quantity(7), "7");
        assert_eq!(quantity(3_900_000), "3.9 ملايين");
        assert_eq!(
            quantity_for(AppLanguage::Arabic, 3_900_000, true),
            "\u{2066}3.9\u{2069} ملايين"
        );
        assert!(!quantity(3_900_000).contains("مشاهدة"));
        assert_eq!(quantity_for(AppLanguage::English, 3_900_000, false), "3.9M");
    }

    #[test]
    fn whole_counts_use_six_noun_forms_with_western_digits() {
        let views = |n| arabic_metric(MetricKind::Views, n, false);
        assert_eq!(views(0), "لا مشاهدات");
        assert_eq!(views(1), "1 مشاهدة");
        assert_eq!(views(2), "2 مشاهدتان");
        assert_eq!(views(3), "3 مشاهدات");
        assert_eq!(views(11), "11 مشاهدة");
        assert_eq!(views(100), "100 مشاهدة");
        assert!(
            !views(3_900_000)
                .chars()
                .any(|ch| ('\u{0660}'..='\u{0669}').contains(&ch))
        );
    }

    #[test]
    fn every_metric_kind_has_an_arabic_form() {
        for kind in [
            MetricKind::Views,
            MetricKind::Likes,
            MetricKind::Dislikes,
            MetricKind::Comments,
            MetricKind::Replies,
            MetricKind::Videos,
            MetricKind::PlaylistVideos,
            MetricKind::Downloads,
            MetricKind::SavedVideos,
            MetricKind::HistoryEntries,
            MetricKind::Subscribers,
        ] {
            for value in [0, 1, 2, 3, 11, 100, 3_900_000] {
                let formatted = arabic_metric(kind, value, false);
                assert!(!formatted.trim().is_empty(), "{kind:?} {value}");
                assert!(!formatted.contains("views"), "{kind:?} {value}");
            }
        }
    }

    #[test]
    fn display_numbers_are_isolated_but_selectable_numbers_are_not() {
        let display = arabic_metric(MetricKind::Views, 3_900_000, true);
        let selectable = arabic_metric(MetricKind::Views, 3_900_000, false);
        assert_eq!(display, "\u{2066}3.9\u{2069} ملايين مشاهدة");
        assert_eq!(selectable, "3.9 ملايين مشاهدة");
        assert!(!selectable.contains('\u{2066}'));
    }

    #[test]
    fn preformatted_youtube_counts_become_arabic_only_with_isolates() {
        let display = from_text_for(
            AppLanguage::Arabic,
            MetricKind::Views,
            Some("3.9M views"),
            true,
        );
        let selectable = from_text_for(
            AppLanguage::Arabic,
            MetricKind::Views,
            Some("3.9M views"),
            false,
        );
        assert_eq!(display, Some("\u{2066}3.9\u{2069} ملايين مشاهدة".into()));
        assert_eq!(selectable, Some("3.9 ملايين مشاهدة".into()));
        assert_eq!(
            from_text_for(
                AppLanguage::English,
                MetricKind::Views,
                Some("3.9M views"),
                true
            ),
            Some("3.9M views".into())
        );
    }

    #[test]
    fn unknown_and_already_localised_labels_are_not_rewritten() {
        for text in [
            "7 episodes",
            "Point • 13 episodes",
            "No views",
            "108 ألف مشاهدة",
        ] {
            assert_eq!(
                from_text_for(AppLanguage::Arabic, MetricKind::Views, Some(text), true),
                None
            );
        }
    }

    #[test]
    fn plain_vote_counts_are_formatted_without_losing_numeric_input() {
        assert_eq!(
            from_text_for(AppLanguage::Arabic, MetricKind::Likes, Some("3"), false),
            Some("3 إعجابات".into())
        );
        assert_eq!(recognized_count(MetricKind::Likes, "3 dislikes"), None);
        assert_eq!(
            recognized_count(MetricKind::Subscribers, "1.2M subscribers"),
            Some(1_200_000)
        );
    }
}
