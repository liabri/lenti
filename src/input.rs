//! Reads the source directory into the internal representation.
//!
//! This is a read-only operation.
use crate::error::PathErrorContext;
use crate::model::{Gallery, Image, Collection};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fmt, fs};
use time::{macros::format_description, Date};

pub(crate) fn gallery_from_dir(path: &Path) -> Result<Gallery> {
    let mut collections = Vec::<Collection>::new();
    for d in read_dir(path)?.iter().filter(|d| d.is_dir) {
        let contents = read_dir(&d.path)?;
        if let Some(group) = Collection::from_entries(&d.file_name, &contents)? {
            collections.push(group);
        }
    }
    collections.sort_by(|lhs, rhs| rhs.date.cmp(&lhs.date));
    Ok(Gallery { collections })
}

impl Image {
    fn from(d: &DirEntry) -> Result<Image> {
        Image::new(d.file_name.clone(), d.path.clone())
    }
}

impl Collection {
    fn from_entries(path: &Path, v: &[DirEntry]) -> Result<Option<Collection>> {
        // let (title, date) = {
        //     let id = path.to_str().unwrap_or("");
        //     let re = Regex::new(r"^(\d{4}-\d{2}-\d{2}).").unwrap();

        //     let c = {
        //         match re.captures(id) {
        //             Some(c) => c,
        //             None => return Ok(None),
        //         }
        //     };
        //     (
        //         re.replace(id, "").into_owned(),
        //         Date::parse(
        //             c.get(1).unwrap().as_str(),
        //             format_description!("[year]-[month]-[day]"),
        //         )?,
        //     )
        // };

        let title = path.to_str().unwrap_or("").to_string();
        let date = Date::parse("2020-03-20", format_description!("[year]-[month]-[day]"))?;

        let mut images = Vec::new();
        for d in v {
            if d.is_image() {
                images.push(Image::from(d)?);
            }
            if d.is_index() {
                //make index
            }
        }
        
        images.sort();
        Ok(Some(Collection {
            path: path.to_owned(),
            title,
            date,
            images,
        }))
    }
}

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
        self.path.file_name().map_or(false, |f| f == "index.md")
    }
}

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
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

#[cfg(test)]
mod tests {
    use super::{DirEntry, Image, Collection};
    use std::path::{Path, PathBuf};
    use time::{Date, Month};

    fn dir(dirname: &str, file_names: &[(&str, bool)]) -> Vec<DirEntry> {
        file_names
            .iter()
            .map(|(p, is_dir)| DirEntry {
                path: [dirname, p].iter().collect(),
                file_name: PathBuf::from(p),
                is_dir: *is_dir,
            })
            .collect()
    }
    struct SimpleCollection<'a> {
        name: &'a str,
        title: &'a str,
        date: Date,
        // (name, path)
        images: &'a [(&'a str, &'a str)],
        markdown_file: Option<&'a str>,
    }
    impl<'a> From<SimpleCollection<'a>> for Collection {
        fn from(s: SimpleCollection) -> Collection {
            Collection {
                path: PathBuf::from(s.name),
                title: String::from(s.title),
                date: s.date,
                images: s
                    .images
                    .iter()
                    .map(|(n, p)| Image {
                        name: String::from(*n),
                        path: PathBuf::from(p),
                        file_name: PathBuf::from(p).file_name().unwrap().into(),
                    })
                    .collect(),
                markdown_file: s.markdown_file.map(PathBuf::from),
            }
        }
    }

    #[test]
    fn test_empty_dir() {
        assert_eq!(
            Collection::from_entries(Path::new("2021-01-01 Fuji, Japan"), &[]).unwrap(),
            Some(Collection::from(SimpleCollection {
                title: "Fuji, Japan",
                name: "2021-01-01 Fuji, Japan",
                date: Date::from_calendar_date(2021, Month::January, 01).unwrap(),
                images: &[],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_simple_dir() {
        assert_eq!(
            Collection::from_entries(
                Path::new("2021-01-01 Fuji, Japan"),
                &dir(
                    "2021-01-01 Fuji, Japan",
                    &[("Valley.webp", false), ("Summit.webp", false),]
                )
            )
            .unwrap(),
            Some(Collection::from(SimpleCollection {
                name: "2021-01-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: Date::from_calendar_date(2021, Month::January, 01).unwrap(),
                images: &[
                    ("Summit", "2021-01-01 Fuji, Japan/Summit.webp"),
                    ("Valley", "2021-01-01 Fuji, Japan/Valley.webp"),
                ],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_index() {
        assert_eq!(
            Collection::from_entries(
                Path::new("2021-01-01 Fuji, Japan"),
                &dir("some/path/2021-01-01 Fuji, Japan", &[("index.md", false)])
            )
            .unwrap(),
            Some(Collection::from(SimpleCollection {
                name: "2021-01-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: Date::from_calendar_date(2021, Month::January, 01).unwrap(),
                images: &[],
                markdown_file: Some("some/path/2021-01-01 Fuji, Japan/index.md")
            }))
        );
    }
    #[test]
    fn test_ignored_entries() {
        assert_eq!(
            Collection::from_entries(
                Path::new("2021-12-01 Fuji, Japan"),
                &dir(
                    "some/path/2021-12-01 Fuji, Japan",
                    &[
                        ("Valley", true), // directory
                        ("Summit.webp", false),
                        ("something.unknown", false),
                    ]
                )
            )
            .unwrap(),
            Some(Collection::from(SimpleCollection {
                name: "2021-12-01 Fuji, Japan",
                title: "Fuji, Japan",
                date: Date::from_calendar_date(2021, Month::December, 01).unwrap(),
                images: &[("Summit", "some/path/2021-12-01 Fuji, Japan/Summit.webp")],
                markdown_file: None,
            }))
        );
    }
    #[test]
    fn test_missing_date_in_dirname() {
        assert_eq!(
            Collection::from_entries(
                Path::new("2021-01 Fuji, Japan"),
                &dir("some/path/2021-01 Fuji, Japan", &[("Summit.webp", false)])
            )
            .unwrap(),
            None
        );
    }
}
