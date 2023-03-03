use clap::{arg, command, Command};
use sqlx::sqlite::SqlitePool;

use axum::{
    routing::get,
    response::Json,    
    Router,
};

use serde_json::{Value, json};

use crate::index::Index;

pub async fn init(pool: &SqlitePool) -> anyhow::Result<()> {

    let index_svc = Index::init(pool);

    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("index")
                .about("Manage the photo catalog")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("create")
                        .about("Scan a directory recursively and create the catalog")
                        .arg(arg!(<PATH> "Path to directory with photos"))
                        .arg_required_else_help(true)
                    )
                .subcommand(
                    Command::new("list")
                        .about("List all photos of the catalog")
                )
            )
        .subcommand(
            Command::new("server")
                .about("Manage the web server that exposes the photo catalog")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("start")
                        .about("Start the web server that exposes the photo catalog")
                )
            )
        .get_matches();

    if let Some(("index", index)) = matches.subcommand() {
        match index.subcommand() {
            Some(("create", create)) => {
                let path = create.get_one::<String>("PATH").expect("required");            
                index_svc.create_index(path.to_string()).await?;
            },
            Some(("list", _)) => {
                index_svc.list_index().await?;
            },
            _ => (),
        }
    };

    if let Some(("server", server)) = matches.subcommand() {
        if let Some(("start", _)) = server.subcommand() {
            // build our application with a single route
            let app = Router::new()
                .route("/", get(|| async {"test"}));

            // run it with hyper on localhost:3000
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    }
    
    Ok(())
}
