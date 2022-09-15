//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod thumbnail;

use crate::model::Gallery;
use crate::error::PathErrorContext;

use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fs::{ write, create_dir_all };
use std::path::PathBuf;

/// Configuration options for the output module.
pub(crate) struct Config {
    /// The target directory where to write the gallery.
    pub output_path: PathBuf,
}

/// Writes the gallery to disk.
pub(crate) fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    let templates = html::make_templates()?;

    // Create work items.
    let html = vec![html::render_overview_html(gallery, config, &templates)?];
    let collections = vec![html::render_collections_html(gallery, config, &templates)?];
    let mut html2 = Vec::new();
    let mut thumbnails = Vec::new();

    for i in &gallery.collections {
        html2.extend(html::render_collection_html(i, config, &templates)?);
        thumbnails.extend(thumbnail::render_thumbnails(i, config)?);
    }


    // find a better way to do them all parallely
    html.into_par_iter()
           .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?; 

    collections.into_par_iter()
        .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?; 



    html2.into_par_iter()
           .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?; 

    thumbnails.into_par_iter()
           .map(|item| item.write())
        .collect::<Result<Vec<_>>>()?; 
        
    write_static(config)
}

/// Writes static assets such as CSS files to disk.
fn write_static(config: &Config) -> Result<()> {
    let custom_css_path = config.output_path.join("css").join("index.css");
    let carousel = config.output_path.join("css").join("carousel.css");

    for (path, content) in [
        (&custom_css_path, include_str!("../../templates/index.css")),
        (&carousel, include_str!("../../templates/carousel.css")),
    ] {
        create_dir_all(path.parent().path_context("Could not determine parent directory", path)?)?;
        write(path, content).path_context("Failed to write asset", path)?;
    }

    Ok(())
}