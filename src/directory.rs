use std::fs;
use std::path::PathBuf;

/// Iterator over the files and subdirecotires of a given root
/// directory. It uses a breath depth approach. It doesn't follow
/// symlinks.
pub struct FilesIter {
    dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
}

impl FilesIter {
    pub fn new(dir: PathBuf) -> FilesIter {
        FilesIter {
            dirs: vec![dir],
            files: Vec::new(),
        }
    }
}

impl Iterator for FilesIter {
    type Item = PathBuf;

    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        if let Some(file) = self.files.pop() {
            return Some(file);
        }

        while let Some(dir) = self.dirs.pop() {
            let dir_entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                _ => continue,
            };

            for entry in dir_entries {
                let path = match entry {
                    Ok(e) => e.path(),
                    _ => continue,
                };
                // Don't follow symlinks
                if path.read_link().is_ok() {
                    continue;
                }
                if path.is_dir() {
                    self.dirs.push(path);
                    continue;
                }
                if !path.is_file() {
                    continue;
                }
                self.files.push(path);
            }

            if let Some(file) = self.files.pop() {
                return Some(file);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn dir_traversal() {
        let src = TempDir::new().unwrap();
        fs::File::create(src.path().join("file1.png")).unwrap();
        let dir_path = src.path().join("dir");
        fs::DirBuilder::new().create(&dir_path).unwrap();
        fs::File::create(dir_path.join("file2.png")).unwrap();

        let files_iter = FilesIter::new(src.path().to_owned());
        let files: Vec<PathBuf> = files_iter.collect();
        assert_eq!(
            vec!(src.path().join("file1.png"), dir_path.join("file2.png")),
            files
        );
    }
}
