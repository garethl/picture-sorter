use super::SpecialHandler;
use crate::exiftool::adjust_canonicalization;
use crate::exiftool::Exif;
use crate::options::SortMode;
use crate::picture::Picture;
use crate::temp::TempFileTracker;
use anyhow::Error;

use log::warn;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::remove_file;
use std::fs::rename;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

#[derive(Default)]
pub struct MotionPhoto {}

enum MotionMode {
    Motion { is_heic: bool },
    Embedded,
}

const KEY_MOTION_PHOTO_VIDEO: &str = "MotionPhotoVideo";
const KEY_EMBEDDED_VIDEO_FILE: &str = "EmbeddedVideoFile";
const KEY_EMBEDDED_VIDEO_FILE_TRAILER: &str = "trailer:all";

impl MotionMode {
    fn get_video_key_to_extract(&self) -> &'static str {
        match self {
            MotionMode::Motion { is_heic: _ } => KEY_MOTION_PHOTO_VIDEO,
            MotionMode::Embedded => KEY_EMBEDDED_VIDEO_FILE,
        }
    }

    fn get_video_key_to_strip(&self) -> &'static str {
        match self {
            MotionMode::Motion { is_heic } => {
                if *is_heic {
                    KEY_MOTION_PHOTO_VIDEO
                } else {
                    KEY_EMBEDDED_VIDEO_FILE_TRAILER
                }
            }
            MotionMode::Embedded => KEY_EMBEDDED_VIDEO_FILE_TRAILER,
        }
    }
}

impl SpecialHandler for MotionPhoto {
    fn name(&self) -> &'static str {
        "Motion Photo v1"
    }

    fn can_handle(
        &self,
        picture: &Picture,
        destination: &Path,
        destination_exists: bool,
        overwrite: bool,
        _mode: &SortMode,
    ) -> bool {
        if !overwrite && destination_exists {
            return false;
        }

        if picture_is_motion_photo(picture) {
            let motion_video_file =
                change_file_name_with_new_extension(destination, "_motion", "mp4");

            if !overwrite && motion_video_file.exists() {
                warn!(
                    "Not processing {}, the extracted motion file already exists.",
                    picture.short_path
                );
                return false;
            }

            return true;
        }

        false
    }

    fn handle(
        &self,
        picture: &Picture,
        destination: &Path,
        _destination_exists: bool,
        _overwrite: bool,
        mode: &SortMode,
    ) -> Result<(), Error> {
        let mut temp_files = TempFileTracker::new();

        let temp_dir = env::temp_dir();
        let temp_dir = destination.parent().unwrap_or(&temp_dir);

        let file_prefix = destination.file_stem().unwrap_or(OsStr::new(""));
        let motion_video_file = change_file_name_with_new_extension(destination, "_motion", "mp4");

        let motion_mode = get_motion_mode(picture);

        // step 1: extract the video file
        let temp_video_path = temp_files.with_prefix_in(file_prefix, temp_dir);
        let temp_video = File::create(&temp_video_path)?;
        Exif::execute(
            vec![
                OsStr::new("-m"),
                OsStr::new("-b"),
                OsStr::new(&format!("-{}", motion_mode.get_video_key_to_extract())),
                adjust_canonicalization(&picture.path).as_os_str(),
            ],
            Some(temp_video.into()),
        )?;

        // step 2: copy the file using exiftool, and remove the baked in video file
        let temp_picture = temp_files.with_prefix_in(file_prefix, temp_dir);

        Exif::execute(
            vec![
                OsStr::new("-m"),
                OsStr::new("-U"),
                OsStr::new("-o"),
                adjust_canonicalization(&temp_picture).as_os_str(),
                OsStr::new(&format!("-{}=", motion_mode.get_video_key_to_strip())),
                adjust_canonicalization(&picture.path).as_os_str(),
            ],
            None,
        )?;

        rename(&temp_video_path, motion_video_file)?;
        rename(&temp_picture, destination)?;

        match mode {
            SortMode::Copy => {
                // we've extracted both files, so nothing else to do
            }
            SortMode::Move => {
                // we've extracted both files, so should delete the original
                remove_file(&picture.path)?;
            }
            SortMode::HardLink => {
                // since this special handler writes different source files,
                //  hardlinking isn't an option
            }
        }

        Ok(())
    }
}

fn get_motion_mode(picture: &Picture) -> MotionMode {
    if picture.metadata.get("motionphotovideo").is_some() {
        if let Some(mime) = picture.metadata.get("mimetype") {
            return MotionMode::Motion {
                is_heic: mime == "image/heic",
            };
        }
        return MotionMode::Motion { is_heic: false };
    }

    MotionMode::Embedded
}

fn picture_is_motion_photo(picture: &Picture) -> bool {
    let is_motionphoto = picture
        .metadata
        .get("motionphoto")
        .is_some_and(|v| v == "1");
    if !is_motionphoto {
        return false;
    }

    let is_supported_versino = picture
        .metadata
        .get("motionphotoversion")
        .is_some_and(|v| v == "1");
    if !is_supported_versino {
        return false;
    }

    let has_binary_video = picture
        .metadata
        .get("motionphotovideo")
        .or_else(|| picture.metadata.get("embeddedvideofile"))
        .is_some_and(|v| v.contains("Binary data"));
    if !has_binary_video {
        return false;
    }

    true
}

fn change_file_name_with_new_extension(path: &Path, suffix: &str, extension: &str) -> PathBuf {
    let parent = path.parent();

    let mut name = path.file_stem().unwrap_or(OsStr::new("")).to_os_string();
    name.push(OsString::from(suffix));
    if !extension.is_empty() {
        name.push(OsString::from("."));
        name.push(OsString::from(extension));
    }

    match parent {
        Some(parent) => parent.join(name),
        None => name.into(),
    }
}
