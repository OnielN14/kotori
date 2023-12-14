use rand::prelude::*;
use reqwest::{
    self,
    header::{
        HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, DNT, ORIGIN, REFERER,
        UPGRADE_INSECURE_REQUESTS, USER_AGENT,
    },
};
use serde_json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let http_client = reqwest::Client::builder().build()?;
    let mut rng = thread_rng();

    let default_user_backend = "https://translate.google.com";
    let endpoint = format!("{default_user_backend}/_/TranslateWebserverUi/data/batchexecute");
    let accept_languages = [None, Some("en-US,en;q=0.9"), Some("en-US"), Some("en")];
    let accept_language = accept_languages[rng.gen_range(0..3)];

    let fsid = long_random(&mut rng);
    let reset_after = rng.gen_range(75..=125);
    let req_id = rng.gen_range(1..=100_000);
    let translate_rpc_id = "MkEWBc";
    let version = "boq_translate-webserver_20210323.10_p0";
    let use_simplest_suggestion = false;

    let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let html = http_client
        .get(default_user_backend)
        .header(USER_AGENT, user_agent)
        .header(UPGRADE_INSECURE_REQUESTS, 1)
        .send()
        .await?
        .text()
        .await?;

    let lookup = "FdrFJe\":\"".to_owned();

    let initial_index = html.find(&lookup).unwrap();
    let start_index = initial_index + lookup.len();
    let end_index = &html[start_index..].find('"').unwrap() + start_index;
    let parsed_fsid = &html[start_index..end_index];

    let untranslated_text = "Testing".to_owned();
    let query = [
        format!("rpcids={}", translate_rpc_id),
        format!("f.sid={}", parsed_fsid),
        format!("bl={}", version),
        "hl=en-US".to_owned(),
        "soc-app=1".to_owned(),
        "soc-platform=1".to_owned(),
        "soc-device=1".to_owned(),
        format!("_reqid={}", req_id),
        "rt=c".to_owned(),
    ]
    .join("&");

    let payload_string = create_payload_string(translate_rpc_id, untranslated_text, "en", "ja");
    let endpoint_with_query = format!("{}?{}", endpoint, query);

    let mut headers = HeaderMap::new();

    headers.append(
        REFERER,
        format!("{}/", default_user_backend).parse().unwrap(),
    );
    headers.append("X-Same-Domain", "1".parse().unwrap());
    headers.append(DNT, "1".parse().unwrap());
    headers.append(
        CONTENT_TYPE,
        "application/x-www-form-urlencoded;charset=UTF-8"
            .parse()
            .unwrap(),
    );
    headers.append(ACCEPT, "*/*".parse().unwrap());
    headers.append(ORIGIN, default_user_backend.parse().unwrap());

    if let Some(value) = accept_language {
        headers.append(ACCEPT_LANGUAGE, value.parse().unwrap());
    }

    let response = http_client
        .post(endpoint_with_query)
        .body(payload_string)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    unwrap_response(&response)?;
    print!("{}", response);

    Ok(())
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

    format!("f.req={}&", payload)
}

fn long_random(rng: &mut ThreadRng) -> i64 {
    let random_value: i64 = rng.gen_range(i64::MIN..i64::MAX);
    random_value.abs()
}

fn unwrap_response(response: &String) -> anyhow::Result<String> {
    let cleaned_text = &response[6..];
    let cleaned_text = &cleaned_text[0..cleaned_text.find('\n').unwrap()];

    let outer_array: serde_json::Value = serde_json::from_str(cleaned_text)?;
    let inner_json_string = outer_array
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap()
        .get(2)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let inner_json_str = &inner_json_string[0..inner_json_string.len()];

    let arr: serde_json::Value = serde_json::from_str(&inner_json_str)?;

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
