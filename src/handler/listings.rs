use crate::Service;
use actix_web::{
    get, post,
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};
use crate::db::{ListingDetails, SearchResult};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_listing,
        search_similar,
    ),
    components(schemas(Listing, SearchListing, ErrorResponse))
)]
pub struct ListingsApi;

pub fn configure(store: Data<Service>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.app_data(store)
            .service(search_similar)
            .service(get_listing);
    }
}

/// Similarity search params
#[derive(Deserialize, Debug, IntoParams, ToSchema)]
pub struct SearchListing {
    /// Value to search similar to given id
    #[schema(example = 39282)]
    similar_to: i64,

    /// Maximum number of results to return
    #[schema(example = 5)]
    max_results: i64,
}

/// Listing's outside representation
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct Listing {
    /// Listing id
    #[schema(example = 1)]
    id: i64,
    /// Title of the listing
    #[schema(example = "Apartment at UWS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Description of the listing
    #[schema(example = "Beautiful apartment with a view of the Hudson River")]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Number of bedrooms
    #[schema(example = 2)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bedrooms: Option<i32>,
    /// Price per night
    #[schema(example = "$200.00")]
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<String>,

    #[schema(example = "0.98")]
    #[serde(skip_serializing_if = "Option::is_none")]
    score: Option<f32>,
}

/// Error response for listing operations
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub enum ErrorResponse {
    /// When Todo is not found by search term.
    NotFound(String),
    /// Server error
    ServerError,
}

/// Get listing by id
///
/// Return found listing by id from storage. Returns 200 and `Listing` item if found.
#[utoipa::path(
    responses(
        (status = 200, description = "Listing found from storage", body = Listing),
        (status = 404, description = "Listing not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Listing")
    )
)]
#[get("/listing/{id}")]
async fn get_listing(id: Path<i64>, store: Data<Service>) -> impl Responder {
    let id = id.into_inner();
    match store.get_listing(id).await {
        Ok(listing) => HttpResponse::Ok().json(Into::<Listing>::into(listing)),
        Err(_) => HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}"))),
    }
}

/// Search listings using other listing id as reference
///
/// Return similar listings to given id. Returns 200 and list of `Listing` items if found.
#[utoipa::path(
    request_body = SearchListing,
    responses(
        (status = 200, description = "Search Listing did not result error", body = [Listing]),
    )
)]
#[post("/listing")]
async fn search_similar(query: Json<SearchListing>, storage: Data<Service>) -> impl Responder {
    match storage.search_similar(query.similar_to, query.max_results).await {
        Ok(listings) => HttpResponse::Ok()
            .json(listings.into_iter().map(Into::into).collect::<Vec<Listing>>()),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse::ServerError),
    }
}

impl From<ListingDetails> for Listing {
    fn from(value: ListingDetails) -> Self {
        Listing {
            id: value.id,
            name: value.name,
            description: value.description,
            bedrooms: value.bedrooms.map(|s|s as i32),
            price: value.price,
            score: None,
        }
    }
}

impl From<SearchResult> for Listing {
    fn from(value: SearchResult) -> Self {
        Listing {
            id: value.listing_details.id,
            name: value.listing_details.name,
            description: value.listing_details.description,
            bedrooms: value.listing_details.bedrooms.map(|x| x as i32),
            price: value.listing_details.price,
            score: Some(value.score as f32),
        }
    }
}
