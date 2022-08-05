pub use crate::model::session_store::SessionStore;

#[derive(Default)]
pub struct Session {
    pub store: SessionStore,
}

impl Session {
    pub fn send() {

    }
}
