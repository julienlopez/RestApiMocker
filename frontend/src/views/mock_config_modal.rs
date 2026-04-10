use dioxus::prelude::*;

/// State for a single condition row in the editor.
#[derive(Clone, Debug)]
struct ConditionRow {
    source: String, // "PathParam", "QueryParam", "Header"
    key: String,
    matcher_type: String, // "Exact", "Regex"
    matcher_value: String,
}

impl ConditionRow {
    fn new() -> Self {
        Self {
            source: "QueryParam".to_string(),
            key: String::new(),
            matcher_type: "Exact".to_string(),
            matcher_value: String::new(),
        }
    }

    fn from_condition(cond: &libcommon::MockCondition) -> Self {
        let source = match &cond.source {
            libcommon::ConditionSource::PathParam => "PathParam",
            libcommon::ConditionSource::QueryParam => "QueryParam",
            libcommon::ConditionSource::Header => "Header",
        }
        .to_string();
        let (matcher_type, matcher_value) = match &cond.matcher {
            libcommon::ConditionMatcher::Exact(v) => ("Exact".to_string(), v.clone()),
            libcommon::ConditionMatcher::Regex(v) => ("Regex".to_string(), v.clone()),
        };
        Self {
            source,
            key: cond.key.clone(),
            matcher_type,
            matcher_value,
        }
    }

    fn to_condition(&self) -> libcommon::MockCondition {
        let source = match self.source.as_str() {
            "PathParam" => libcommon::ConditionSource::PathParam,
            "Header" => libcommon::ConditionSource::Header,
            _ => libcommon::ConditionSource::QueryParam,
        };
        let matcher = match self.matcher_type.as_str() {
            "Regex" => libcommon::ConditionMatcher::Regex(self.matcher_value.clone()),
            _ => libcommon::ConditionMatcher::Exact(self.matcher_value.clone()),
        };
        libcommon::MockCondition {
            source,
            key: self.key.clone(),
            matcher,
        }
    }
}

/// Props for MockConfigModal.
///
/// - `method` + `path`: always required to display the target.
/// - `prefill`: when provided, pre-populates path param flags, status, body and conditions
///   from an existing `MockConfig` (used for duplication). When `None`, starts with defaults.
#[component]
pub fn MockConfigModal(
    method: String,
    path: String,
    #[props(default)] prefill: Option<libcommon::MockConfig>,
    on_close: EventHandler<()>,
) -> Element {
    let segments: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let n = segments.len();

    // Derive initial param flags from the prefill pattern (if any).
    let initial_flags: Vec<bool> = if let Some(ref pf) = prefill {
        let pattern_segs: Vec<&str> = pf
            .path_pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        segments
            .iter()
            .enumerate()
            .map(|(i, _)| pattern_segs.get(i).is_some_and(|s| s.starts_with(':')))
            .collect()
    } else {
        vec![false; n]
    };

    let initial_status = prefill
        .as_ref()
        .map(|pf| pf.status.to_string())
        .unwrap_or_else(|| "200".to_string());
    let initial_body = prefill
        .as_ref()
        .map(|pf| pf.body.clone())
        .unwrap_or_default();
    let initial_conditions: Vec<ConditionRow> = prefill
        .as_ref()
        .map(|pf| pf.conditions.iter().map(ConditionRow::from_condition).collect())
        .unwrap_or_default();

    let mut param_flags = use_signal(|| initial_flags);
    let mut status_str = use_signal(|| initial_status);
    let mut body = use_signal(|| initial_body);
    let mut error = use_signal(|| Option::<String>::None);
    let mut conditions = use_signal(|| initial_conditions);

    let segs_for_pattern = segments.clone();

    // If there's a prefill, preserve the original param names from the pattern.
    let prefill_pattern_segs: Vec<String> = prefill
        .as_ref()
        .map(|pf| {
            pf.path_pattern
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let path_pattern = use_memo(move || {
        let flags = param_flags.read();
        let parts: Vec<String> = segs_for_pattern
            .iter()
            .zip(flags.iter())
            .enumerate()
            .map(|(i, (s, is_param))| {
                if *is_param {
                    // Reuse original param name from prefill if available.
                    prefill_pattern_segs
                        .get(i)
                        .filter(|p| p.starts_with(':'))
                        .cloned()
                        .unwrap_or_else(|| format!(":param{i}"))
                } else {
                    s.clone()
                }
            })
            .collect();
        format!("/{}", parts.join("/"))
    });

    let method_clone = method.clone();

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal-content",
                onclick: move |e| e.stop_propagation(),

                h2 { "Configure Mock" }
                p { class: "mock-target", "{method} {path}" }

                div { class: "path-segments-label", "Click segments to mark as parameters:" }
                div { class: "path-segments",
                    for (i , seg) in segments.iter().cloned().enumerate() {
                        span {
                            class: if param_flags.read()[i] { "path-segment param" } else { "path-segment" },
                            onclick: move |_| param_flags.with_mut(|f| f[i] = !f[i]),
                            "{seg}"
                        }
                    }
                }

                p { class: "pattern-preview", "Pattern: {path_pattern}" }

                label { r#for: "status-input", "Status code" }
                input {
                    id: "status-input",
                    r#type: "number",
                    value: "{status_str}",
                    oninput: move |e| *status_str.write() = e.value(),
                }

                label { r#for: "body-input", "Response body" }
                textarea {
                    id: "body-input",
                    value: "{body}",
                    oninput: move |e| *body.write() = e.value(),
                }

                // ── Conditions section ──
                div { class: "conditions-section",
                    div { class: "conditions-header",
                        span { class: "conditions-label", "Conditions" }
                        button {
                            class: "add-condition-btn",
                            onclick: move |_| conditions.with_mut(|c| c.push(ConditionRow::new())),
                            "+ Add condition"
                        }
                    }
                    if conditions.read().is_empty() {
                        p { class: "conditions-hint",
                            "No conditions — this mock will match any request on the pattern."
                        }
                    }
                    for (i , _row) in conditions.read().iter().enumerate() {
                        ConditionEditor { key: "{i}", index: i, conditions }
                    }
                }

                if let Some(err) = error() {
                    p { class: "mock-error", "Error: {err}" }
                }

                div { class: "modal-actions",
                    button {
                        class: "cancel-btn",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "save-btn",
                        onclick: move |_| {
                            let built_conditions: Vec<libcommon::MockCondition> = conditions
                                .read()
                                .iter()
                                .filter(|r| !r.key.is_empty())
                                .map(|r| r.to_condition())
                                .collect();
                            let mock = libcommon::MockConfig {
                                method: method_clone.clone(),
                                path_pattern: path_pattern(),
                                status: status_str().trim().parse().unwrap_or(200),
                                body: body(),
                                conditions: built_conditions,
                            };
                            spawn(async move {
                                match crate::requests::ui_requests::set_mock(mock).await {
                                    Ok(()) => on_close.call(()),
                                    Err(e) => *error.write() = Some(e),
                                }
                            });
                        },
                        "Save Mock"
                    }
                }
            }
        }
    }
}

#[component]
fn ConditionEditor(index: usize, conditions: Signal<Vec<ConditionRow>>) -> Element {
    let source = conditions.read()[index].source.clone();
    let key = conditions.read()[index].key.clone();
    let matcher_type = conditions.read()[index].matcher_type.clone();
    let matcher_value = conditions.read()[index].matcher_value.clone();

    rsx! {
        div { class: "condition-row",
            select {
                class: "condition-select source-select",
                value: "{source}",
                onchange: move |e| conditions.with_mut(|c| c[index].source = e.value()),
                option { value: "PathParam", "Path Param" }
                option { value: "QueryParam", "Query Param" }
                option { value: "Header", "Header" }
            }
            input {
                class: "condition-input key-input",
                placeholder: "key",
                value: "{key}",
                oninput: move |e| conditions.with_mut(|c| c[index].key = e.value()),
            }
            select {
                class: "condition-select matcher-select",
                value: "{matcher_type}",
                onchange: move |e| conditions.with_mut(|c| c[index].matcher_type = e.value()),
                option { value: "Exact", "Exact" }
                option { value: "Regex", "Regex" }
            }
            input {
                class: "condition-input value-input",
                placeholder: "value",
                value: "{matcher_value}",
                oninput: move |e| conditions.with_mut(|c| c[index].matcher_value = e.value()),
            }
            button {
                class: "remove-condition-btn",
                onclick: move |_| {
                    conditions
                        .with_mut(|c| {
                            c.remove(index);
                        })
                },
                "×"
            }
        }
    }
}
