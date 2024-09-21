use crate::db::{Database, ListingDetails, SearchResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Service {
    database: Arc<Mutex<Database>>,
}

impl Service {
    pub fn from_file(filename: &str) -> Result<Self> {
        let database = Arc::new(Mutex::new(Database::in_file(filename)?));
        Ok(Self { database })
    }

    pub async fn get_database(&self) -> Arc<Mutex<Database>> {
        self.database.clone()
    }

    pub async fn get_listing(&self, id: i64) -> Result<ListingDetails> {
        let db = self.database.lock().await;
        db.get_listing_by_id(id)
    }

    pub async fn search_similar(&self, id: i64, limit: i64) -> Result<Vec<SearchResult>> {
        let db = self.database.lock().await;
        db.get_similar_to_listing(id, limit)
    }
}

