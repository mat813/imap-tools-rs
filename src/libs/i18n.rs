//! Locale selection for user-facing messages.
//!
//! The language is picked automatically from the environment (`LC_ALL`,
//! `LC_MESSAGES`, `LANG`, ...). When it is not one of the supported languages,
//! the messages stay in English.

/// Languages for which a translation exists, primary language subtag only.
pub const SUPPORTED: &[&str] = &["en", "fr", "de", "ja", "zh", "ko"];

/// Reduce a locale tag such as `fr_FR.UTF-8` or `zh-Hans-CN` to its supported
/// primary language, if any.
fn primary_language(tag: &str) -> Option<&'static str> {
    let language = tag.split(['_', '-', '.', '@']).next()?.to_ascii_lowercase();
    SUPPORTED.iter().copied().find(|l| *l == language)
}

/// Select the display language from the system locale.
///
/// Must be called before anything user-facing is built, in particular before
/// the command line is parsed, as clap resolves its help texts at that point.
pub fn init() {
    if let Some(language) = sys_locale::get_locale()
        .as_deref()
        .and_then(primary_language)
    {
        rust_i18n::set_locale(language);
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
        sync::LazyLock,
    };

    use regex::Regex;
    use rstest::rstest;
    use rust_i18n::t;

    use super::*;

    /// Translations of each key, by language.
    type Catalog = BTreeMap<String, BTreeMap<String, String>>;

    /// The path of `path` under `root`, with `/` as separator.
    fn relative(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn files_with_extension(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                files_with_extension(&path, extension, files);
                continue;
            }
            if path.extension().is_some_and(|e| e == extension) {
                files.push(path);
            }
        }
    }

    /// The files of the catalog, by path under `locales/`, with their content.
    fn locale_files() -> BTreeMap<String, String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let mut files = Vec::new();
        files_with_extension(&root, "yml", &mut files);
        files
            .iter()
            // `cargo i18n` writes what is missing there, rust-i18n does not load it
            .filter(|f| !f.ends_with("TODO.yml"))
            .map(|f| {
                (
                    relative(&root, f),
                    fs::read_to_string(f).expect("read locale file"),
                )
            })
            .collect()
    }

    /// The texts of each locale file, by key.
    fn catalog_by_file() -> BTreeMap<String, Catalog> {
        locale_files()
            .into_iter()
            .map(|(file, text)| {
                let raw: BTreeMap<String, serde_value::Value> =
                    serde_any::from_str(&text, serde_any::Format::Yaml)
                        .unwrap_or_else(|e| unreachable!("{file} is not valid YAML: {e}"));
                let catalog = raw
                    .into_iter()
                    .filter(|entry| !entry.0.starts_with('_'))
                    .map(|(key, value)| {
                        let translations = value.deserialize_into().unwrap_or_else(|e| {
                            unreachable!("{file}: {key} is not a language map: {e}")
                        });
                        (key, translations)
                    })
                    .collect();
                (file, catalog)
            })
            .collect()
    }

    /// Every text, whichever file it is in.
    fn catalog() -> Catalog {
        catalog_by_file().into_values().flatten().collect()
    }

    fn placeholders(text: &str) -> BTreeSet<&str> {
        static PLACEHOLDER: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"%\{(\w+)\}").expect("regex"));
        PLACEHOLDER
            .captures_iter(text)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect()
    }

    /// Every key given to `t!()` in the sources, but the ones of this file, with the
    /// source files (under `src/`) that use it.
    fn used_keys() -> BTreeMap<String, BTreeSet<String>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        files_with_extension(&root, "rs", &mut files);
        let call = Regex::new(r#"\bt!\(\s*"([^"]+)""#).expect("regex");
        let mut used: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in files.iter().filter(|f| !f.ends_with("libs/i18n.rs")) {
            let text = fs::read_to_string(file).expect("read source");
            for key in call.captures_iter(&text).filter_map(|c| c.get(1)) {
                used.entry(key.as_str().to_owned())
                    .or_default()
                    .insert(relative(&root, file));
            }
        }
        used
    }

    /// The file of `locales/` for the texts that only this source file uses. What is not
    /// here (`main.rs`, `run.rs`, `commands/mod.rs`), and what several of these files use,
    /// is in `app.yml`.
    const HOMES: &[(&str, &str)] = &[
        ("commands/archive.rs", "commands/archive.yml"),
        ("commands/clean.rs", "commands/clean.yml"),
        ("commands/find_dups.rs", "commands/find_dups.yml"),
        ("commands/list.rs", "commands/list.yml"),
        ("commands/imap/create.rs", "commands/imap.yml"),
        ("commands/imap/delete.rs", "commands/imap.yml"),
        ("commands/imap/disk_usage.rs", "commands/imap.yml"),
        ("commands/imap/list.rs", "commands/imap.yml"),
        ("commands/imap/mod.rs", "commands/imap.yml"),
        ("libs/args.rs", "libs/args.yml"),
        ("libs/auth/mod.rs", "libs/auth.yml"),
        ("libs/base_config.rs", "libs/config.yml"),
        ("libs/config.rs", "libs/config.yml"),
        ("libs/filter.rs", "libs/config.yml"),
        ("libs/imap.rs", "libs/imap.yml"),
        ("libs/mode.rs", "libs/mode.yml"),
        ("libs/render/mod.rs", "libs/render.yml"),
        ("libs/render/traits.rs", "libs/render.yml"),
    ];

    fn home(source: &str) -> &'static str {
        HOMES
            .iter()
            .find(|entry| entry.0 == source)
            .map_or("app.yml", |entry| entry.1)
    }

    #[rstest]
    #[case("fr_FR.UTF-8", Some("fr"))]
    #[case("fr-FR", Some("fr"))]
    #[case("de_DE@euro", Some("de"))]
    #[case("ja_JP.eucJP", Some("ja"))]
    #[case("zh-Hans-CN", Some("zh"))]
    #[case("zh_TW", Some("zh"))]
    #[case("KO_kr", Some("ko"))]
    #[case("en_US.UTF-8", Some("en"))]
    #[case("es_ES", None)]
    #[case("C", None)]
    #[case("", None)]
    fn primary_language_from_tag(#[case] tag: &str, #[case] expected: Option<&str>) {
        assert_eq!(primary_language(tag), expected);
    }

    #[test]
    fn catalog_has_exactly_the_supported_languages() {
        let available: BTreeSet<String> = rust_i18n::available_locales!()
            .iter()
            .map(ToString::to_string)
            .collect();
        let supported: BTreeSet<String> = SUPPORTED.iter().map(|l| (*l).to_owned()).collect();
        assert_eq!(available, supported);
    }

    #[test]
    fn every_key_is_translated_in_every_language() {
        let mut problems = Vec::new();
        for (key, translations) in catalog() {
            let expected = placeholders(translations.get("en").expect("english text"));
            for language in SUPPORTED {
                match translations.get(*language) {
                    None => problems.push(format!("{key}: no {language} translation")),
                    Some(text) if text.trim().is_empty() => {
                        problems.push(format!("{key}: empty {language} translation"));
                    },
                    Some(text) if text.contains("TODO") => {
                        problems.push(format!("{key}: {language} translation is a TODO"));
                    },
                    Some(text) if placeholders(text) != expected => problems.push(format!(
                        "{key}: {language} placeholders {:?} differ from {expected:?}",
                        placeholders(text)
                    )),
                    Some(_) => {},
                }
            }
            for language in translations.keys() {
                if !SUPPORTED.contains(&language.as_str()) {
                    problems.push(format!("{key}: unsupported language {language}"));
                }
            }
        }
        assert_eq!(problems, [] as [String; 0]);
    }

    #[test]
    fn every_used_key_exists_and_every_key_is_used() {
        let catalog: BTreeSet<String> = catalog().into_keys().collect();
        let used: BTreeSet<String> = used_keys().into_keys().collect();
        let missing: Vec<_> = used.difference(&catalog).collect();
        let unused: Vec<_> = catalog.difference(&used).collect();
        assert!(
            missing.is_empty(),
            "keys used but not translated: {missing:#?}"
        );
        assert_eq!(unused, [] as [&String; 0]);
    }

    #[test]
    fn every_locale_file_is_version_2() {
        // Without it, rust-i18n reads the file as the texts of a language named after it
        let wrong: Vec<_> = locale_files()
            .into_iter()
            .filter(|entry| !entry.1.starts_with("_version: 2\n"))
            .map(|(file, _)| file)
            .collect();
        assert!(
            wrong.is_empty(),
            "files not starting with `_version: 2`: {wrong:#?}"
        );
    }

    #[test]
    fn no_key_is_defined_in_two_files() {
        // rust-i18n merges the files, and silently keeps only one of the two texts
        let mut owner: BTreeMap<String, String> = BTreeMap::new();
        let mut problems = Vec::new();
        for (file, catalog) in catalog_by_file() {
            for key in catalog.into_keys() {
                if let Some(other) = owner.insert(key.clone(), file.clone()) {
                    problems.push(format!("{key}: defined in {other} and in {file}"));
                }
            }
        }
        assert_eq!(problems, [] as [String; 0]);
    }

    #[test]
    fn keys_live_in_the_file_of_their_module() {
        let used = used_keys();
        let mut problems = Vec::new();
        for (file, catalog) in catalog_by_file() {
            for key in catalog.keys() {
                // Keys that are never used are reported by another test
                let Some(sources) = used.get(key) else {
                    continue;
                };
                let homes: BTreeSet<&str> = sources.iter().map(|s| home(s)).collect();
                let expected = if homes.len() == 1 {
                    homes.first().copied().expect("one home")
                } else {
                    "app.yml"
                };
                if file != expected {
                    problems.push(format!(
                        "{key}: in {file}, expected in {expected} (used by {sources:?})"
                    ));
                }
            }
        }
        assert_eq!(problems, [] as [String; 0]);
    }

    #[rstest]
    #[case("en", "Path to the configuration file")]
    #[case("fr", "Chemin du fichier de configuration")]
    #[case("de", "Pfad zur Konfigurationsdatei")]
    #[case("ja", "設定ファイルのパス")]
    #[case("zh", "配置文件的路径")]
    #[case("ko", "설정 파일 경로")]
    // A territory falls back to its language, an unknown language to English
    #[case("zh-TW", "配置文件的路径")]
    #[case("es", "Path to the configuration file")]
    fn translates_to_the_requested_language(#[case] locale: &str, #[case] expected: &str) {
        assert_eq!(t!("cli.args.config", locale = locale), expected);
    }

    #[test]
    fn placeholders_and_line_breaks_are_translated() {
        let text = t!(
            "error.base_config.command_fail",
            locale = "fr",
            command_type = "password",
            command = "\"x\"",
            status = "exit status: 3",
            stdout = "STDOUT: out",
            stderr = "STDERR: oops"
        );
        assert_eq!(
            text,
            "Exécution de la commande password \"x\" (sortie exit status: 3)\nSTDOUT: out\nSTDERR: oops"
        );
    }

    #[test]
    fn debug_specifier_quotes_the_value() {
        assert_eq!(
            t!(
                "output.imap_create.already_exists",
                locale = "en",
                mailbox = "Archive" : {:?},
                reason = "NO Mailbox already exists"
            ),
            "Cannot create \"Archive\", it already exists: NO Mailbox already exists"
        );
    }
}
