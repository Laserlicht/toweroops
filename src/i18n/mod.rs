use std::path::Path;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

/// Fluent-based internationalization.
pub struct I18n {
    bundle: FluentBundle<FluentResource>,
    lang: String,
}

impl I18n {
    /// Load `.ftl` files from the resources directory and auto-detect the system language.
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Self {
        let dir = dir.as_ref();

        // Detect system locale
        let sys_lang = sys_locale::get_locale()
            .unwrap_or_else(|| "en".to_string())
            .to_lowercase();
        let lang_code = if sys_lang.starts_with("de") {
            "de"
        } else {
            "en"
        };

        // Try loading requested language, fall back to English
        if let Some(i18n) = Self::try_load(dir, lang_code) {
            return i18n;
        }
        if lang_code != "en" {
            if let Some(i18n) = Self::try_load(dir, "en") {
                return i18n;
            }
        }

        // Empty fallback
        let langid: LanguageIdentifier = "en".parse().unwrap();
        Self {
            bundle: FluentBundle::new(vec![langid]),
            lang: "en".to_string(),
        }
    }

    fn try_load(dir: &Path, lang: &str) -> Option<Self> {
        let path = dir.join(format!("{}.ftl", lang));
        let source = std::fs::read_to_string(&path).ok()?;
        let resource = FluentResource::try_new(source).ok()?;
        let langid: LanguageIdentifier = lang.parse().ok()?;
        let mut bundle = FluentBundle::new(vec![langid]);
        bundle.add_resource(resource).ok()?;
        Some(Self {
            bundle,
            lang: lang.to_string(),
        })
    }

    /// Get a translated message by its identifier.
    pub fn t(&self, id: &str) -> String {
        self.format(id, None)
    }

    /// Get a translated message with arguments.
    #[allow(dead_code)]
    pub fn t_args(&self, id: &str, args: &FluentArgs) -> String {
        self.format(id, Some(args))
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        let msg = match self.bundle.get_message(id) {
            Some(m) => m,
            None => return id.to_string(),
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return id.to_string(),
        };
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, args, &mut errors)
            .to_string()
    }

    #[allow(dead_code)]
    pub fn current_language(&self) -> &str {
        &self.lang
    }
}
