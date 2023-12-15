use std::collections::HashMap;

use rand::prelude::*;
use reqwest::{
    self,
    header::{
        HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, DNT, ORIGIN, REFERER,
        UPGRADE_INSECURE_REQUESTS,
    },
};
use serde_json;

struct EndpointConfig<'a> {
    user_backend: &'a str,
    endpoint: String,
    accept_language: Option<&'a str>,

    req_id: i32,
    translate_rpc_id: String,
    version: String,
}

impl<'a> EndpointConfig<'a> {
    pub fn new() -> Self {
        let default_user_backend = "https://translate.google.com";
        let accept_languages = [None, Some("en-US,en;q=0.9"), Some("en-US"), Some("en")];

        let mut rng = thread_rng();

        Self {
            user_backend: default_user_backend,
            endpoint: format!("{default_user_backend}/_/TranslateWebserverUi/data/batchexecute"),
            accept_language: accept_languages[rng.gen_range(0..3)],
            req_id: rng.gen_range(1..=100_000),
            translate_rpc_id: "MkEWBc".to_owned(),
            version: "boq_translate-webserver_20231212.05_p0".to_owned(),
        }
    }

    pub fn build_query_str(&self, fsid: &str) -> String {
        let query = [
            format!("rpcids={}", self.translate_rpc_id),
            format!("f.sid={}", fsid),
            format!("bl={}", self.version),
            "hl=en-US".to_owned(),
            "soc-app=1".to_owned(),
            "soc-platform=1".to_owned(),
            "soc-device=1".to_owned(),
            format!("_reqid={}", self.req_id),
            "rt=c".to_owned(),
        ]
        .join("&");

        query
    }

    pub fn build_endpoint(&self, fsid: &str) -> String {
        format!("{}?{}", self.endpoint, self.build_query_str(fsid))
    }

    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.append(REFERER, format!("{}/", self.user_backend).parse().unwrap());
        headers.append("X-Same-Domain", "1".parse().unwrap());
        headers.append(DNT, "1".parse().unwrap());
        headers.append(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded;charset=UTF-8"
                .parse()
                .unwrap(),
        );
        headers.append(ACCEPT, "*/*".parse().unwrap());
        headers.append(ORIGIN, self.user_backend.parse().unwrap());

        if let Some(value) = self.accept_language {
            headers.append(ACCEPT_LANGUAGE, value.parse().unwrap());
        }

        headers
    }
}

pub async fn translate(
    untranslated_text: String,
    source_lang: &str,
    destination_lang: &str,
    user_agent: Option<String>,
    http_client: Option<&reqwest::Client>,
) -> anyhow::Result<String> {
    let user_agent = resolve_user_agent(user_agent);
    let default_http_client = reqwest::Client::builder()
        .cookie_store(true)
        .user_agent(user_agent)
        .build()?;

    let http_client = http_client.unwrap_or(&default_http_client);

    let endpoint_config = EndpointConfig::new();

    let response = http_client
        .get(endpoint_config.user_backend)
        .header(UPGRADE_INSECURE_REQUESTS, 1)
        .send()
        .await?;

    let html = response.text().await?;
    let parsed_fsid = acquire_fsid_from_html(&html);

    let payload_string = create_payload_string(
        &endpoint_config.translate_rpc_id,
        untranslated_text,
        source_lang,
        destination_lang,
    );
    let endpoint_with_query = endpoint_config.build_endpoint(parsed_fsid);

    let headers = endpoint_config.build_headers();

    let raw_response = http_client
        .post(endpoint_with_query)
        .body(payload_string)
        .headers(headers)
        .send()
        .await?;

    let response = raw_response.text().await?;
    let result = unwrap_response(&response)?;

    Ok(result)
}

pub mod blocking {
    use crate::UPGRADE_INSECURE_REQUESTS;
    use crate::{
        acquire_fsid_from_html, create_payload_string, resolve_user_agent, unwrap_response,
        EndpointConfig,
    };

    pub fn translate(
        untranslated_text: String,
        source_lang: &str,
        destination_lang: &str,
        user_agent: Option<String>,
        http_client: Option<&reqwest::blocking::Client>,
    ) -> anyhow::Result<String> {
        let user_agent = resolve_user_agent(user_agent);
        let default_http_client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .user_agent(user_agent)
            .build()?;

        let http_client = http_client.unwrap_or(&default_http_client);

        let endpoint_config = EndpointConfig::new();

        let response = http_client
            .get(endpoint_config.user_backend)
            .header(UPGRADE_INSECURE_REQUESTS, 1)
            .send()?;

        let html = response.text()?;
        let parsed_fsid = acquire_fsid_from_html(&html);

        let payload_string = create_payload_string(
            &endpoint_config.translate_rpc_id,
            untranslated_text,
            source_lang,
            destination_lang,
        );
        let endpoint_with_query = endpoint_config.build_endpoint(parsed_fsid);

        let headers = endpoint_config.build_headers();

        let raw_response = http_client
            .post(endpoint_with_query)
            .body(payload_string)
            .headers(headers)
            .send()?;

        let response = raw_response.text()?;
        let result = unwrap_response(&response)?;

        Ok(result)
    }
}

fn resolve_user_agent(user_agent: Option<String>) -> String {
    user_agent.unwrap_or("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_owned())
}

fn acquire_fsid_from_html<'a>(html: &'a String) -> &'a str {
    let lookup = "FdrFJe\":\"".to_owned();
    let initial_index = html.find(&lookup).unwrap();
    let start_index = initial_index + lookup.len();
    let end_index = &html[start_index..].find('"').unwrap() + start_index;
    let parsed_fsid = &html[start_index..end_index];

    parsed_fsid
}

fn create_payload_string(
    rpc: &str,
    untranslated_text: String,
    source_lang: &str,
    destination_lang: &str,
) -> String {
    let payload = format!(
        "[[[\"{}\",\"[[\\\"{}\\\",\\\"{}\\\",\\\"{}\\\",true],[null]]\",null,\"generic\"]]]",
        rpc, untranslated_text, source_lang, destination_lang
    );
    let map = HashMap::from([("f.req", payload)]);

    serde_urlencoded::to_string(map).unwrap()
}

fn unwrap_response(response: &String) -> anyhow::Result<String> {
    let first_lookup = "[";
    let second_lookup = "\n";
    let first_lookup_index = response.find(first_lookup).unwrap();
    let cleaned_text = &response[first_lookup_index..];

    let second_lookup_index = cleaned_text.find(second_lookup).unwrap();
    let cleaned_text = &cleaned_text[0..second_lookup_index];

    let outer_array: serde_json::Value = serde_json::from_str(cleaned_text)?;
    let inner_json_str = outer_array
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap()
        .get(2)
        .unwrap()
        .as_str()
        .unwrap();

    let arr: serde_json::Value = serde_json::from_str(inner_json_str)?;

    /* Ignored other suggestions. Only pick first result */
    let translation_result = arr
        .as_array()
        .unwrap()
        .get(1)
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap()
        .get(5)
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();

    Ok(translation_result)
}
