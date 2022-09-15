use crate::error::{path_error, PathErrorContext};

use anyhow::Result;
use std::path::{Path, PathBuf};
use time::Date;
// use chrono::DateTime;

/// An unorganised endless gallery of all the images.
#[derive(Debug)]
pub(crate) struct Gallery {
    /// The list of image groups in the gallery.
    /// Sorted by date (most recent first).
    pub collections: Vec<ImageGroup>,
}

/// A collection of images.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ImageGroup {
    /// The path to the image group directory relative to the base directory.
    pub path: PathBuf,
    /// The user-visible title of the image group.
    pub title: String,
    /// The date range of the image group.
    pub date: Date, // DateRange
    /// Images sorted alphabetically. MAKE chronologically
    pub images: Vec<Image>,
}

impl ImageGroup {
    // do i want only ascii characters? fuck ascii-only...
    /// The URL to this image group, relative to the base directory, consisting only of ASCII characters.
    pub(crate) fn url(&self) -> Result<PathBuf> {
        to_web_path(&self.path)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Image {
    /// The user-visible name of the image.
    pub name: String,
    /// The user-visible description of the image.
    // pub desc: String,
    /// The user-visible description of the image.
    // pub date: DateTime<Tz>,
    /// The full path to the source image.
    pub path: PathBuf,
    /// The file name of the source image.
    pub file_name: PathBuf
}

impl Image {
    pub(crate) fn new(file_name: PathBuf, path: PathBuf) -> Result<Image> {
        Ok(Image {
            name: file_name
                .file_stem()
                .path_context("Failed to determine file stem", &file_name)?
                .to_str()
                .path_context("Failed to decode file name as UTF-8", &file_name)?
                .to_owned(),
            path,
            file_name,
        })
    }

    /// The URL to this image relative to the location of the image.
    pub(crate) fn url_file_name(&self) -> Result<PathBuf> {
        to_web_path(&self.file_name)
    }

    /// The URL to the thumbnail image relative to the output base directory.
    pub(crate) fn thumbnail_url(&self, group: &ImageGroup) -> Result<PathBuf> {
        let mut suffix = to_web_path(&group.path)?.join(to_web_path(&self.file_name)?);
        
        // Always use webp for thumbnails to get a reasonable quality.
        suffix.set_extension("jpg");
        Ok(PathBuf::from("thumbnails").join(&suffix))
    }
}

/// Converts a single-element path into something suitable for a URL.
fn to_web_path(path: &Path) -> Result<PathBuf> {
    if path.components().count() != 1 {
        return Err(path_error(
            "Cannot convert multi-component paths into URLs",
            path,
        ));
    }
    
    let p = path
        .to_str()
        .path_context("Failed to convert path to UTF-8", path)?;
    
    // Keep the file extension intact if one is present.
    match p.rsplit_once('.') {
        Some((path, ext)) => Ok(PathBuf::from(slug::slugify(path) + "." + ext)),
        None => Ok(PathBuf::from(slug::slugify(p))),
    }
}


// pub(crate) struct Collection {
//     pub title: String,
//     pub date: Date,
//     pub locations: Vec<String>,
//     pub chromaticity: Chromaticity,
//     pub camera: Vec<String>,
//     pub imaging: Imaging,
//     pub images: Vec<Image>
// }

// pub enum Chromaticity {
//     Mono,
//     Colour,
// }

// pub enum Imaging {
//     Film,
//     Digital
// }

