use super::SpecialHandler;
use crate::exiftool::adjust_canonicalization;
use crate::exiftool::Exif;
use crate::picture::Picture;
use crate::temp::TempFileTracker;
use anyhow::Error;
use log::warn;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::rename;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

#[derive(Default)]
pub struct MotionPhoto {}

impl SpecialHandler for MotionPhoto {
    fn name(&self) -> &'static str {
        "Motion Photo v1"
    }

    fn can_handle(&self, picture: &Picture, destination: &Path, destination_exists: bool) -> bool {
        if destination_exists {
            return false;
        }

        if picture_is_motion_photo(picture) {
            let motion_video_file =
                change_file_name_with_new_extension(destination, "_motion", "mp4");

            if motion_video_file.exists() {
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
    ) -> Result<(), Error> {
        let mut temp_files = TempFileTracker::new();

        let temp_dir = env::temp_dir();
        let temp_dir = destination.parent().unwrap_or(&temp_dir);

        let file_prefix = destination.file_stem().unwrap_or(OsStr::new(""));
        let motion_video_file = change_file_name_with_new_extension(destination, "_motion", "mp4");

        // step 1: extract the video file
        let temp_video_path = temp_files.with_prefix_in(file_prefix, temp_dir);
        let temp_video = File::create(&temp_video_path)?;
        Exif::execute(
            vec![
                OsStr::new("-m"),
                OsStr::new("-b"),
                OsStr::new("-MotionPhotoVideo"),
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
                OsStr::new("-MotionPhotoVideo="),
                adjust_canonicalization(&picture.path).as_os_str(),
            ],
            None,
        )?;

        rename(&temp_video_path, motion_video_file)?;
        rename(&temp_picture, destination)?;

        Ok(())
    }
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
