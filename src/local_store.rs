use rusqlite;

pub struct LocalStore {
    conn: rusqlite::Connection,
}

impl LocalStore {
    pub fn connect(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
