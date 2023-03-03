use std::path::Path;
use log::debug;
use sqlx::sqlite::SqlitePool;
use sha256::try_digest;
use crate::model::Photo;

pub struct Index<'a> {
    pool: &'a SqlitePool,
}

impl Index<'_> {
    pub fn init(pool: &SqlitePool) -> Index {
        Index {
            pool
        }
    }

    pub async fn list_index(self: &Self) -> anyhow::Result<Vec<Photo>> {

        let records = sqlx::query!("SELECT id, path, hash FROM photos").fetch_all(self.pool).await?;
        
        let mut photos = Vec::new();
        for record in records {
            let photo = Photo::new (
                record.id,
                record.path,
                record.hash,
            );

            debug!("Adding photo to result set: {:?}", photo);

            photos.push(photo);
        };

        Ok(photos)
    }

    pub async fn create_index(self: &Self, path: String) -> anyhow::Result<()> {

        let photos = get_photos_in_path(Path::new(&path));

        println!("{:?}", photos);

        for photo in photos {
            insert_photo(&photo, self.pool).await?;
        }

        Ok(())
    }
}

fn get_photos_in_path(path: &Path) -> Vec<Photo> {
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
                let sub_dir_photos = get_photos_in_path(i_path.as_path());
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
