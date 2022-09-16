//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod thumbnail;

use crate::model::Gallery;
use crate::error::PathErrorContext;

use anyhow::Result;
use std::fs::{ write, create_dir_all };
use std::path::PathBuf;

/// Configuration options for the output module.
pub(crate) struct Config {
    /// The target directory where to write the gallery.
    pub output_path: PathBuf,
}

/// Writes everything to disk.
pub(crate) fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    let templates = html::make_templates()?;

    // Collect items to write
    let gallery_html = vec![html::render_gallery_html(gallery, config, &templates)?];
    let collections_html = vec![html::render_collections_html(gallery, config, &templates)?];
    // let (collection, thumbnails) = gallery.collections.into_iter().for_each(|i| {

    // }).collect::<Vec<>>(); 

    let mut collection_html = Vec::new();
    let mut thumbnails = Vec::new();
    for i in &gallery.collections {
        collection_html.extend(html::render_collection_html(i, config, &templates)?);
        thumbnails.extend(thumbnail::render_thumbnails(i, config)?);
    }

    gallery_html.into_iter()
        .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?; 

    collections_html.into_iter()
        .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?;

    collection_html.into_iter()
        .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?;

    // thumbnails.into_iter()
    //     .map(|item| item.write())
    //     .collect::<Result<Vec<_>>>()?; 
        
    write_static(config)
}

/// Writes static assets such as CSS files to disk.
fn write_static(config: &Config) -> Result<()> {
    for (path, content) in [
        (&config.output_path.join("index.css"), include_str!("../../templates/index.css")),
        (&config.output_path.join("carousel.css"), include_str!("../../templates/carousel.css")),
        // (&config.output_path.join("data").join("TiredOfCourierThin.ttf"), include_str!("../../data/fonts/TiredOfCourierThin.ttf")),
        (&config.output_path.join("data").join("worm.svg"), include_str!("../../data/svgs/worm.svg")),

    ] {
        create_dir_all(path.parent().path_context("Could not determine parent directory", path)?)?;
        write(path, content).path_context("Failed to write asset", path)?;
    }

    Ok(())
}