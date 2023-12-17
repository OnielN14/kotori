pub mod google_translate;
pub mod google_translate_m;

pub fn default_user_agent(user_agent: Option<String>) -> String {
    user_agent.unwrap_or("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_owned())
}
