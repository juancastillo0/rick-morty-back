#![feature(decl_macro, proc_macro_hygiene)]

use juniper::IntrospectionFormat;
use rick_morty_back::db;
use rick_morty_back::graphql::{self, Ctx, GraphqlSchema};
use rocket::{http, response::content, State};

#[rocket::get("/graphql")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::playground_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
fn get_graphql_handler(
    context: State<Ctx>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<GraphqlSchema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    context: State<Ctx>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<GraphqlSchema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &context)
}

use std::fs::File;
use std::io::BufWriter;

fn main() {
    let counts = db::init_db().unwrap();
    println!("{:?}", counts);

    let schema_graphql = graphql::create_schema();
    let ctx = Ctx::new(counts);

    let (res, _errors) =
        juniper::introspect(&schema_graphql, &ctx, IntrospectionFormat::default()).unwrap();
    let file = File::create("graphql_schema.json").unwrap();
    serde_json::to_writer_pretty(BufWriter::new(file), &res).unwrap();

    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::some_exact(&[
            "http://localhost:4200",
            "http://localhost:8000",
        ]),
        allowed_methods: vec![http::Method::Get, http::Method::Post, http::Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Error building CORS");
    let rocket_config = rocket::config::Config::build(rocket::config::Environment::Development)
        .address("127.0.0.1")
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(rocket_config)
        .manage(ctx)
        .manage(schema_graphql)
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .attach(cors)
        .launch();
}
