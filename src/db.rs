use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Error;
use rusqlite::ffi::sqlite3_auto_extension;
use serde::Serialize;
use sqlite_vec::sqlite3_vec_init;
use std::path::PathBuf;
use std::result;
use zerocopy::AsBytes;


#[derive(Clone)]
pub struct Database {
    connection: Pool<SqliteConnectionManager>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub listing_details: ListingDetails,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct ListingDetails {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub neighborhood_overview: Option<String>,
    pub neighbourhood_cleansed: Option<String>,
    pub property_type: Option<String>,
    pub room_type: Option<String>,
    pub accommodates: Option<i64>,
    pub bathrooms_text: Option<String>,
    pub bedrooms: Option<f64>,
    pub beds: Option<f64>,
    pub amenities: Option<String>,
    pub price: Option<String>,
}

impl Database {
    pub fn in_file(path: &str) -> Result<Self> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let path: PathBuf = path.into();
        let connection = Pool::new(SqliteConnectionManager::file(path))?;
        Ok(Database { connection })
    }

    pub fn in_memory() -> Result<Self> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let connection = Pool::new(SqliteConnectionManager::memory())?;
        Ok(Database { connection })
    }

    pub fn get_listings(&self, limit: i64) -> Result<Vec<ListingDetails>> {
        let conn = self.connection.get()?;
        let mut stmt = conn.prepare("SELECT * FROM listings LIMIT ?")?;
        let rows = stmt.query_map(&[&limit], Self::map_row_to_listing)?;

        let mut listings = Vec::new();
        for row in rows {
            listings.push(row?);
        }
        Ok(listings)
    }

    pub fn get_embedding(&self, id: i64) -> Result<Vec<f32>> {
        let conn = self.connection.get()?;
        let mut stmt = conn.prepare("SELECT embedding FROM embeddings WHERE listing_id = ?")?;
        // read as binary blob without unsafe
        let row = stmt.query_row(&[&id], |row| {
            let bytes: Vec<u8> = row.get(0)?;
            Ok(bytes.chunks_exact(4).map(|chunk|
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect())
        })?;

        Ok(row)
    }

    pub fn get_similar_to_listing(&self, id: u64, limit: u64) -> Result<Vec<SearchResult>> {
        let conn = self.connection.get()?;
        let mut stmt = conn.prepare(
            r"
            WITH search_results AS (
                SELECT listing_id, distance
                FROM embeddings_index
                WHERE embedding MATCH (
                    SELECT embedding FROM embeddings WHERE listing_id = ?1
                )
                ORDER BY distance
                LIMIT ?2
            ) SELECT l.*, sr.distance FROM search_results sr, embeddings e, listings l
                WHERE sr.listing_id == e.rowid AND e.listing_id == l.id",
        )?;

        let rows = stmt.query_map(&[&id, &limit], |row| {
            let distance:f64 = row.get(13)?;
            Ok(SearchResult {
                listing_details: Self::map_row_to_listing(row)?,
                score: 1. - distance,
            })
        })?;

        let mut listings = Vec::new();
        for row in rows {
            listings.push(row?);
        }
        Ok(listings)
    }

    pub fn get_similar_to_embedding(&self, embedding: &Vec<f32>, limit: u64) -> Result<Vec<SearchResult>> {
        let conn = self.connection.get()?;
        let mut stmt = conn.prepare(
            r"
            WITH search_results AS (
                SELECT listing_id, distance
                FROM embeddings_index
                WHERE embedding MATCH ?1
                ORDER BY distance
                LIMIT ?2
            ) SELECT l.*, sr.distance FROM search_results sr, embeddings e, listings l
                WHERE sr.listing_id == e.rowid AND e.listing_id == l.id",
        )?;

        let rows = stmt.query_map((embedding.as_bytes(), limit), |row| {
            let distance:f64 = row.get(13)?;
            Ok(SearchResult {
                listing_details: Self::map_row_to_listing(row)?,
                score: 1. - distance,
            })
        })?;

        let mut listings = Vec::new();
        for row in rows {
            listings.push(row?);
        }
        Ok(listings)
    }

    pub fn get_listing_by_id(&self, id: u64) -> Result<ListingDetails> {
        let conn = self.connection.get()?;
        let row = conn.query_row("SELECT * FROM listings WHERE id = ?",
                                 &[&id],
                                 Self::map_row_to_listing)?;
        Ok(row)
    }

    pub fn map_row_to_listing(row: &rusqlite::Row) -> result::Result<ListingDetails, Error> {
        Ok(ListingDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            neighborhood_overview: row.get(3)?,
            neighbourhood_cleansed: row.get(4)?,
            property_type: row.get(5)?,
            room_type: row.get(6)?,
            accommodates: row.get(7)?,
            bathrooms_text: row.get(8)?,
            bedrooms: row.get(9)?,
            beds: row.get(10)?,
            amenities: row.get(11)?,
            price: row.get(12)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_listing() -> Result<()> {
        let db = Database::in_file("database/listings.db")?;
        let listing = db.get_listings(100)?;
        assert_eq!(listing.len(), 100);
        Ok(())
    }


    #[test]
    fn test_get_listing_by_id() -> Result<()> {
        let db = Database::in_file("database/listings.db")?;
        let listing = db.get_listing_by_id(2595)?;
        assert_eq!(listing.id, 2595);
        Ok(())
    }

    #[test]
    fn test_get_embedding() -> Result<()> {
        let db = Database::in_file("database/listings.db")?;
        let embedding = db.get_embedding(2595)?;
        let listings = db.get_similar_to_embedding(&embedding, 10)?;
        for l in listings {
            println!("{:?} -> {:?}", l.score, l.listing_details.name);
        }

        Ok(())
    }

    #[test]
    fn test_get_similar_to_listing() -> Result<()> {
        let db = Database::in_file("database/listings.db")?;
        let listing = db.get_similar_to_listing(12937, 123)?;
        assert_eq!(listing.len(), 123);
        Ok(())
    }

    #[test]
    #[ignore]
    fn create_virtual_tables() -> Result<()> {
        let db = Database::in_file("database/listings.db")?;
        let conn = db.connection.get()?;
        conn.execute("CREATE VIRTUAL TABLE \
            IF NOT EXISTS embeddings_index \
            USING vec0(embedding float[768] distance_metric=cosine, listing_id int primary key)", [])?;
        conn.execute("INSERT INTO embeddings_index(listing_id, embedding) \
            SELECT rowid, embedding FROM embeddings", [])?;
        Ok(())
    }
}