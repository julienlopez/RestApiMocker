use libcommon::{MockConfig, RequestRecord};
use std::sync::{Arc, Mutex};

pub type RequestHistory = Arc<Mutex<Vec<RequestRecord>>>;
pub type MockRegistry = Arc<Mutex<Vec<MockConfig>>>;
