use std::path::Path;
use rusqlite::{Connection, Result};
use log::{error, info, debug};
use simple_logger::SimpleLogger;
use sha256::try_digest;
use clap::{arg, command, Command};

#[derive(Debug)]
struct Photo {
    id: i32,
    path: String,
    hash: String,
}

fn setup_db() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;

    connection.execute(
        "CREATE TABLE photo (
            id      INTEGER PRIMARY KEY,
            path    TEXT NOT NULL,
            hash    TEXT NOT NULL
        )", ()
    )?;

    Ok(connection)
}

fn get_photos_in_path(path: &Path) -> std::io::Result<Vec<Photo>> {
    let mut photos: Vec<Photo> = Vec::new();

    debug!("Getting photos in path: {:?}", path);

    for entry in std::fs::read_dir(path)? {

        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            photos.append(&mut get_photos_in_path(path.as_path())?);
        } else {
            let photo = Photo {
                id: 0,
                path: path.to_string_lossy().to_string(),
                hash: try_digest(entry.path().as_path()).unwrap()
            };

            debug!("Photo found on disc: {:?}", photo);
             
            photos.push(photo);
        }
    }

    debug!("Found photos in path {:?}: {:?}", path, photos);

    Ok(photos)
}

fn insert_photo(photo: &Photo, connection: &Connection) -> Result<usize> {
    connection.execute("INSERT INTO photo (path, hash) VALUES (?1, ?2)",
        (&photo.path, &photo.hash),
    )
}

fn create_index() -> Result<()> {

    let connection: Connection = setup_db()?;
    let photos = match get_photos_in_path(Path::new("sample_photos")) {
        Ok(res) => res,
        Err(_) => Vec::new(),
    };

    photos.iter()
        .map(|photo| insert_photo(photo, &connection))
        .for_each(|res| match res {
            Ok(_) => {},
            Err(err) => {
                error!("Inserting photo into database resulted in an error: {:}", err);
                panic!("Something bad happened, check log for reasons.");
            },
        });

    
    let mut stmt = connection.prepare("SELECT id, path, hash FROM photo")?;
    let photo_iter = stmt.query_map([], |row| {
        Ok(Photo {
            id: row.get(0)?,
            path: row.get(1)?,
            hash: row.get(2)?,
        })
    })?;

    for photo in photo_iter {
        info!("{:?}", photo.unwrap());
    }

    Ok(())
}

fn main() -> Result<()> {
    
    SimpleLogger::new().init().unwrap();


    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("index")
                .about("Looks for photos and catalogs them")
            )
        .get_matches();

    match matches.subcommand() {
        Some(("index", sub_matches)) => {create_index();},
        _ => unreachable!("The command is valid but not available!"),
    }

    Ok(())
}
