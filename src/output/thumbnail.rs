//! Creates & writes the thumbnails that make up the gallery.

use super::Config;
use crate::model::{ Image, Collection };
use anyhow::Result;
use std::path::{ Path, PathBuf };
use image::ImageOutputFormat;
// use imager::api::

/// Prepares an image group for writing.
pub(super) fn render_thumbnails(image_group: &Collection, config: &Config) -> Result<Vec<Thumbnail>> {
    let mut res = Vec::new();
    for img in &image_group.images {
        if let Some(thumbnail) = render_thumbnail(img, image_group, config) {
            res.push(thumbnail);
        }
    }

    Ok(res)
}

/// A single thumbnail ready to be written to disk.
#[derive(Debug)]
pub struct Thumbnail {
    input_path: PathBuf,
    output_path: PathBuf,
}

/// Prepares a single thumbnail for writing if it does not currently exist
fn render_thumbnail(image: &Image, group: &Collection, config: &Config) -> Option<Thumbnail> {
    let p = config.output_path.join(image.thumbnail_url(group).unwrap());
    
    if !needs_update(&image.path, &config.output_path) {
        None
    } else {
        Some(Thumbnail {
            input_path: image.path.clone(),
            output_path: p.clone(),
        })
    }
}

fn needs_update(input_path: &Path, output_path: &Path) -> bool {
    if !output_path.exists() {
        println!("making thumbnail");
        let res = || -> Result<bool> {
            let output_modified = output_path.metadata()?.modified()?;
            let input_modified = input_path.metadata()?.modified()?;
            
            // Needs update if the output is older than the input.
            Ok(output_modified < input_modified)
        };

        return res().unwrap_or(true)
    }

    true
}

impl Thumbnail {
    pub fn write(&self) -> Result<()> {
        // Resize image to suitable thumbnail size
        let mut img = image::open(&self.input_path)?;
        img = img.thumbnail(400, 400);

        // Write it to output
        std::fs::create_dir_all(&self.output_path.parent().unwrap())?;
        let file = std::fs::File::create(&self.output_path).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        img.write_to(&mut writer, ImageOutputFormat::Jpeg(40))?;
        Ok(())
    }
}