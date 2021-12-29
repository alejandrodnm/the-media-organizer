use clap;
use color_eyre::eyre::{bail, eyre, Result, WrapErr};
use directories::ProjectDirs;
use std::ffi::OsString;
use std::path::PathBuf;
use viperus::{Format, Viperus};

/// Loads the configuration options.
///
/// The configuration can be set via command line arguments or via
/// config file using the TOML format. The command line arguments have
/// precedence over the configuration file in cases where both are
/// specified.
///
/// The available configuration options are:
///
/// - Config file: file to load configuration from.
///     - cmd line long: --config-file
///     - cmd line short: -c
///   Defaults to:
///     - Linux: /home/ainara/.config/media-organizer/config.toml
///     - Windows: C:\\Users\\Ainara\\AppData\\Roaming\\adn\\media-organizer\\config\\config.toml
///     - Mac: /Users/Ainara/Library/Application Support/dev.adn.media-organizer/config.toml",
/// - Media source: Source directory with media files to organize.
///     - cmd line long: --media-src
///     - cmd short: -m
///     - toml: media_src
/// - Photos destination: Directory where photos will be moved and organized.
///     - cmd line long: --photos-dst
///     - cmd short: -p
///     - toml: photos_dst
/// - Videos destination: Directory where videos will be moved and organized.
///     - cmd line long: --videos-dst
///     - cmd short: -v
///     - toml: videos_dst
/// - No load default config file: Do not load the config file from the default location.
///     - cmd line long: --no-load-default-config-file
pub fn get_config<I, T>(cmd_args: I) -> Result<Config>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut v = Viperus::new();
    let should_load_default_config_file = load_claps(&mut v, cmd_args)
        .wrap_err_with(|| eyre!("failed to load command line arguments"))?;

    let config_file_loaded = match v.get::<String>("config_file") {
        Some(config_file) => {
            if let Err(e) = v.load_file(&config_file, Format::TOML) {
                bail!("failed to load config file '{}': {}", config_file, e);
            }
            true
        }
        None => false,
    };

    if !config_file_loaded && should_load_default_config_file {
        if let Some(config_file) = get_default_config_file() {
            if let Err(e) = v.load_file(&config_file, Format::TOML) {
                bail!("failed to load config file '{}': {}", config_file, e);
            }
        }
    }

    let mut config_builder = match v.get::<String>("media_src") {
        Some(dir) => ConfigBuilder::new(dir),
        None => bail!("media source is required"),
    };

    config_builder = match v.get::<String>("photos_dst") {
        Some(dir) => config_builder.with_photos_dst(dir),
        None => config_builder,
    };

    config_builder = match v.get::<String>("videos_dst") {
        Some(dir) => config_builder.with_videos_dst(dir),
        None => config_builder,
    };

    config_builder.build()
}

/// Consolidates the configuration of both command line arguments and
/// the what's specified in the configuration file.
#[derive(Debug)]
pub struct Config {
    pub media_src: PathBuf,
    pub photos_dst: PathBuf,
    pub videos_dst: PathBuf,
}

impl Config {
    /// Creates a new Config object. It validates that the given path point
    /// to existing directories and that at least one of photos_dst_str or
    /// videos_dst_str are not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let valid_dir = PathBuf::from(file!()).parent().unwrap().to_string();
    /// let config = Config::new(valid_dir, valid_dir, valid_dir);
    /// assert!(config.is_ok());
    /// ```
    fn new(
        media_src_str: String,
        photos_dst_str: String,
        videos_dst_str: String,
    ) -> Result<Config> {
        let media_src = PathBuf::from(media_src_str);
        if !media_src.is_dir() {
            bail!("media source dir doesn't exist");
        }

        if photos_dst_str.is_empty() && videos_dst_str.is_empty() {
            bail!("at least one of photos_dst or videos_dst shouldn't be empty");
        }

        let photos_dst = if !photos_dst_str.is_empty() {
            let path = PathBuf::from(photos_dst_str);
            if !path.is_dir() {
                bail!("photos destination dir doesn't exist");
            }
            path
        } else {
            PathBuf::new()
        };

        let videos_dst = if !videos_dst_str.is_empty() {
            let path = PathBuf::from(videos_dst_str);
            if !path.is_dir() {
                bail!("videos destination dir doesn't exist");
            }
            path
        } else {
            PathBuf::new()
        };

        Ok(Config {
            media_src,
            photos_dst,
            videos_dst,
        })
    }
}

struct ConfigBuilder {
    media_src_str: String,
    photos_dst_str: String,
    videos_dst_str: String,
}

impl ConfigBuilder {
    fn new(media_src_str: String) -> ConfigBuilder {
        ConfigBuilder {
            media_src_str,
            photos_dst_str: "".to_owned(),
            videos_dst_str: "".to_owned(),
        }
    }

    fn with_photos_dst(mut self, photos_dst_str: String) -> ConfigBuilder {
        self.photos_dst_str = photos_dst_str;
        self
    }

    fn with_videos_dst(mut self, videos_dst_str: String) -> ConfigBuilder {
        self.videos_dst_str = videos_dst_str;
        self
    }

    fn build(self) -> Result<Config> {
        Config::new(self.media_src_str, self.photos_dst_str, self.videos_dst_str)
    }
}

fn get_default_config_file() -> Option<String> {
    let config_dir = match ProjectDirs::from("dev", "adn", "media-organizer")
        .map(|dirs: ProjectDirs| dirs.config_dir().to_owned())
    {
        Some(config_dir) => config_dir,
        None => return None,
    };

    if !config_dir.is_dir() {
        return None;
    }

    let config_file = config_dir.join("config.toml");

    if !config_file.is_file() {
        return None;
    }

    config_file.to_str().map(|s| s.to_owned())
}

fn load_claps<I, T>(v: &mut Viperus, cmd_args: I) -> Result<bool>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = clap::App::new("media-organizer")
        .arg(
            clap::Arg::with_name("config_file")
                .short("c")
                .long("config-file")
                .value_name("FILE")
                .long_help(
                    "\
File to load configuration from. Defaults to:
- Linux: /home/ainara/.config/media-organizer/config.toml
- Windows: C:\\Users\\Ainara\\AppData\\Roaming\\adn\\media-organizer\\config\\config.toml
- Mac: /Users/Ainara/Library/Application Support/dev.adn.media-organizer/config.toml",
                )
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("media_src")
                .short("m")
                .long("media-src")
                .value_name("DIRECTORY")
                .help("Source directory with media files to organize")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("photos_dst")
                .short("p")
                .long("photos-dst")
                .value_name("DIRECTORY")
                .help("Directory where photos will be moved and organized")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("videos_dst")
                .short("v")
                .long("videos-dst")
                .value_name("DIRECTORY")
                .help("Directory where videos will be moved and organized")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("no_load_default_config_file")
                .long("no-load-default-config-file")
                .help("Do not load the config file from the default location"),
        )
        .get_matches_from(cmd_args);

    let no_load_default_config = matches.is_present("no_load_default_config_file");
    if let Err(e) = v.load_clap(matches) {
        bail!("{}", e);
    }
    Ok(!no_load_default_config)
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_config_from_file() {
        let config_file_dir = tempdir().unwrap();
        let config_file_path = config_file_dir.path().join("config.toml");
        let photos_dst = tempdir().unwrap();
        let videos_dst = tempdir().unwrap();
        let media_src = tempdir().unwrap();

        fs::write(
            &config_file_path,
            format!(
                "photos_dst='{}'\nmedia_src='{}'\nvideos_dst='{}'",
                photos_dst.path().to_str().unwrap(),
                media_src.path().to_str().unwrap(),
                videos_dst.path().to_str().unwrap(),
            ),
        )
        .unwrap();
        let config = get_config(vec!["self", "-c", config_file_path.to_str().unwrap()]).unwrap();
        assert_eq!(config.media_src, media_src.path());
        assert_eq!(config.photos_dst, photos_dst.path());
        assert_eq!(config.videos_dst, videos_dst.path());
    }

    #[test]
    fn load_config_from_cmd_line_args() {
        let photos_dst = tempdir().unwrap();
        let videos_dst = tempdir().unwrap();
        let media_src = tempdir().unwrap();

        let config = get_config(vec![
            "self",
            "-m",
            media_src.path().to_str().unwrap(),
            "-p",
            photos_dst.path().to_str().unwrap(),
            "-v",
            videos_dst.path().to_str().unwrap(),
        ])
        .unwrap();
        assert_eq!(config.media_src, media_src.path());
        assert_eq!(config.photos_dst, photos_dst.path());
        assert_eq!(config.videos_dst, videos_dst.path());
    }

    #[test]
    fn missing_both_videos_and_photos_err() {
        let media_src = tempdir().unwrap();
        let err = get_config(vec![
            "self",
            "-m",
            media_src.path().to_str().unwrap(),
            "--no-load-default-config-file",
        ])
        .unwrap_err();

        assert_eq!(
            "at least one of photos_dst or videos_dst shouldn't be empty",
            err.to_string(),
        )
    }

    #[test]
    fn cmd_line_takes_precedence_over_file() {
        let config_file_dir = tempdir().unwrap();
        let config_file_path = config_file_dir.path().join("config.toml");
        let photos_dst_file = tempdir().unwrap();
        let videos_dst_file = tempdir().unwrap();
        let media_src_file = tempdir().unwrap();

        fs::write(
            &config_file_path,
            format!(
                "photos_dst='{}'\nmedia_src='{}'\nvideos_dst='{}'",
                photos_dst_file.path().to_str().unwrap(),
                media_src_file.path().to_str().unwrap(),
                videos_dst_file.path().to_str().unwrap(),
            ),
        )
        .unwrap();

        let photos_dst_cmd = tempdir().unwrap();
        let media_src_cmd = tempdir().unwrap();

        let config = get_config(vec![
            "self",
            "-m",
            media_src_cmd.path().to_str().unwrap(),
            "-p",
            photos_dst_cmd.path().to_str().unwrap(),
            "-c",
            config_file_path.to_str().unwrap(),
        ])
        .unwrap();
        assert_eq!(config.media_src, media_src_cmd.path());
        assert_eq!(config.photos_dst, photos_dst_cmd.path());
        // videos_dst is not on the cmd line args and it's taken from the
        // config file.
        assert_eq!(config.videos_dst, videos_dst_file.path());
    }
}
