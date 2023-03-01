use std::path::Path;
use async_recursion::async_recursion;
use log::debug;
use sqlx::sqlite::SqlitePool;
use sha256::try_digest;
use crate::model::Photo;

pub async fn list_index(pool: &SqlitePool) -> anyhow::Result<()> {

    let records = sqlx::query!("SELECT id, path, hash FROM photos").fetch_all(pool).await?;

    for record in records {
        println!("{:?}", Photo::new (
            record.id,
            record.path,
            record.hash,
        ));
    };

    Ok(())
}

pub async fn create_index(path: String, pool: &SqlitePool) -> anyhow::Result<()> {

    let photos = get_photos_in_path(Path::new(&path)).await;

    println!("{:?}", photos);

    for photo in photos {
        insert_photo(&photo, pool).await?;
    }

    Ok(())
}


#[async_recursion]
async fn get_photos_in_path(path: &Path) -> Vec<Photo> {
    let mut photos: Vec<Photo> = Vec::new();

    debug!("Getting photos in path: {:?}", path);

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let i_path = entry.path();
            if i_path.is_file() {
                photos.push(Photo::new (                       
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

    photos
}

async fn insert_photo(photo: &Photo, pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query!("INSERT INTO photos (path, hash) VALUES (?1, ?2)", photo.path, photo.hash).execute(pool).await?;

    Ok(())
}
