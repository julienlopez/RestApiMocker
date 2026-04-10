use crate::views::mock_config_modal::MockConfigModal;
use dioxus::prelude::*;

const HOME_CSS: Asset = asset!("/assets/styling/home.css");

fn mock_matches(mock: &libcommon::MockConfig, entry: &libcommon::RequestRecord) -> bool {
    if !mock.method.eq_ignore_ascii_case(&entry.method) {
        return false;
    }
    let pp: Vec<&str> = mock
        .path_pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let sp: Vec<&str> = entry.path.split('/').filter(|s| !s.is_empty()).collect();
    if pp.len() != sp.len() {
        return false;
    }
    pp.iter()
        .zip(sp.iter())
        .all(|(p, s)| p.starts_with(':') || p == s)
}

#[component]
pub fn Home() -> Element {
    let mut tick = use_signal(|| 0u32);
    let mut configuring = use_signal(|| Option::<libcommon::RequestRecord>::None);

    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
                if configuring.read().is_none() {
                    *tick.write() += 1;
                }
            }
        });
    });

    let history = use_resource(move || async move {
        tick();
        crate::requests::ui_requests::get_history().await
    });
    let mut mocks = use_resource(move || async move {
        tick();
        crate::requests::ui_requests::get_mocks().await
    });

    rsx! {
        document::Stylesheet { href: HOME_CSS }
        div { "Received requests history" }
        match &*history.read_unchecked() {
            Some(Ok(history)) => rsx! {
                for entry in history {
                    HistoryEntry { entry: entry.clone(), mocks, configuring }
                }
            },
            Some(Err(e)) => rsx! {
                p { "Loading request history failed, {e}" }
            },
            None => rsx! {
                p { "Loading..." }
            },
        }
        if let Some(entry) = configuring() {
            MockConfigModal {
                method: entry.method.clone(),
                path: entry.path.clone(),
                on_close: move |_| {
                    *configuring.write() = None;
                    mocks.restart();
                },
            }
        }
    }
}

fn path_segments(
    entry: &libcommon::RequestRecord,
    matched_mock: &Option<libcommon::MockConfig>,
) -> Vec<(String, bool)> {
    let sp: Vec<&str> = entry.path.split('/').filter(|s| !s.is_empty()).collect();
    if let Some(ref mock) = matched_mock {
        let pp: Vec<bool> = mock
            .path_pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|p| p.starts_with(':'))
            .collect();
        sp.iter()
            .zip(pp.iter())
            .map(|(s, is_param)| (s.to_string(), *is_param))
            .collect()
    } else {
        sp.iter().map(|s| (s.to_string(), false)).collect()
    }
}

#[component]
fn HistoryEntry(
    entry: libcommon::RequestRecord,
    mocks: Resource<Result<Vec<libcommon::MockConfig>, String>>,
    configuring: Signal<Option<libcommon::RequestRecord>>,
) -> Element {
    let matched_mock: Option<libcommon::MockConfig> = mocks
        .read_unchecked()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|ms| ms.iter().find(|m| mock_matches(m, &entry)))
        .cloned();
    let path_segments: Vec<(String, bool)> = path_segments(&entry, &matched_mock);
    rsx! {
        div { class: "history-entry",
            span { class: "entry-method", "{entry.method}" }
            span { class: "entry-path",
                for (seg , is_param) in path_segments {
                    span { class: "path-sep", "/" }
                    span { class: if is_param { "path-part param" } else { "path-part" }, "{seg}" }
                }
            }
            span { class: "entry-timestamp", " at {entry.timestamp}" }
            button {
                class: "configure-btn",
                onclick: move |_| *configuring.write() = Some(entry.clone()),
                "Configure Mock"
            }
        }
    }
}
