use crate::db::{Database, ListingDetails, SearchResult};
use anyhow::Result;

pub struct Service {
    database: Database,
}

impl Service {
    pub fn from_file(filename: &str) -> Result<Self> {
        Ok(Self { database: Database::in_file(filename)? })
    }

    pub async fn get_database(&self) -> Database {
        self.database.clone()
    }

    pub async fn get_listing(&self, id: u64) -> Result<ListingDetails> {
        let db = self.database.clone();
        tokio::task::spawn_blocking(move || db.get_listing_by_id(id)).await?
    }

    pub async fn search_similar(&self, id: u64, limit: u64) -> Result<Vec<SearchResult>> {
        let db = self.database.clone();
        tokio::task::spawn_blocking(move || db.get_similar_to_listing(id, limit)).await?
    }
}

