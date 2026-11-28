//! Interface language, independent of the language requested from YouTube.
use gpui_kit::base::LayoutDirection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum AppLanguage {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "az")]
    Azerbaijani,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "kk")]
    Kazakh,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "tr")]
    Turkish,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
    #[serde(rename = "zh-TW")]
    ChineseTraditional,
}

impl AppLanguage {
    pub fn direction(self) -> LayoutDirection {
        LayoutDirection::from_locale(self.code())
    }

    pub const OPTIONS: &[(Self, &str)] = &[
        (Self::English, "English"),
        (Self::Arabic, "العربية"),
        (Self::Azerbaijani, "Azərbaycanca"),
        (Self::German, "Deutsch"),
        (Self::French, "Français"),
        (Self::Turkish, "Türkçe"),
        (Self::Russian, "Русский"),
        (Self::Ukrainian, "Українська"),
        (Self::Kazakh, "Қазақша"),
        (Self::Japanese, "日本語"),
        (Self::ChineseSimplified, "简体中文"),
        (Self::ChineseTraditional, "繁體中文"),
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Arabic => "ar",
            Self::Russian => "ru",
            Self::French => "fr",
            Self::Azerbaijani => "az",
            Self::Ukrainian => "uk",
            Self::Kazakh => "kk",
            Self::German => "de",
            Self::Turkish => "tr",
            Self::Japanese => "ja",
            Self::ChineseSimplified => "zh-CN",
            Self::ChineseTraditional => "zh-TW",
        }
    }

    fn catalog(self) -> Option<&'static Catalog> {
        static AR: Catalog = Catalog::new(include_str!("i18n/locales/ar.json"));
        static RU: Catalog = Catalog::new(include_str!("i18n/locales/ru.json"));
        static FR: Catalog = Catalog::new(include_str!("i18n/locales/fr.json"));
        static AZ: Catalog = Catalog::new(include_str!("i18n/locales/az.json"));
        static UK: Catalog = Catalog::new(include_str!("i18n/locales/uk.json"));
        static KK: Catalog = Catalog::new(include_str!("i18n/locales/kk.json"));
        static DE: Catalog = Catalog::new(include_str!("i18n/locales/de.json"));
        static TR: Catalog = Catalog::new(include_str!("i18n/locales/tr.json"));
        static JA: Catalog = Catalog::new(include_str!("i18n/locales/ja.json"));
        static ZH_CN: Catalog = Catalog::new(include_str!("i18n/locales/zh-CN.json"));
        static ZH_TW: Catalog = Catalog::new(include_str!("i18n/locales/zh-TW.json"));

        Some(match self {
            Self::English => return None,
            Self::Arabic => &AR,
            Self::Russian => &RU,
            Self::French => &FR,
            Self::Azerbaijani => &AZ,
            Self::Ukrainian => &UK,
            Self::Kazakh => &KK,
            Self::German => &DE,
            Self::Turkish => &TR,
            Self::Japanese => &JA,
            Self::ChineseSimplified => &ZH_CN,
            Self::ChineseTraditional => &ZH_TW,
        })
    }

    fn translate(self, source: &str) -> &str {
        self.catalog()
            .and_then(|catalog| catalog.messages().get(source))
            .map(String::as_str)
            .unwrap_or(source)
    }

    fn format(self, source: &str, args: &[&dyn Display], display_only: bool) -> String {
        let translated = self.translate(source);
        format_template(
            translated,
            args,
            display_only && self.direction().is_rtl() && translated != source,
        )
    }
}

struct Catalog {
    source: &'static str,
    messages: OnceLock<HashMap<String, String>>,
}

impl Catalog {
    const fn new(source: &'static str) -> Self {
        Self {
            source,
            messages: OnceLock::new(),
        }
    }

    fn messages(&self) -> &HashMap<String, String> {
        self.messages.get_or_init(|| {
            serde_json::from_str(self.source).expect("valid embedded interface catalog")
        })
    }
}

#[cfg(test)]
fn parse_catalog(source: &str) -> serde_json::Result<HashMap<String, String>> {
    use serde::de;
    struct UniqueKeys;

    impl<'de> de::Visitor<'de> for UniqueKeys {
        type Value = HashMap<String, String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a translation catalog with unique keys")
        }

        fn visit_map<A: de::MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
            let mut result = HashMap::new();
            while let Some((key, value)) = entries.next_entry::<String, String>()? {
                match result.entry(key) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(value);
                    }
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        return Err(de::Error::custom(format!(
                            "duplicate translation key: {}",
                            entry.key()
                        )));
                    }
                }
            }
            Ok(result)
        }
    }

    let mut json = serde_json::Deserializer::from_str(source);
    let result = de::Deserializer::deserialize_map(&mut json, UniqueKeys)?;
    json.end()?;
    Ok(result)
}

pub fn language() -> AppLanguage {
    crate::store::settings_snapshot().app_language
}

pub fn t(source: &str) -> &str {
    language().translate(source)
}

pub fn apply_component_locale(cx: &mut gpui_kit::App) {
    let language = language();
    gpui_kit::component::set_locale(language.code());
    language.direction().set_global(cx);
}

pub fn format(source: &str, args: &[&dyn Display]) -> String {
    language().format(source, args, true)
}

pub fn format_selectable(source: &str, args: &[&dyn Display]) -> String {
    language().format(source, args, false)
}

/// Wraps a known LTR value for display only. Stored and copied values stay raw.
pub fn isolate_ltr(value: impl Display) -> String {
    let value = value.to_string();
    let mut message = String::with_capacity(value.len() + 2);
    push_isolate(&mut message, &value, '\u{2066}');
    message
}

/// Isolates technical values such as `1080p` or `1.25x`, but leaves translated
/// Arabic text untouched.
pub fn isolate_ascii(value: impl Display) -> String {
    let value = value.to_string();
    let has_ascii = value.chars().any(|ch| ch.is_ascii_alphanumeric());
    let has_non_ascii_letter = value.chars().any(|ch| ch.is_alphabetic() && !ch.is_ascii());
    if has_ascii && !has_non_ascii_letter {
        isolate_ltr(value)
    } else {
        value
    }
}

// Translators can specify a known direction (`{0:ltr}` / `{0:rtl}`).
// Unannotated arguments use their first strong character.
fn argument(field: &str) -> Option<(usize, char)> {
    let (index, direction) = match field.split_once(':') {
        Some((index, "ltr")) => (index, '\u{2066}'),
        Some((index, "rtl")) => (index, '\u{2067}'),
        Some(_) => return None,
        None => (field, '\u{2068}'),
    };
    Some((index.parse().ok()?, direction))
}

fn is_directional_format(ch: char) -> bool {
    matches!(ch, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

fn push_isolate(message: &mut String, value: &str, direction: char) {
    // These strings are for display only; stored and copied values stay intact.
    if !value.chars().all(is_directional_format) {
        message.push(direction);
        message.extend(value.chars().filter(|&ch| !is_directional_format(ch)));
        message.push('\u{2069}');
    }
}

fn format_template(template: &str, args: &[&dyn Display], isolate_args: bool) -> String {
    let mut message = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((prefix, field_and_tail)) = rest.split_once('{') {
        let Some((field, mut tail)) = field_and_tail.split_once('}') else {
            break;
        };
        message.push_str(prefix);
        if let Some((value, direction)) =
            argument(field).and_then(|(index, direction)| Some((args.get(index)?, direction)))
        {
            if isolate_args {
                let mut value = value.to_string();
                // A percent or multiplication sign belongs to the preceding
                // numeric token, even in a right-to-left sentence.
                if direction == '\u{2066}'
                    && !value.is_empty()
                    && let Some(suffix @ ('%' | '×')) = tail.chars().next()
                {
                    value.push(suffix);
                    tail = &tail[suffix.len_utf8()..];
                }
                push_isolate(&mut message, &value, direction);
            } else {
                write!(message, "{value}").expect("writing to a string");
            }
        } else {
            message.push('{');
            message.push_str(field);
            message.push('}');
        }
        rest = tail;
    }
    message.push_str(rest);
    message
}

// Catalogs can reorder arguments. Apply Rust's numeric formats at the call site.
macro_rules! tf {
    (selectable: $source:expr $(, $arg:expr)* $(,)?) => {
        $crate::i18n::format_selectable($source, &[$(&($arg) as &dyn std::fmt::Display),*])
    };
    ($source:expr $(, $arg:expr)* $(,)?) => {
        $crate::i18n::format($source, &[$(&($arg) as &dyn std::fmt::Display),*])
    };
}

pub(crate) use tf;

#[cfg(test)]
mod tests {
    use super::{
        AppLanguage, LayoutDirection, argument, format_template, isolate_ascii, isolate_ltr,
        parse_catalog,
    };
    use crate::store::Settings;

    #[test]
    fn old_settings_keep_the_interface_in_english() {
        let settings: Settings = serde_json::from_str(r#"{"language":"ru"}"#).unwrap();
        assert_eq!(settings.app_language, AppLanguage::English);
        assert_eq!(settings.language, "ru");
    }

    #[test]
    fn russian_interface_round_trips_independently() {
        let settings: Settings =
            serde_json::from_str(r#"{"app_language":"ru","language":"en"}"#).unwrap();
        assert_eq!(settings.app_language, AppLanguage::Russian);
        assert_eq!(settings.language, "en");
        assert_eq!(
            serde_json::to_value(settings).unwrap()["app_language"],
            "ru"
        );
    }

    #[test]
    fn falls_back_to_the_source_text() {
        assert_eq!(AppLanguage::English.translate("Settings"), "Settings");
        assert_eq!(AppLanguage::Russian.translate("Settings"), "Настройки");
        assert_eq!(
            AppLanguage::Russian.translate("An untranslated label"),
            "An untranslated label"
        );
    }

    #[test]
    fn every_language_has_the_same_messages_and_arguments() {
        let reference = AppLanguage::OPTIONS
            .iter()
            .filter(|(language, _)| *language != AppLanguage::Arabic)
            .filter_map(|(language, _)| language.catalog())
            .flat_map(|catalog| catalog.messages().keys())
            .collect::<std::collections::HashSet<_>>();
        let placeholders = |text: &str| {
            let mut slots: Vec<_> = text
                .split('{')
                .skip(1)
                .filter_map(|part| part.split_once('}'))
                .filter_map(|(field, _)| argument(field).map(|(index, _)| index))
                .collect();
            slots.sort_unstable();
            slots
        };
        for &(language, _) in AppLanguage::OPTIONS {
            let saved = serde_json::to_value(language).unwrap();
            assert_eq!(saved, language.code());
            assert_eq!(
                serde_json::from_value::<AppLanguage>(saved).unwrap(),
                language
            );
            let Some(catalog) = language.catalog() else {
                continue;
            };
            let messages = parse_catalog(catalog.source)
                .unwrap_or_else(|error| panic!("{}: {error}", language.code()));
            for &source in &reference {
                assert!(
                    messages.contains_key(source),
                    "{}: missing {source}",
                    language.code()
                );
            }
            for (source, translated) in &messages {
                assert!(
                    !translated.trim().is_empty(),
                    "{}: {source}",
                    language.code()
                );
                assert_eq!(
                    placeholders(translated),
                    placeholders(source),
                    "{}: {source}",
                    language.code()
                );
            }
        }
    }

    #[test]
    fn formatting_can_reorder_arguments_without_interpreting_inserted_text() {
        assert_eq!(
            format_template("{1} / {0}", &[&"title {1}", &42], false),
            "42 / title {1}"
        );
        assert_eq!(
            format_template("{0}%", &[&format_args!("{:.1}", 1.24)], false),
            "1.2%"
        );
    }

    #[test]
    fn arabic_messages_isolate_mixed_direction_arguments() {
        assert_eq!(
            format_template("نتائج البحث عن «{0}»", &[&"Rust 2026"], true),
            "نتائج البحث عن «\u{2068}Rust 2026\u{2069}»"
        );
    }

    #[test]
    fn catalog_rejects_duplicate_keys_instead_of_overwriting() {
        assert!(parse_catalog(r#"{"Play":"A","Play":"B"}"#).is_err());
        assert!(parse_catalog(r#"{"Play":"A","play":"B"}"#).is_ok());
        let arabic = AppLanguage::Arabic.catalog().unwrap();
        assert_eq!(parse_catalog(arabic.source).unwrap(), *arabic.messages());
    }

    #[test]
    fn interface_direction_comes_from_the_saved_app_language() {
        for &(language, _) in AppLanguage::OPTIONS {
            let expected = if language == AppLanguage::Arabic {
                LayoutDirection::Rtl
            } else {
                LayoutDirection::Ltr
            };
            assert_eq!(language.direction(), expected);
        }
        let settings: Settings =
            serde_json::from_str(r#"{"app_language":"ar","language":"en"}"#).unwrap();
        assert_eq!(settings.app_language.direction(), LayoutDirection::Rtl);
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn mixed_messages_keep_punctuation_and_keep_tokens_intact() {
        assert_eq!(
            format_template(
                "افتح YouTube: «{0}»، من ({1:ltr}).",
                &[&"Rust: a guide", &"https://example.test/a?q=1"],
                true
            ),
            "افتح YouTube: «\u{2068}Rust: a guide\u{2069}»، من (\u{2066}https://example.test/a?q=1\u{2069})."
        );
        let path = r"C:\Users\user\Downloads\مقطع (1).mp4";
        assert_eq!(
            format_template("حُفظ الملف في {0:ltr}", &[&path], true),
            format!("حُفظ الملف في \u{2066}{path}\u{2069}")
        );
        assert_eq!(
            format_template("القناة «{0}»", &[&"قناة Rust الرسمية"], true),
            "القناة «\u{2068}قناة Rust الرسمية\u{2069}»"
        );
    }

    #[test]
    fn formatted_values_support_numeric_metadata_and_reordering() {
        assert_eq!(
            format_template(
                "التاريخ: {1:ltr}، المشاهدات: {0:ltr}، التقدم: {2:ltr}، المدة: {3:ltr}",
                &[&"190,038", &"14/08/2026", &"42.5%", &"1:02:03"],
                true
            ),
            "التاريخ: \u{2066}14/08/2026\u{2069}، المشاهدات: \u{2066}190,038\u{2069}، التقدم: \u{2066}42.5%\u{2069}، المدة: \u{2066}1:02:03\u{2069}"
        );
        assert_eq!(
            format_template("«{0:rtl}»", &[&"العربية"], true),
            "«\u{2067}العربية\u{2069}»"
        );
    }

    #[test]
    fn known_ltr_values_can_be_isolated_for_display_only() {
        assert_eq!(isolate_ltr("14/08/2026"), "\u{2066}14/08/2026\u{2069}");
        assert_eq!(
            isolate_ltr("\u{2068}42\u{2069}\u{202E}"),
            "\u{2066}42\u{2069}"
        );
        assert_eq!(isolate_ascii("1080p"), "\u{2066}1080p\u{2069}");
        assert_eq!(isolate_ascii("1.25x"), "\u{2066}1.25x\u{2069}");
        assert_eq!(isolate_ascii("تلقائي"), "تلقائي");
    }

    #[test]
    fn source_metadata_marks_the_technical_identifier_as_ltr() {
        assert_eq!(
            AppLanguage::Arabic.translate("Source: {0}, libmpv"),
            "المصدر: {0:ltr}، المشغّل: libmpv"
        );
        for key in ["Flat", "Bass boost", "Voice", "Treble boost", "Warm"] {
            assert_ne!(AppLanguage::Arabic.translate(key), key, "{key}");
        }
    }

    #[test]
    fn formatting_handles_empty_missing_and_already_isolated_arguments() {
        assert_eq!(
            format_template(
                "({0}) {1:ltr} {2}",
                &[&"", &"\u{2068}42\u{2069}\u{202E}"],
                true
            ),
            "() \u{2066}42\u{2069} {2}"
        );
    }

    #[test]
    fn suffixes_and_selectable_values_have_the_expected_copy_semantics() {
        assert_eq!(
            AppLanguage::Arabic.format("Downloading… {0}%", &[&42], true),
            "جارٍ التنزيل… \u{2066}42%\u{2069}"
        );
        let path = r"C:\Videos\مقطع.mp4";
        assert_eq!(
            AppLanguage::Arabic.format("Saved to {0}", &[&path], false),
            format!("حُفظ في {path}")
        );
        assert_eq!(
            AppLanguage::English.format("Downloading… {0}%", &[&42], true),
            "Downloading… 42%"
        );
    }
}
