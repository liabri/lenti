//! Reads the source directory into the internal representation.
//!
//! This is a read-only operation.
use crate::error::PathErrorContext;
use crate::model::{Gallery, Image, Collection};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fs, io::BufReader};

// BELOW IS TEST

// let mut img = image::open(&self.input_path)?;
// img = img.thumbnail(400, 400);

// std::fs::create_dir_all(&self.output_path.parent().unwrap())?;
// let file = std::fs::File::create(&self.output_path).unwrap();
// let mut writer = std::io::BufWriter::new(file);
// img.write_to(&mut writer, ImageOutputFormat::Jpeg(40));

// ABOVE IS TEST

#[derive(Debug)]
struct DirEntry {
    path: PathBuf,
    file_name: PathBuf, // relative to the base dir
    is_dir: bool,
}

impl DirEntry {
    fn is_image(&self) -> bool {
        self.path
            .extension()
            .map_or(false, |e| e == "webp" || e == "jpeg" || e== "png" || e == "jpg" || e == "JPG")
    }
    fn is_index(&self) -> bool {
        self.path.file_name().map_or(false, |f| f == "collection-info.zm")
    }
}

impl Gallery {
    pub fn from_path(path: &Path) -> Result<Gallery> {
        let mut collections = Vec::<Collection>::new();

        // iterate through every collection folder
        for d in read_dir(path)?.iter().filter(|d| d.is_dir) {
            let contents = read_dir(&d.path)?;
            if let Some(group) = Collection::from_entries(&path.join(&d.file_name), &contents)? {
                collections.push(group);
            }
        }

        collections.sort_by(|lhs, rhs| rhs.date.cmp(&lhs.date));
        Ok(Gallery { collections })
    }
}

impl Collection {
    fn from_entries(path: &Path, v: &[DirEntry]) -> Result<Option<Collection>> {
        let mut collection: Option<Collection> = None;
        let mut images = Vec::new();

        for d in v {
            if d.is_image() {
                images.push(Image::from(d)?);
            }

            if d.is_index() {
                let file = fs::File::open(&d.path)?;
                let reader = BufReader::new(file);
                collection = Some(zmerald::from_reader(reader).unwrap());
            }
        }

        if let Some(mut coll) = collection {
            images.sort();
            coll.images = images;
            coll.path = PathBuf::from(path.file_name().unwrap());
            return Ok(Some(coll));
        }

        Ok(None)
    }
}

impl Image {
    fn from(d: &DirEntry) -> Result<Image> {
        Image::new(d.file_name.clone(), d.path.clone())
    }
}

// Reads a directory non-recursively.
fn read_dir(base_dir: &Path) -> Result<Vec<DirEntry>> {
    let mut res = Vec::new();
    for path in fs::read_dir(base_dir).path_context("Failed to open directory", base_dir)? {
        let d = path.path_context("Failed to read the contents of directory", base_dir)?;
        let path = d.path();
        res.push(DirEntry {
            file_name: path
                .strip_prefix(base_dir)
                .path_context("Failed to remove base directory prefix", &path)?
                .to_owned(),
            is_dir: d
                .metadata()
                .path_context("Failed to read metadata", &path)?
                .is_dir(),
            path,
        })
    }
    Ok(res)
}

// impl std::fmt::Display for DirEntry {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.path.to_string_lossy())
//     }
// }