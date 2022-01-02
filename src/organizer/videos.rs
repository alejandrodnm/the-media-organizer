use super::MediaTypeOrganizer;
use crate::date::Date;
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;
use std::path::{Path, PathBuf};

/// It organizes videos in directories by year. The year is taken from
/// the file name using the regex `^(?:VID[-_])?(\d{4})(\d{2})\d{2}[_-].+\.mp4$`,
/// which basically translate to `VID-YYYYMMDD-whatever.mp4` where
/// `VID-` is optional and `-` can be changed to `_`.
pub struct VideoOrganizer {
    dst_dir: PathBuf,
    date_from_filename_regex: Regex,
}

impl VideoOrganizer {
    const SUPPORTED: [&'static str; 1] = ["mp4"];

    pub fn new(dst_dir: PathBuf) -> VideoOrganizer {
        VideoOrganizer {
            dst_dir,
            date_from_filename_regex: Regex::new(r"^(?:VID[-_])?(\d{4})(\d{2})\d{2}[_-].+\.mp4$")
                .unwrap(),
        }
    }

    fn get_date(&self, video: &Path) -> Result<Date> {
        let file_name = video
            .file_name()
            .ok_or_else(|| eyre!("failed to read file name"))?
            .to_str()
            .ok_or_else(|| eyre!("failed to get date as string"))?;

        let captures = self
            .date_from_filename_regex
            .captures(file_name)
            .ok_or_else(|| eyre!("file name doesn't contain date in the format YYYYMMDD"))?;
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

    fn is_supported(extension: &str) -> bool {
        for i in VideoOrganizer::SUPPORTED.iter() {
            if extension.eq(*i) {
                return true;
            }
        }
        false
    }
}

impl MediaTypeOrganizer for VideoOrganizer {
    fn should_organize(&self, item: &Path) -> bool {
        let extension = item.extension().and_then(|e| e.to_str());
        match extension {
            Some(e) => VideoOrganizer::is_supported(e),
            None => false,
        }
    }

    fn destination_dir(&self, item: &Path) -> Result<PathBuf> {
        let video_date = self
            .get_date(item)
            .wrap_err("failed to generate destination dir")?;
        Ok(self.dst_dir.join(video_date.get_year()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn should_organize() {
        let organizer = VideoOrganizer::new(PathBuf::new());
        for extension in VideoOrganizer::SUPPORTED.iter() {
            assert!(organizer.should_organize(&PathBuf::from(format!("file.{}", extension))));
        }
    }

    #[test]
    fn should_not_organize() {
        let organizer = VideoOrganizer::new(PathBuf::new());
        let extensions = vec!["jpg", "doc", ""];
        for extension in extensions.iter() {
            assert!(!organizer.should_organize(&PathBuf::from(format!("file.{}", extension))));
        }
    }

    #[test]
    fn destination_dir() {
        let src = TempDir::new().unwrap();
        let video_dst = TempDir::new().unwrap().into_path();
        let dst = video_dst.clone();

        let video = PathBuf::from(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .join("20200829_205420.mp4");
        let sub_dir = src.path().join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();
        fs::copy(&video, src.path().join("20200829_205420.mp4")).unwrap();
        let video_organizer = VideoOrganizer::new(video_dst);

        assert_eq!(
            dst.join("2020").to_str().unwrap(),
            video_organizer
                .destination_dir(&video)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}
