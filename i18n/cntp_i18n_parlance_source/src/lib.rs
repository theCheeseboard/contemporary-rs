mod parse_raw_string;

use crate::parse_raw_string::parse_raw_string;
use cntp_i18n::{
    I18N_MANAGER, I18nEntry, I18nPluralStringEntry, I18nSource, I18nString, I18nStringPart, Locale,
};
use serde::Deserialize;
use signalr_client::{ArgumentConfiguration, InvocationContext, SignalRClient};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};
use url::ParseError;
use zed_reqwest::{Client, Url};

pub struct CntpI18nParlanceSource {
    base_url: Url,
    project: String,
    subproject: String,

    crate_name: String,

    entries: Arc<RwLock<HashMap<String, HashMap<String, &'static I18nEntry<'static>>>>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParlanceEntry {
    key: String,
    context: String,
    source: String,
    translation: Vec<ParlanceEntryTranslation>,
    requires_pluralisation: bool,
    comment: Option<String>,
    old_source_string: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParlanceEntryTranslation {
    plural_type: String,
    translation_content: String,
}

#[derive(Debug)]
pub enum ParlanceSourceError {
    RequestError(zed_reqwest::Error),
    UrlParseError,
}

impl Display for ParlanceSourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError(e) => write!(f, "Request error: {}", e),
            Self::UrlParseError => write!(f, "URL parse error"),
        }
    }
}

impl Error for ParlanceSourceError {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SubprojectResponse {
    available_languages: Vec<SubprojectAvailableLanguage>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SubprojectAvailableLanguage {
    language: String,
}

impl From<ParseError> for ParlanceSourceError {
    fn from(_: ParseError) -> Self {
        ParlanceSourceError::UrlParseError
    }
}

impl From<zed_reqwest::Error> for ParlanceSourceError {
    fn from(value: zed_reqwest::Error) -> Self {
        ParlanceSourceError::RequestError(value)
    }
}

enum SignalRMessage {
    Subscribe {
        project: String,
        subproject: String,
        language: String,
    },
}

enum SignalRHubMessage {
    TranslationUpdated {
        project: String,
        subproject: String,
        language: String,
        data: HashMap<String, Vec<ParlanceEntryTranslation>>,
    },
}

impl CntpI18nParlanceSource {
    pub async fn new(
        base_url: Url,
        project: String,
        subproject: String,
        crate_name: String,
    ) -> Result<Self, ParlanceSourceError> {
        warn!(
            "The Parlance translation source leaks memory! Turn off the Parlance source if you are not using it to translate this application."
        );

        let client = Client::builder().build()?;
        let mut entries = HashMap::new();

        // Find all languages supported by the project
        let response = client
            .get(base_url.join(&format!("api/projects/{}/{}", project, subproject))?)
            .send()
            .await?;

        let response = response.json::<SubprojectResponse>().await?;
        for language in &response.available_languages {
            // Find entries for this language
            let response = client
                .get(base_url.join(&format!(
                    "api/projects/{}/{}/{}/entries",
                    project, subproject, language.language
                ))?)
                .send()
                .await?;

            response.error_for_status_ref()?;

            let parlance_entries = response.json::<Vec<ParlanceEntry>>().await?;

            let mut i18n_entries = HashMap::new();
            for entry in parlance_entries {
                if let Some(i18n_entry) = entry.to_i18n_entry(language.language.clone().into()) {
                    let boxed = Box::new(i18n_entry);
                    i18n_entries.insert(entry.key.clone(), Box::leak(boxed) as &'static I18nEntry);
                }
            }
            entries.insert(language.language.clone(), i18n_entries);
        }

        // let entries = Arc::new(RwLock::new(entries));
        let (tx_signalr, mut rx_signalr) = tokio::sync::mpsc::channel(16);
        let (tx_signalr_ret, mut rx_signalr_ret) = tokio::sync::mpsc::channel(16);

        tokio::spawn({
            let base_url = base_url.clone();
            async move {
                let mut signalr_client = match SignalRClient::connect_with(
                    &format!(
                        "{}:{}",
                        base_url.host_str().unwrap(),
                        base_url.port_or_known_default().unwrap()
                    ),
                    "api/signalr/translator",
                    |c| {
                        if base_url.scheme() == "http" {
                            c.unsecure();
                        }
                    },
                )
                .await
                {
                    Ok(signalr_client) => signalr_client,
                    Err(e) => {
                        error!("Unable to connect to SignalR endpoint: {:?}", e);
                        return;
                    }
                };

                signalr_client.register("TranslationUpdated".into(), {
                    let tx_signalr_ret = tx_signalr_ret.clone();
                    move |cx: InvocationContext| {
                        tokio::spawn({
                            let tx_signalr_ret = tx_signalr_ret.clone();
                            async move {
                                let project = match cx.argument::<String>(0) {
                                    Ok(project) => project,
                                    Err(e) => {
                                        error!("Unable to parse SignalR message: {:?}", e);
                                        return;
                                    }
                                };
                                let subproject = match cx.argument::<String>(1) {
                                    Ok(subproject) => subproject,
                                    Err(e) => {
                                        error!("Unable to parse SignalR message: {:?}", e);
                                        return;
                                    }
                                };
                                let language = match cx.argument::<String>(2) {
                                    Ok(language) => language,
                                    Err(e) => {
                                        error!("Unable to parse SignalR message: {:?}", e);
                                        return;
                                    }
                                };
                                let data = match cx
                                    .argument::<HashMap<String, Vec<ParlanceEntryTranslation>>>(4)
                                {
                                    Ok(data) => data,
                                    Err(e) => {
                                        error!("Unable to parse SignalR message: {:?}", e);
                                        return;
                                    }
                                };

                                let _ = tx_signalr_ret
                                    .send(SignalRHubMessage::TranslationUpdated {
                                        project,
                                        subproject,
                                        language,
                                        data,
                                    })
                                    .await;
                            }
                        });
                    }
                });

                while let Some(message) = rx_signalr.recv().await {
                    match message {
                        SignalRMessage::Subscribe {
                            project,
                            subproject,
                            language,
                        } => {
                            if let Err(e) = signalr_client
                                .invoke_with_args::<String, _>(
                                    "Subscribe".into(),
                                    |c: &mut ArgumentConfiguration| {
                                        c.argument(project.clone())
                                            .argument(subproject.clone())
                                            .argument(language.clone());
                                    },
                                )
                                .await
                            {
                                error!("Unable to subscribe to the project on SignalR: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        for language in response.available_languages {
            // Subscribe to the SignalR endpoint
            if let Err(e) = tx_signalr
                .send(SignalRMessage::Subscribe {
                    project: project.clone(),
                    subproject: subproject.clone(),
                    language: language.language.clone(),
                })
                .await
            {
                error!("Unable to subscribe to the project on SignalR: {:?}", e);
            }
        }

        let entries = Arc::new(RwLock::new(entries));
        tokio::spawn({
            let project = project.clone();
            let subproject = subproject.clone();
            let weak_entries = Arc::downgrade(&entries);
            async move {
                while let Some(message) = rx_signalr_ret.recv().await {
                    match message {
                        SignalRHubMessage::TranslationUpdated {
                            project: signalr_project,
                            subproject: signalr_subproject,
                            language,
                            data,
                        } => {
                            if signalr_project != project || signalr_subproject != subproject {
                                // This message is not for us
                                return;
                            }

                            let Some(entries) = weak_entries.upgrade() else {
                                return;
                            };

                            let mut entries = entries.write().unwrap();
                            let language_entries = entries.entry(language.clone()).or_default();
                            for (key, translation) in data {
                                let Some(existing_entry) = language_entries.get(&key) else {
                                    // Can't update a key that doesn't already exist
                                    continue;
                                };

                                let Some(new_entry) = entry_translations_to_i18n_entry(
                                    &key,
                                    existing_entry.is_plural(),
                                    language.clone().into(),
                                    &translation,
                                ) else {
                                    continue;
                                };

                                let boxed = Box::new(new_entry);
                                language_entries
                                    .insert(key.clone(), Box::leak(boxed) as &'static I18nEntry);

                                I18N_MANAGER.evict_key(&key);

                                info!("Translation updated: {} {}", language, key);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            base_url,
            project,
            subproject,
            crate_name,
            entries,
        })
    }
}

impl<'a> ParlanceEntry {
    pub fn to_i18n_entry(&self, locale: I18nString) -> Option<I18nEntry<'a>> {
        entry_translations_to_i18n_entry(
            self.key.as_str(),
            self.requires_pluralisation,
            locale,
            &self.translation,
        )
    }
}

fn entry_translations_to_i18n_entry<'a>(
    key: &str,
    requires_pluralisation: bool,
    locale: I18nString,
    translations: &Vec<ParlanceEntryTranslation>,
) -> Option<I18nEntry<'a>> {
    // TODO: Don't leak memory
    if requires_pluralisation {
        let parts_for_plural = |plural_type: &str| -> Option<&'a [I18nStringPart]> {
            let p = parse_raw_string(
                &translations
                    .iter()
                    .find(|t| t.plural_type == plural_type)?
                    .translation_content,
            )
            .iter()
            .map(|part| part.calculate_string_part(key))
            .collect::<Vec<_>>();

            Some(p.leak())
        };

        let zero = parts_for_plural("zero");
        let one = parts_for_plural("one");
        let two = parts_for_plural("two");
        let few = parts_for_plural("few");
        let many = parts_for_plural("many");
        let other = parts_for_plural("other")?;

        let plural_entry = I18nPluralStringEntry {
            locale,
            zero,
            one,
            two,
            few,
            many,
            other,
        };

        Some(I18nEntry::PluralEntry(plural_entry))
    } else {
        let parts = parse_raw_string(&translations.first()?.translation_content)
            .iter()
            .map(|part| part.calculate_string_part(key))
            .collect::<Vec<_>>();
        Some(I18nEntry::Entry(parts.leak()))
    }
}

impl I18nSource for CntpI18nParlanceSource {
    fn lookup(
        &'_ self,
        locale: &Locale,
        id: &str,
        lookup_crate: &str,
    ) -> Option<&'_ I18nEntry<'_>> {
        if self.crate_name != lookup_crate {
            return None;
        }

        let entries = self.entries.read().unwrap();
        for locale in &locale.messages {
            let Some(entries) = entries.get(locale) else {
                continue;
            };

            for (key, entry) in entries {
                if key == id {
                    return Some(entry);
                }
            }
        }
        None
    }
}

pub async fn install_cntp_i18n_parlance_source(
    base_url: Url,
    project: impl Into<String>,
    subproject: impl Into<String>,
    crate_name: impl Into<String>,
) -> Result<(), ParlanceSourceError> {
    let source = CntpI18nParlanceSource::new(
        base_url,
        project.into(),
        subproject.into(),
        crate_name.into(),
    )
    .await?;
    I18N_MANAGER.load_source(source);
    Ok(())
}
