use std::process::Child;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub struct BackendProcess(pub Mutex<Option<Child>>);

impl Drop for BackendProcess {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.0.lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

pub struct AppState {
    pub jwt: Mutex<Option<String>>,
    pub user: Mutex<Option<UserInfo>>,
    pub db_path: String,
    pub backend_url: String,
}
