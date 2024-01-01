//! Writes the gallery to the output directory.
//!
//! Together with its submodules, this module writes everything including images, thumbnails, and HTML files.
mod html;
mod thumbnail;

use crate::model::Gallery;
use crate::error::PathErrorContext;

use anyhow::Result;
use std::fs::{ write, create_dir_all };
use std::path::{ Path, PathBuf };
use glob::glob;

/// Configuration options for the output module.
pub(crate) struct Config {
    /// The directory which contains the gallery.
    pub input_path: PathBuf,
    /// The target directory where to write the gallery.
    pub output_path: PathBuf,
    /// The directory which contains the templates and scripts.
    pub resources_path: PathBuf,
}

/// Writes everything to disk.
pub(crate) fn write_files(gallery: &Gallery, config: &Config) -> Result<()> {
    // Register templates and scripts
    let templates = html::make_templates(config)?;

    // Collect and write
    html::render_gallery_html(gallery, config, &templates)?.write()?;
    html::render_collections_html(gallery, config, &templates)?.write()?;
    for coll in &gallery.collections {
        if let Some(coll) = html::render_collection_html(coll, config, &templates)? { coll.write()?; }
        for img in &coll.images {
            if let Some(thumbnail) = thumbnail::render_thumbnail(&img, coll, config) { thumbnail.write()?; }
            copy(&img.path, &config.output_path.join("data").join("albums").join(&coll.path).join(&img.file_name))?;
        }
        println!("DE {:?}", &coll.path.join("collection-info.zm"));
        println!("DE {:?}", &config.output_path.join("data").join("albums").join(&coll.path).join("collection-info.zm"));
        copy(&config.input_path.join(&coll.path).join("collection-info.zm"), &config.output_path.join("data").join("albums").join(&coll.path).join("collection-info.zm"))?;
    }
        
    write_static(config)
}

/// Writes static assets such as CSS files to disk.
fn write_static(config: &Config) -> Result<()> {
    let res = config.resources_path.join("**/*.css").display().to_string();
    for stylesheet in glob(&res)? {
        if let Ok(stylesheet) = stylesheet {
            copy(&stylesheet, &config.output_path.join(&stylesheet.file_name().unwrap()))?;
        }
    }
    Ok(())
}

pub(crate) fn copy(input_path: &Path, output_path: &Path) -> Result<()> {
    std::fs::create_dir_all(&output_path.parent().unwrap())?;
    std::fs::copy(input_path, output_path)?;
    Ok(())
}