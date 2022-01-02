mod config;
use ::the_media_organizer::{MediaTypeOrganizer, Organizer, PhotoOrganizer, VideoOrganizer};
use color_eyre::eyre::{bail, Result, WrapErr};
use std::env;

/// Loads the config and runs the organizers
fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;

    let config = config::get_config(env::args_os()).wrap_err("error getting config")?;
    println!("Media Organizer configuration loaded");

    match config.media_src.to_str() {
        Some(dir) => println!("Media source directory: {}", dir),
        None => bail!("media source directory is not a valid unicode path"),
    }
    let mut organizers: Vec<Box<dyn MediaTypeOrganizer>> = Vec::new();

    match config.photos_dst.to_str() {
        Some(dir) => {
            println!(
                "Photo organizer enable, photos will be organized in directory: {}",
                dir
            );
            organizers.push(Box::new(PhotoOrganizer::new(config.photos_dst)));
        }
        None => bail!("media source directory is not a valid unicode path"),
    };

    match config.videos_dst.to_str() {
        Some(dir) => {
            println!(
                "Video organizer enable, videos will be organized in directory: {}",
                dir
            );
            organizers.push(Box::new(VideoOrganizer::new(config.videos_dst)));
        }
        None => bail!("media source directory is not a valid unicode path"),
    }
    let organizer = Organizer::new(organizers);
    organizer.organize(config.media_src)
}
