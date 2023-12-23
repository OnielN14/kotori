use std::collections::HashMap;

use crate::default_user_agent;
use html_escape;
use reqwest::header::USER_AGENT;
use serde_urlencoded;

struct EndpointConfig<'a> {
    backend: &'a str,
}

impl<'a> EndpointConfig<'a> {
    fn new() -> Self {
        Self {
            backend: "https://translate.google.com/m",
        }
    }

    fn build_endpoint(
        &self,
        untranslated_text: &'a str,
        source_lang: &'a str,
        destination_lang: &'a str,
    ) -> String {
        let query_map = HashMap::from([
            ("sl", source_lang),
            ("tl", destination_lang),
            ("q", untranslated_text),
        ]);
        let query = serde_urlencoded::to_string(query_map).unwrap();

        format!("{}?{}", self.backend, query)
    }
}

pub async fn translate(
    untranslated_text: String,
    source_lang: &str,
    destination_lang: &str,
    user_agent: Option<String>,
    http_client: Option<&reqwest::Client>,
) -> anyhow::Result<String> {
    let default_http_client = reqwest::Client::builder().build()?;
    let http_client = http_client.unwrap_or(&default_http_client);
    let user_agent = default_user_agent(user_agent);

    let endpoint =
        EndpointConfig::new().build_endpoint(&untranslated_text, source_lang, destination_lang);

    let html = http_client
        .get(endpoint)
        .header(USER_AGENT, user_agent)
        .send()
        .await?
        .text()
        .await?;

    let cleaned = if let Some(value) = get_value(&html) {
        html_escape::decode_html_entities(value).to_string()
    } else {
        return Err(anyhow::anyhow!("Unable to parse value"));
    };

    Ok(cleaned)
}

#[cfg(feature = "blocking")]
pub mod blocking {
    use super::{default_user_agent, get_value, html_escape, EndpointConfig, USER_AGENT};

    pub fn translate(
        untranslated_text: String,
        source_lang: &str,
        destination_lang: &str,
        user_agent: Option<String>,
        http_client: Option<&reqwest::blocking::Client>,
    ) -> anyhow::Result<String> {
        let default_http_client = reqwest::blocking::Client::builder().build()?;
        let http_client = http_client.unwrap_or(&default_http_client);
        let user_agent = default_user_agent(user_agent);

        let endpoint =
            EndpointConfig::new().build_endpoint(&untranslated_text, source_lang, destination_lang);

        let html = http_client
            .get(endpoint)
            .header(USER_AGENT, user_agent)
            .send()?
            .text()?;

        let cleaned = if let Some(value) = get_value(&html) {
            html_escape::decode_html_entities(value).to_string()
        } else {
            return Err(anyhow::anyhow!("Unable to parse value"));
        };

        Ok(cleaned.to_owned())
    }
}

fn get_value(html: &str) -> Option<&str> {
    let lookup = r#"<div class="result-container">"#;
    let start_index = html.find(lookup)? + lookup.len();
    let cleaned = &html[start_index..];
    let end_index = cleaned.find("</div>")?;
    let cleaned = &cleaned[..end_index];
    Some(cleaned)
}
