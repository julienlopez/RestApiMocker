use crate::types::MockRegistry;
use libcommon::{ConditionMatcher, ConditionSource, MockCondition};
use regex::Regex;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::status::Custom;
use std::collections::HashMap;

fn matches_pattern(pattern: &str, path: &str) -> bool {
    let pp: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let sp: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if pp.len() != sp.len() {
        return false;
    }
    pp.iter()
        .zip(sp.iter())
        .all(|(p, s)| p.starts_with(':') || p == s)
}

/// Extract named path parameters from a pattern match.
/// e.g. pattern="/users/:id/posts/:pid", path="/users/42/posts/7" → {"id":"42","pid":"7"}
fn extract_path_params<'a>(pattern: &'a str, path: &'a str) -> HashMap<&'a str, &'a str> {
    let pp: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let sp: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    pp.iter()
        .zip(sp.iter())
        .filter_map(|(p, s)| p.strip_prefix(':').map(|name| (name, *s)))
        .collect()
}

/// Parse query string into a multimap. `?a=1&b=2&a=3` → {"a":["1","3"], "b":["2"]}
fn parse_query_params(query: &str) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for part in query.split('&') {
        if let Some((k, v)) = part.split_once('=') {
            map.entry(k.to_string()).or_default().push(v.to_string());
        }
    }
    map
}

/// Check whether a matcher is satisfied by at least one of the given values.
fn matcher_matches(matcher: &ConditionMatcher, values: &[String]) -> bool {
    if values.is_empty() {
        return false;
    }
    values.iter().any(|v| match matcher {
        ConditionMatcher::Exact(expected) => v == expected,
        ConditionMatcher::Regex(pattern) => Regex::new(pattern)
            .map(|re| re.is_match(v))
            .unwrap_or(false),
    })
}

/// Resolve the values targeted by a condition from the request context.
fn resolve_condition_values(
    cond: &MockCondition,
    path_params: &HashMap<&str, &str>,
    query_params: &HashMap<String, Vec<String>>,
    req: &Request<'_>,
) -> Vec<String> {
    match &cond.source {
        ConditionSource::PathParam => path_params
            .get(cond.key.as_str())
            .map(|v| vec![v.to_string()])
            .unwrap_or_default(),
        ConditionSource::QueryParam => query_params.get(&cond.key).cloned().unwrap_or_default(),
        ConditionSource::Header => req
            .headers()
            .get(&cond.key)
            .map(|v| v.to_string())
            .collect(),
    }
}

/// Check whether a single condition is satisfied.
fn condition_matches(
    cond: &MockCondition,
    path_params: &HashMap<&str, &str>,
    query_params: &HashMap<String, Vec<String>>,
    req: &Request<'_>,
) -> bool {
    let values = resolve_condition_values(cond, path_params, query_params, req);
    matcher_matches(&cond.matcher, &values)
}

#[rocket::catch(default)]
pub fn mock_catcher(status: Status, req: &Request<'_>) -> Custom<String> {
    let path = req.uri().path().as_str().to_string();
    let method = req.method().to_string();
    let query_str = req.uri().query().map(|q| q.as_str()).unwrap_or("");
    let query_params = parse_query_params(query_str);

    if let Some(registry) = req.rocket().state::<MockRegistry>() {
        let registry = registry.lock().unwrap();

        // Sort candidates: more conditions first (most-specific wins).
        let mut candidates: Vec<_> = registry
            .iter()
            .filter(|m| {
                m.method.eq_ignore_ascii_case(&method) && matches_pattern(&m.path_pattern, &path)
            })
            .collect();
        candidates.sort_by(|a, b| b.conditions.len().cmp(&a.conditions.len()));

        for mock in candidates {
            let path_params = extract_path_params(&mock.path_pattern, &path);
            let all_match = mock
                .conditions
                .iter()
                .all(|c| condition_matches(c, &path_params, &query_params, req));
            if all_match {
                let mock_status =
                    Status::from_code(mock.status).unwrap_or(Status::InternalServerError);
                return Custom(mock_status, mock.body.clone());
            }
        }
    }

    Custom(
        status,
        format!("No mock configured for {} {}", method, path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{ConditionMatcher, ConditionSource, MockCondition};

    // ───── matches_pattern ─────

    #[test]
    fn pattern_exact_match() {
        assert!(matches_pattern("/users", "/users"));
    }

    #[test]
    fn pattern_with_wildcard() {
        assert!(matches_pattern("/users/:id", "/users/42"));
    }

    #[test]
    fn pattern_multiple_wildcards() {
        assert!(matches_pattern(
            "/users/:id/posts/:pid",
            "/users/1/posts/99"
        ));
    }

    #[test]
    fn pattern_mismatch_different_segment_count() {
        assert!(!matches_pattern("/users/:id", "/users/42/extra"));
    }

    #[test]
    fn pattern_mismatch_literal_segment() {
        assert!(!matches_pattern("/users/admin", "/users/42"));
    }

    #[test]
    fn pattern_root() {
        assert!(matches_pattern("/", "/"));
    }

    #[test]
    fn pattern_trailing_slash_ignored() {
        // Both split to the same segments because empty strings are filtered.
        assert!(matches_pattern("/users/", "/users"));
    }

    // ───── extract_path_params ─────

    #[test]
    fn extract_single_param() {
        let params = extract_path_params("/users/:id", "/users/42");
        assert_eq!(params.get("id"), Some(&"42"));
    }

    #[test]
    fn extract_multiple_params() {
        let params = extract_path_params("/users/:uid/posts/:pid", "/users/5/posts/17");
        assert_eq!(params.get("uid"), Some(&"5"));
        assert_eq!(params.get("pid"), Some(&"17"));
    }

    #[test]
    fn extract_no_params() {
        let params = extract_path_params("/static/path", "/static/path");
        assert!(params.is_empty());
    }

    // ───── parse_query_params ─────

    #[test]
    fn parse_simple_query() {
        let qp = parse_query_params("a=1&b=2");
        assert_eq!(qp.get("a").unwrap(), &vec!["1".to_string()]);
        assert_eq!(qp.get("b").unwrap(), &vec!["2".to_string()]);
    }

    #[test]
    fn parse_repeated_key() {
        let qp = parse_query_params("x=1&x=2&x=3");
        assert_eq!(
            qp.get("x").unwrap(),
            &vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn parse_empty_query() {
        let qp = parse_query_params("");
        assert!(qp.is_empty());
    }

    // ───── matcher_matches ─────

    #[test]
    fn exact_matcher_hit() {
        let m = ConditionMatcher::Exact("42".into());
        assert!(matcher_matches(&m, &["42".into()]));
    }

    #[test]
    fn exact_matcher_miss() {
        let m = ConditionMatcher::Exact("42".into());
        assert!(!matcher_matches(&m, &["99".into()]));
    }

    #[test]
    fn exact_matcher_case_sensitive() {
        let m = ConditionMatcher::Exact("Hello".into());
        assert!(!matcher_matches(&m, &["hello".into()]));
    }

    #[test]
    fn regex_matcher_hit() {
        let m = ConditionMatcher::Regex("^test".into());
        assert!(matcher_matches(&m, &["testing123".into()]));
    }

    #[test]
    fn regex_matcher_miss() {
        let m = ConditionMatcher::Regex("^test".into());
        assert!(!matcher_matches(&m, &["notest".into()]));
    }

    #[test]
    fn regex_matcher_full_match() {
        let m = ConditionMatcher::Regex(r"^\d+$".into());
        assert!(matcher_matches(&m, &["12345".into()]));
        assert!(!matcher_matches(&m, &["abc".into()]));
    }

    #[test]
    fn regex_matcher_invalid_regex_returns_false() {
        let m = ConditionMatcher::Regex("[".into()); // invalid regex
        assert!(!matcher_matches(&m, &["anything".into()]));
    }

    #[test]
    fn matcher_empty_values_returns_false() {
        let m = ConditionMatcher::Exact("42".into());
        assert!(!matcher_matches(&m, &[]));
    }

    #[test]
    fn matcher_any_value_satisfies() {
        // At least one value must match.
        let m = ConditionMatcher::Exact("b".into());
        assert!(matcher_matches(&m, &["a".into(), "b".into(), "c".into()]));
    }

    // ───── condition evaluation with path / query params ─────

    #[test]
    fn condition_path_param_exact_match() {
        let cond = MockCondition {
            source: ConditionSource::PathParam,
            key: "id".into(),
            matcher: ConditionMatcher::Exact("42".into()),
        };
        let path_params: HashMap<&str, &str> = [("id", "42")].into_iter().collect();
        assert!(matcher_matches(
            &cond.matcher,
            &path_params
                .get(cond.key.as_str())
                .map(|v| vec![v.to_string()])
                .unwrap_or_default(),
        ));

        // Non-matching value
        let path_params2: HashMap<&str, &str> = [("id", "99")].into_iter().collect();
        assert!(!matcher_matches(
            &cond.matcher,
            &path_params2
                .get(cond.key.as_str())
                .map(|v| vec![v.to_string()])
                .unwrap_or_default(),
        ));
    }

    #[test]
    fn condition_path_param_missing_key() {
        let cond = MockCondition {
            source: ConditionSource::PathParam,
            key: "missing".into(),
            matcher: ConditionMatcher::Exact("42".into()),
        };
        let path_params: HashMap<&str, &str> = [("id", "42")].into_iter().collect();
        assert!(!matcher_matches(
            &cond.matcher,
            &path_params
                .get(cond.key.as_str())
                .map(|v| vec![v.to_string()])
                .unwrap_or_default(),
        ));
    }

    #[test]
    fn condition_query_param_exact() {
        let cond = MockCondition {
            source: ConditionSource::QueryParam,
            key: "page".into(),
            matcher: ConditionMatcher::Exact("3".into()),
        };
        let query_params = parse_query_params("page=3&limit=10");
        let values = query_params.get(&cond.key).cloned().unwrap_or_default();
        assert!(matcher_matches(&cond.matcher, &values));
    }

    #[test]
    fn condition_query_param_regex() {
        let cond = MockCondition {
            source: ConditionSource::QueryParam,
            key: "q".into(),
            matcher: ConditionMatcher::Regex("^test".into()),
        };
        let query_params = parse_query_params("q=testing&page=1");
        let values = query_params.get(&cond.key).cloned().unwrap_or_default();
        assert!(matcher_matches(&cond.matcher, &values));
    }

    #[test]
    fn condition_query_param_repeated_any_matches() {
        let cond = MockCondition {
            source: ConditionSource::QueryParam,
            key: "tag".into(),
            matcher: ConditionMatcher::Exact("rust".into()),
        };
        let query_params = parse_query_params("tag=go&tag=rust&tag=python");
        let values = query_params.get(&cond.key).cloned().unwrap_or_default();
        assert!(matcher_matches(&cond.matcher, &values));
    }

    #[test]
    fn condition_query_param_missing_key() {
        let cond = MockCondition {
            source: ConditionSource::QueryParam,
            key: "missing".into(),
            matcher: ConditionMatcher::Exact("any".into()),
        };
        let query_params = parse_query_params("page=1");
        let values = query_params.get(&cond.key).cloned().unwrap_or_default();
        assert!(!matcher_matches(&cond.matcher, &values));
    }

    // ───── integration-style: multiple conditions ─────

    #[test]
    fn all_conditions_must_match() {
        let conditions = [
            MockCondition {
                source: ConditionSource::PathParam,
                key: "id".into(),
                matcher: ConditionMatcher::Exact("42".into()),
            },
            MockCondition {
                source: ConditionSource::QueryParam,
                key: "verbose".into(),
                matcher: ConditionMatcher::Exact("true".into()),
            },
        ];

        let path_params: HashMap<&str, &str> = [("id", "42")].into_iter().collect();
        let query_params = parse_query_params("verbose=true");

        // All match
        let all_pass = conditions.iter().all(|c| {
            let values = match &c.source {
                ConditionSource::PathParam => path_params
                    .get(c.key.as_str())
                    .map(|v| vec![v.to_string()])
                    .unwrap_or_default(),
                ConditionSource::QueryParam => {
                    query_params.get(&c.key).cloned().unwrap_or_default()
                }
                _ => vec![],
            };
            matcher_matches(&c.matcher, &values)
        });
        assert!(all_pass);

        // One fails → overall fails
        let query_params_bad = parse_query_params("verbose=false");
        let all_pass2 = conditions.iter().all(|c| {
            let values = match &c.source {
                ConditionSource::PathParam => path_params
                    .get(c.key.as_str())
                    .map(|v| vec![v.to_string()])
                    .unwrap_or_default(),
                ConditionSource::QueryParam => {
                    query_params_bad.get(&c.key).cloned().unwrap_or_default()
                }
                _ => vec![],
            };
            matcher_matches(&c.matcher, &values)
        });
        assert!(!all_pass2);
    }

    #[test]
    fn empty_conditions_always_match() {
        let conditions: Vec<MockCondition> = vec![];
        let path_params: HashMap<&str, &str> = HashMap::new();
        let query_params: HashMap<String, Vec<String>> = HashMap::new();
        let _ = &query_params; // used below

        let all_pass = conditions.iter().all(|c| {
            let values: Vec<String> = match &c.source {
                ConditionSource::PathParam => path_params
                    .get(c.key.as_str())
                    .map(|v| vec![v.to_string()])
                    .unwrap_or_default(),
                ConditionSource::QueryParam => {
                    query_params.get(&c.key).cloned().unwrap_or_default()
                }
                _ => vec![],
            };
            matcher_matches(&c.matcher, &values)
        });
        assert!(all_pass, "empty conditions should vacuously match");
    }

    // ───── specificity ordering ─────

    #[test]
    fn more_conditions_sorted_first() {
        use libcommon::MockConfig;

        let mocks = [
            MockConfig {
                method: "GET".into(),
                path_pattern: "/users/:id".into(),
                status: 200,
                body: "fallback".into(),
                conditions: vec![],
            },
            MockConfig {
                method: "GET".into(),
                path_pattern: "/users/:id".into(),
                status: 200,
                body: "specific".into(),
                conditions: vec![MockCondition {
                    source: ConditionSource::PathParam,
                    key: "id".into(),
                    matcher: ConditionMatcher::Exact("42".into()),
                }],
            },
            MockConfig {
                method: "GET".into(),
                path_pattern: "/users/:id".into(),
                status: 200,
                body: "most_specific".into(),
                conditions: vec![
                    MockCondition {
                        source: ConditionSource::PathParam,
                        key: "id".into(),
                        matcher: ConditionMatcher::Exact("42".into()),
                    },
                    MockCondition {
                        source: ConditionSource::QueryParam,
                        key: "verbose".into(),
                        matcher: ConditionMatcher::Exact("true".into()),
                    },
                ],
            },
        ];

        let mut candidates: Vec<_> = mocks.iter().collect();
        candidates.sort_by(|a, b| b.conditions.len().cmp(&a.conditions.len()));

        assert_eq!(candidates[0].body, "most_specific");
        assert_eq!(candidates[1].body, "specific");
        assert_eq!(candidates[2].body, "fallback");
    }
}
