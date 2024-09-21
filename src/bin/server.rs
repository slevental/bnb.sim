use actix_web::middleware::Logger;
use actix_web::{middleware, web, App, HttpServer};
use bnb_sim::{handler, Service};
use std::{env, io};
use utoipa::{
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
use bnb_sim::handler::{Listing, ListingsApi};

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let openapi = ListingsApi::openapi();

    // read database from the argument
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <database>", args[0]);
        std::process::exit(1);
    }

    let store = web::Data::new(
        Service::from_file(&args[1])
            .expect("Failed to open database"));

    HttpServer::new(move || {
        let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", openapi.clone());

        App::new()
            .wrap(Logger::default())
            .wrap(middleware::Compress::default())
            .route("/ping", web::get().to(ping))
            .service(swagger)
            .service(web::scope("")
                .configure(handler::configure(store.clone())))
    })
        .bind("0.0.0.0:9090")?
        .run()
        .await
}

async fn ping() -> &'static str {
    "pong"
}