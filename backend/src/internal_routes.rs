use crate::types::{MockRegistry, RequestHistory};
use libcommon::{Config, RequestRecord};
use rocket::State;
use rocket::delete;
use rocket::get;
use rocket::http::Status;
use rocket::post;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// Get the server configuration (public and private ports).
#[openapi(tag = "Internal")]
#[get("/config")]
pub async fn get_config(config: &State<Config>) -> Json<Config> {
    Json(config.inner().clone())
}

/// Get the full list of recorded requests.
#[openapi(tag = "Internal")]
#[get("/history")]
pub async fn get_history(history: &State<RequestHistory>) -> Json<Vec<RequestRecord>> {
    Json(history.lock().unwrap().clone())
}

/// Get all configured mock responses.
#[openapi(tag = "Internal")]
#[get("/mocks")]
pub async fn get_mocks(registry: &State<MockRegistry>) -> Json<Vec<libcommon::MockConfig>> {
    Json(registry.lock().unwrap().clone())
}

/// Register a new mock response for a given method and path pattern.
#[openapi(tag = "Internal")]
#[post("/mock", data = "<mock>")]
pub async fn set_mock(
    mock: Json<libcommon::MockConfig>,
    registry: &State<MockRegistry>,
) -> &'static str {
    registry.lock().unwrap().push(mock.into_inner());
    "Mock configured"
}

/// Delete a mock by its index in the list (0-based).
#[openapi(tag = "Internal")]
#[delete("/mock/<index>")]
pub async fn delete_mock(index: usize, registry: &State<MockRegistry>) -> Custom<String> {
    let mut mocks = registry.lock().unwrap();
    if index >= mocks.len() {
        return Custom(
            Status::NotFound,
            format!("Mock index {index} out of range (have {})", mocks.len()),
        );
    }
    mocks.remove(index);
    Custom(Status::Ok, "Mock deleted".to_string())
}

/// Delete all configured mocks.
#[openapi(tag = "Internal")]
#[delete("/mocks")]
pub async fn delete_all_mocks(registry: &State<MockRegistry>) -> &'static str {
    registry.lock().unwrap().clear();
    "All mocks deleted"
}

/// Delete all mocks whose path pattern matches the given value.
///
/// Example: `DELETE /internal/mocks/by-pattern?path_pattern=/users/:id`
#[openapi(tag = "Internal")]
#[delete("/mocks/by-pattern?<path_pattern>")]
pub async fn delete_mocks_by_pattern(
    path_pattern: String,
    registry: &State<MockRegistry>,
) -> Custom<String> {
    let mut mocks = registry.lock().unwrap();
    let before = mocks.len();
    mocks.retain(|m| m.path_pattern != path_pattern);
    let removed = before - mocks.len();
    if removed == 0 {
        Custom(
            Status::NotFound,
            format!("No mocks found with path pattern '{path_pattern}'"),
        )
    } else {
        Custom(Status::Ok, format!("{removed} mock(s) deleted"))
    }
}
