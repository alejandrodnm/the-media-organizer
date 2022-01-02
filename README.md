# Media Organizer

Organizes media files from a given directory into defined destinations using a
directory tree structure by dates.

Let's imagine you have a folder with some media items like:

```
$ tree /media-to-sort
 media-to-sort
 ├── images
 │  ├── camera
 │  │  └── camera-001.jpg
 │  └── whatsapp-images
 │     └── IMG-20200407-WA0004.jpg
 └── videos
    └── 20200829_205420.mp4

```

By running this project like:

```
./media-organizer --media-src /media-to-sort --photos-dst /my-photos --videos-dst /my-videos
```

The files will be sorted as:


```
$ tree /
 /
 ├── my-photos
 │   ├── 2019
 │   │  └── 01 - January
 │   │     └── camera-001.jpg
 │   └── 2020
 │      └── 04 - April
 │         └── IMG-20200407-WA0004.jpg
 └── my-videos
    └── 2020
       └── 20200829_205420.mp4
```

So what happened? Images where sort by year and month, the date is taken first
from the exif data, if the image has no exif then it's taken from the name.
Videos are organized by year, and the date is taken from the name.

Items are moved (not copied) from source to destination, and if a file with the
same name already exists in the destination and error will be shown for that
item and the process will continue with the next one.

## Installing

Publishing to be able to install with cargo is WIP. Just clone the repo and
execute:

```
media-organizer $ cargo run -- -h
media-organizer $ cargo run -- --media-src /media-to-sort --photos-dst /my-photos --videos-dst /my-videos
```

## How do I use it and why?

My workflow is that I copy my phones camera's directory, and some other media
folders like WhatsApp images or videos that I want to keep, to my laptop. I run
this program to have the photos organized into my Amazon Drive sync folder, the
videos are organized into another folder that's not in Amazon, then I rsync'd
both into an external hard drive.

Why organize videos and photos into different folders? Because I'm cheap and
Amazon gives me unlimited storage for images only.

Why make this? I wanted to learn Rust, basically. I used to organize all my
media manually, it was a really good experience since I got to view everything
again, but now my time is super limited and things were just piling up.

Why not just sync from the phone directly? I do have backups with Google Photos
but I like my workflow, and keeping things organize in my external hard drive.
I started doing things this way and I just went with it.

## Media type organizers

There are 2 media type organizers, one for photos and one for videos.

### Photo Organizer

It organizes photos in a 2 level directory structure where the first level is
the year and the second level the month. The date is taken from the exif of the
photo, if this fails or the image doesn't have exif, it tries to get the date
from the name.

Taking the date from the name is just a regex over the format that WhatsApp and
cameras use, which is `IMG-YYYYMMDD-WAXXXX.jpg` or `IMG_YYYYMMDD_XXXXX.jpg`.

Only the following formats are organized `jpeg`, `jpg` and `JPG`.

### Video Organizer

It organizes videos in directories by year. The year is taken from the file
name using the regex `^(?:VID[-_])?(\d{4})(\d{2})\d{2}[_-].+\.mp4$`, which
basically translate to `VID-YYYYMMDD-whatever.mp4` where `VID-` is optional and
`-` can be changed to `_`. Yes, it only supports `mp4` (PR's are welcome if you
want to change it).

## Configuration

It's required that a media source directory is specified and at least one of
photos or videos destination directories.

Options can be passed via command line arguments:

```
./media-organizer -h

USAGE:
    media-organizer [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                           Prints help information
        --no-load-default-config-file    Do not load the config file from the default location
    -V, --version                        Prints version information

OPTIONS:
    -c, --config-file <FILE>        File to load configuration from. Defaults to:
                                    - Linux: /home/alice/.config/media-organizer/config.toml
                                    - Windows: C:\Users\Alice\AppData\Roaming\adn\media-organizer\config\config.toml
                                    - Mac: /Users/Alice/Library/Application Support/dev.adn.media-organizer/config.toml
    -m, --media-src <DIRECTORY>     Source directory with media files to organize
    -p, --photos-dst <DIRECTORY>    Directory where photos will be moved and organized
    -v, --videos-dst <DIRECTORY>    Directory where videos will be moved and organized
```

For example:

```
./media-organizer --media-src /media-to-sort --photos-dst /my-photos --videos-dst /my-videos
```

Or using a TOML configuration file:

```
./media-organizer -c my-config.toml
```

Where `my-config.toml` is:

```
$ cat my-config.toml

media_src = '/media-to-sort'
photos_dst = '/my-photos'
videos_dst = '/my-videos'
```

If both command line arguments and a file are specified, the command line
arguments will take precedence. Also, if no file is specified with the
`--config-file` option, a default configuration file will try to be loaded
from:

- Linux: `/home/alice/.config/media-organizer/config.toml`
- Windows: `C:\Users\Alice\AppData\Roaming\adn\media-organizer\config\config.toml`
- Mac: `/Users/Alice/Library/Application Support/dev.adn.media-organizer/config.toml`

Loading the default configuration file can be disable with the
`--no-load-default-config-file` flag.

## Testing

Just run `cargo test`, nothing fancy here.

## Contributing

PR's and issues are welcome.

## Limitations

This was tailored made to fit my needs and as a learning exercise, so there are
some things that are not very flexible:

- List of supported file formats per media type organizer.
- Format to get the dates from file names.
- Directory structures and names of destinations.
- It just support images and videos.
