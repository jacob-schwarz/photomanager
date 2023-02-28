#[derive(Debug)]
pub struct Photo {
    pub id: i64,
    pub path: String,
    pub hash: String,
}

impl Photo {
    pub fn new(id: i64, path: String, hash: String) -> Self {
        Self {
            id,
            path,
            hash,
        }
    }
}
