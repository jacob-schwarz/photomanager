mod model {
    #[derive(Debug)]
    pub struct Photo {
        id: i64,

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
}

mod index {
    use std::path::Path;
    use sqlx::SqliteExecutor;
    use async_recursion::async_recursion;
    use log::debug;
    use sqlx::sqlite::SqlitePool;
    use sha256::try_digest;

    pub async fn create_index(path: String, pool: &SqlitePool) -> anyhow::Result<()> {

        let photos = get_photos_in_path(Path::new(&path)).await;

        println!("{:?}", photos);

        for photo in photos {
            insert_photo(&photo, pool).await?;
        }

        Ok(())
    }

    pub async fn list_index(pool: &SqlitePool) -> anyhow::Result<()> {

        let records = sqlx::query!("SELECT id, path, hash FROM photos").fetch_all(pool).await?;

        for record in records {
            println!("{:?}", super::model::Photo::new (
                record.id,
                record.path,
                record.hash,
            ));
        };

        Ok(())
    }

    #[async_recursion]
    async fn get_photos_in_path(path: &Path) -> Vec<super::model::Photo> {
        let mut photos: Vec<super::model::Photo> = Vec::new();

        debug!("Getting photos in path: {:?}", path);

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries {

                if let Ok(entry) = entry {
                    let i_path = entry.path();

                    if i_path.is_file() {
                        photos.push(super::model::Photo::new (                       
                            0,
                            i_path.to_string_lossy().to_string(),
                            try_digest(entry.path().as_path()).unwrap()
                        ));
                    } else if i_path.is_dir() {
                        let sub_dir_photos = get_photos_in_path(i_path.as_path()).await;
                        photos.extend(sub_dir_photos);
                    }
                }
            }
        }

        debug!("Found photos in path {:?}: {:?}", path, photos);

        photos
    }

    async fn insert_photo(photo: &super::model::Photo, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("INSERT INTO photos (path, hash) VALUES (?1, ?2)", photo.path, photo.hash).execute(pool).await?;

        Ok(())
    }

}

mod cli {
    use clap::{arg, command, Command};
    use sqlx::sqlite::SqlitePool;
    
    pub async fn init(pool: &SqlitePool) -> anyhow::Result<()> {

        let matches = command!()
            .propagate_version(true)
            .subcommand_required(true)
            .subcommand(
                Command::new("index")
                    .about("Manage the photo catalog")
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

        match matches.subcommand() {
            Some(("index", index)) => {
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
            },
            _ => (),
        };

        Ok(())
    }
}

use simple_logger::SimpleLogger;
use sqlx::sqlite::SqlitePool;
use std::env;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    SimpleLogger::new().init().unwrap();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    cli::init(&pool).await?;

    Ok(())
}
