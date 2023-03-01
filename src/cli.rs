use clap::{arg, command, Command};
use sqlx::sqlite::SqlitePool;

pub async fn init(pool: &SqlitePool) -> anyhow::Result<()> {

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
        .get_matches();

    if let Some(("index", index)) = matches.subcommand() {
        match index.subcommand() {
            Some(("create", create)) => {
                let path = create.get_one::<String>("PATH").expect("required");            
                super::index::create_index(path.to_string(), pool).await?;
            },
            Some(("list", _)) => {
                super::index::list_index(pool).await?;
            },
            _ => (),
        }
    };

    Ok(())
}
