//! Writes the HTML pages that make up the gallery.
use super::Config;

use crate::error::{ path_error, PathErrorContext };
use crate::model::{ Gallery, Image, Collection };

use anyhow::{ Context, Result };
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct Templates<'a>(Handlebars<'a>);

pub struct HTMLFile {
    content: String,
    output_path: PathBuf,
}

impl HTMLFile {
    pub fn write(&self) -> Result<()> {
        std::fs::create_dir_all(&self.output_path.parent().path_context("Could not determine parent directory", &self.output_path)?)?;
        fs::write(&self.output_path, &self.content)
            .path_context("Failed to write HTML file", &self.output_path)
    }
}

pub(super) fn make_templates<'a>() -> Result<Templates<'a>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("gallery", include_str!("../../templates/gallery.hbs"))?;
    handlebars.register_template_string("collections", include_str!("../../templates/collections.hbs"))?;
    handlebars.register_template_string("collection", include_str!("../../templates/collection.hbs"))?;
    handlebars.register_script_helper_file("inc", "scripts/inc.rhai")?;
    handlebars.register_script_helper_file("dec", "scripts/dec.rhai")?;
    Ok(Templates(handlebars))
}











/// Used in handlebars templates to describe a gallery.
#[derive(Serialize)]
struct GalleryData {
    featured_images: Vec<ImageData>,
}

impl GalleryData {
    fn from_gallery(gallery: &Gallery) -> Result<GalleryData> {
        let mut featured_images: Vec<ImageData> = Vec::new();
        for collection in &gallery.collections {
            for image in &collection.images {
                if collection.feat.contains(&image.name) {
                    featured_images.push(ImageData::from_image(image.clone(), &collection)?);
                }
            }           
        }

        Ok(GalleryData{featured_images})
    }
}

pub(super) fn render_gallery_html(gallery: &Gallery, config: &Config, templates: &Templates) -> Result<HTMLFile> {
    // let mut data = std::collections::BTreeMap::new();
    // data.insert("gallery".to_string(), );

    Ok(HTMLFile {
        content: templates
            .0
            .render("gallery", &GalleryData::from_gallery(gallery)?)
            .with_context(|| "Failed to render gallery HTML page")?,
        output_path: config.output_path.join("gallery.html"),
    })
}








/// Used in handlebars templates to describe a collection.
#[derive(Serialize)]
struct CollectionData {
    path: String,
    title: String,
    date: String,
    images: Vec<ImageData>,
    url: String,
}

impl CollectionData {
    fn from_collection(collection: &Collection) -> Result<CollectionData> {
        let images = collection
            .images
            .iter()
            .map(|image| ImageData::from_image(image, collection))
            .collect::<Result<Vec<_>>>()?;

        let data = CollectionData {
            path: collection.path.display().to_string(),
            title: collection.title.clone(),
            date: collection.date.to_string(),
            images,
            url: url_to_string(&collection.url()?)?
        };

        Ok(data)
    }
}

pub(super) fn render_collections_html(gallery: &Gallery, config: &Config, templates: &Templates) -> Result<HTMLFile> {
    let mut data = std::collections::BTreeMap::new();
    data.insert("collections".to_string(), gallery.collections.iter()
        .map(|group| CollectionData::from_collection(group))
        .collect::<Result<Vec<_>>>()?);

    Ok(HTMLFile {
        content: templates
            .0
            .render("collections", &data)
            .with_context(|| "Failed to render collections HTML page")?,
        output_path: config.output_path.join("collections.html"),
    })
}

pub(super) fn render_collection_html(collection: &Collection, config: &Config, templates: &Templates) -> Result<Option<HTMLFile>> {
    let mut data = std::collections::BTreeMap::new();
    data.insert("collection".to_string(), CollectionData::from_collection(collection)?);

    Ok(Some(HTMLFile {
        content: templates.0.render("collection", &data).with_context(|| {
            format!("Failed to render HTML page for image group \"{}\"", collection.title)
        })?,
        output_path: config.output_path.join("collection").join(&collection.path).with_extension("html"),
    }))
}








/// Used in handlebars templates to describe a single image.
#[derive(Debug, Serialize)]
struct ImageData {
    file_name: String,
    name: String,
    thumbnail: String,
    anchor: String,
    collection: String
}

impl ImageData {
    fn from_image(image: &Image, collection: &Collection) -> Result<ImageData> {
        Ok(ImageData {
            file_name: url_to_string(&image.url_file_name()?)?,
            name: image.name.clone(),
            thumbnail: url_to_string(&image.thumbnail_url(collection)?)?,
            anchor: slug::slugify(&image.name),
            collection: collection.path.display().to_string()
        })
    }
}












/// Converts a URL from path form into a string.
/// The path components will be joined by slashes.
fn url_to_string(url: &Path) -> Result<String> {
    Ok(url
        .iter()
        .map(|c| c.to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| path_error("Failed to decode UTF-8", url))?
        .join("/"))
}