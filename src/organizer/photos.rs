use super::MediaTypeOrganizer;
use crate::date::Date;
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct PhotoOrganizer {
    dst_dir: PathBuf,
    date_from_filename_regex: Regex,
}

impl PhotoOrganizer {
    const SUPPORTED: [&'static str; 3] = ["jpeg", "jpg", "JPG"];

    pub fn new(dst_dir: PathBuf) -> PhotoOrganizer {
        PhotoOrganizer {
            dst_dir,
            date_from_filename_regex: Regex::new(r"^IMG\-(\d{4})(\d{2})\d{2}\-WA\d+\..*$").unwrap(),
        }
    }

    fn get_date(&self, photo: &PathBuf) -> Result<Date> {
        let exif_date =
            PhotoOrganizer::date_from_exif(photo).wrap_err("failed to get date from exif");
        if exif_date.is_ok() {
            return exif_date;
        }

        self.date_from_filename(photo)
            .wrap_err("failed to get date from filename")
            .wrap_err(exif_date.unwrap_err())
    }

    fn date_from_filename(&self, photo: &PathBuf) -> Result<Date> {
        let file_name = photo
            .file_name()
            .ok_or_else(|| eyre!("failed to retrieve photo filename"))?;

        let captures = self
            .date_from_filename_regex
            .captures(
                &file_name
                    .to_str()
                    .ok_or_else(|| eyre!("failed to get file name as string"))?,
            )
            .ok_or_else(|| eyre!("file name doesn't have date format"))?;
        let year: u16 = match captures.get(1) {
            Some(y) => y.as_str().parse().unwrap(),
            None => return Err(eyre!("failed to retrieve year from filename")),
        };
        let month: u8 = match captures.get(2) {
            Some(m) => m.as_str().parse().unwrap(),
            None => return Err(eyre!("failed retrieve month from filename")),
        };
        Date::new(year, month)
    }

    fn date_from_exif(photo: &PathBuf) -> Result<Date> {
        let file = fs::File::open(photo).wrap_err("failed to open file")?;
        let mut bufreader = io::BufReader::new(&file);
        let exifreader = exif::Reader::new();
        let exif = exifreader
            .read_from_container(&mut bufreader)
            .wrap_err("failed to read the file")?;
        let datetime_tag = exif
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
            .ok_or_else(|| eyre!("exif DateTimeOriginal tag is missing"))?;
        let exif_datetime = match datetime_tag.value {
            exif::Value::Ascii(ref vec) if !vec.is_empty() => {
                exif::DateTime::from_ascii(&vec[0]).wrap_err("exif date value is broken")?
            }
            _ => return Err(eyre!("exif date value is broken")),
        };
        Date::new(exif_datetime.year, exif_datetime.month)
    }

    fn is_supported(extension: &str) -> bool {
        for i in PhotoOrganizer::SUPPORTED.iter() {
            if extension.eq(*i) {
                return true;
            }
        }
        return false;
    }
}

impl MediaTypeOrganizer for PhotoOrganizer {
    fn should_organize(&self, item: &PathBuf) -> bool {
        let extension = item.extension().and_then(|e| e.to_str());
        match extension {
            Some(e) => PhotoOrganizer::is_supported(e),
            None => false,
        }
    }

    fn destination_dir(&self, item: &PathBuf) -> Result<PathBuf> {
        let photo_date = self.get_date(&item)?;
        return Ok(self
            .dst_dir
            .join(photo_date.get_year())
            .join(photo_date.get_month()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn should_organize() {
        let organizer = PhotoOrganizer::new(PathBuf::new());
        for extension in PhotoOrganizer::SUPPORTED.iter() {
            assert!(organizer.should_organize(&PathBuf::from(format!("file.{}", extension))));
        }
    }

    #[test]
    fn should_not_organize() {
        let organizer = PhotoOrganizer::new(PathBuf::new());
        let extensions = vec!["mp4", "doc", ""];
        for extension in extensions.iter() {
            assert!(!organizer.should_organize(&PathBuf::from(format!("file.{}", extension))));
        }
    }

    #[test]
    fn destination_dir_from_exif() {
        let src = TempDir::new().unwrap();
        let photo_dst = TempDir::new().unwrap().into_path();
        let dst = photo_dst.clone();

        let photo = PathBuf::from(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .join("camera.jpg");
        let sub_dir = src.path().join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();
        fs::copy(photo.clone(), sub_dir.join("camera.jpg")).unwrap();
        let photo_organizer = PhotoOrganizer::new(photo_dst);

        assert_eq!(
            dst.join("2019").join("01 - January").to_str().unwrap(),
            photo_organizer
                .destination_dir(&photo)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn destination_dir_from_filename() {
        let src = TempDir::new().unwrap();
        let photo_dst = TempDir::new().unwrap().into_path();
        let dst = photo_dst.clone();

        let photo = PathBuf::from(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .join("IMG-20200407-WA0004.jpg");
        let sub_dir = src.path().join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();
        fs::copy(photo.clone(), sub_dir.join("camera.jpg")).unwrap();
        let photo_organizer = PhotoOrganizer::new(photo_dst);

        assert_eq!(
            dst.join("2020").join("04 - April").to_str().unwrap(),
            photo_organizer
                .destination_dir(&photo)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}
