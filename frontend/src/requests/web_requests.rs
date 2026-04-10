fn origin() -> String {
    web_sys::window().unwrap().location().origin().unwrap()
}

pub async fn get(path: &str) -> reqwest::Result<reqwest::Response> {
    reqwest::get(format!("{}{}", origin(), path)).await
}

pub async fn post(path: &str, body: serde_json::Value) -> reqwest::Result<reqwest::Response> {
    reqwest::Client::new()
        .post(format!("{}{}", origin(), path))
        .json(&body)
        .send()
        .await
}
