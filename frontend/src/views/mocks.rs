use crate::views::mock_config_modal::MockConfigModal;
use dioxus::prelude::*;

const MOCKS_CSS: Asset = asset!("/assets/styling/mocks.css");

fn format_condition(cond: &libcommon::MockCondition) -> String {
    let source = match &cond.source {
        libcommon::ConditionSource::PathParam => "path",
        libcommon::ConditionSource::QueryParam => "query",
        libcommon::ConditionSource::Header => "header",
    };
    let matcher = match &cond.matcher {
        libcommon::ConditionMatcher::Exact(v) => format!("= \"{v}\""),
        libcommon::ConditionMatcher::Regex(v) => format!("~ /{v}/"),
    };
    format!("{source}:{} {matcher}", cond.key)
}

#[component]
pub fn Mocks() -> Element {
    let mut mocks = use_resource(crate::requests::ui_requests::get_mocks);
    let mut duplicating = use_signal(|| Option::<libcommon::MockConfig>::None);

    rsx! {
        document::Stylesheet { href: MOCKS_CSS }
        div { "Configured mocks" }
        match &*mocks.read_unchecked() {
            Some(Ok(mocks_list)) => rsx! {
                if mocks_list.is_empty() {
                    p { class: "mocks-empty", "No mocks configured yet." }
                }
                for mock in mocks_list {
                    MockEntry { mock: mock.clone(), duplicating }
                }
            },
            Some(Err(e)) => rsx! {
                p { "Loading mocks failed, {e}" }
            },
            None => rsx! {
                p { "Loading..." }
            },
        }
        if let Some(mock) = duplicating() {
            MockConfigModal {
                method: mock.method.clone(),
                path: mock.path_pattern.clone(),
                prefill: mock.clone(),
                on_close: move |_| {
                    *duplicating.write() = None;
                    mocks.restart();
                },
            }
        }
    }
}

#[component]
fn MockEntry(
    mock: libcommon::MockConfig,
    duplicating: Signal<Option<libcommon::MockConfig>>,
) -> Element {
    let has_conditions = !mock.conditions.is_empty();
    rsx! {
        div { class: "mock-entry",
            div { class: "mock-entry-header",
                span { class: "entry-method", "{mock.method}" }
                span { class: "entry-path",
                    for seg in mock.path_pattern.split('/').filter(|s| !s.is_empty()) {
                        span { class: "path-sep", "/" }
                        span { class: if seg.starts_with(':') { "path-part param" } else { "path-part" },
                            "{seg}"
                        }
                    }
                }
                span { class: "mock-status", "→ {mock.status}" }
                button {
                    class: "duplicate-btn",
                    onclick: move |_| *duplicating.write() = Some(mock.clone()),
                    "Duplicate"
                }
            }
            if has_conditions {
                div { class: "mock-conditions",
                    for cond in &mock.conditions {
                        span { class: "mock-condition-badge", "{format_condition(cond)}" }
                    }
                }
            }
            pre { class: "mock-body", "{mock.body}" }
        }
    }
}
