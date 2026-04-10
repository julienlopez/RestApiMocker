use std::sync::{Arc, Mutex};
use libcommon::{MockConfig, RequestRecord};

pub type RequestHistory = Arc<Mutex<Vec<RequestRecord>>>;
pub type MockRegistry = Arc<Mutex<Vec<MockConfig>>>;