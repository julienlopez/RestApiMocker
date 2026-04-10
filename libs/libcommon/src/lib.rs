use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub public_port: u16,
    pub private_port: u16,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RequestRecord {
    pub method: String,
    pub path: String,
    pub timestamp: u64,
}

/// Where to look for the value to match against.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ConditionSource {
    /// Value extracted from a `:param` segment in the path pattern.
    PathParam,
    /// Value from the query string (`?key=value`).
    QueryParam,
    /// Value from a request header.
    Header,
}

/// How to compare the extracted value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ConditionMatcher {
    /// Must be exactly equal (case-sensitive).
    Exact(String),
    /// Must match this regular expression.
    Regex(String),
}

/// A single condition that must be satisfied for a mock to match.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MockCondition {
    /// Where to extract the value from.
    pub source: ConditionSource,
    /// The name of the parameter / header.
    pub key: String,
    /// How to compare the extracted value.
    pub matcher: ConditionMatcher,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MockConfig {
    pub method: String,
    /// Path pattern where segments starting with `:` are wildcards, e.g. `/users/:id/profile`
    pub path_pattern: String,
    pub status: u16,
    pub body: String,
    /// Optional conditions that must ALL match for this mock to be selected.
    /// Mocks with more conditions are evaluated first (most-specific wins).
    /// An empty vec (the default) means the mock matches unconditionally.
    #[serde(default)]
    pub conditions: Vec<MockCondition>,
}
