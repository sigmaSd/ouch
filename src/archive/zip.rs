//! Contains Zip-specific building and unpacking functions

use std::{
    env,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

use fs_err as fs;
use walkdir::WalkDir;
use zip::{self, read::ZipFile, ZipArchive};

use crate::{
    error::FinalError,
    info,
    list::FileInArchive,
    utils::{
        cd_into_same_dir_as, clear_path, concatenate_os_str_list, dir_is_empty, get_invalid_utf8_paths, strip_cur_dir,
        to_utf, Bytes,
    },
    QuestionPolicy,
};

/// Unpacks the archive given by `archive` into the folder given by `into`.
pub fn unpack_archive<R>(
    mut archive: ZipArchive<R>,
    into: &Path,
    question_policy: QuestionPolicy,
) -> crate::Result<Vec<PathBuf>>
where
    R: Read + Seek,
{
    let mut unpacked_files = vec![];
    for idx in 0..archive.len() {
        let mut file = archive.by_index(idx)?;
        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let file_path = into.join(file_path);
        if !clear_path(&file_path, question_policy)? {
            // User doesn't want to overwrite
            continue;
        }

        check_for_comments(&file);

        match (&*file.name()).ends_with('/') {
            _is_dir @ true => {
                println!("File {} extracted to \"{}\"", idx, file_path.display());
                fs::create_dir_all(&file_path)?;
            }
            _is_file @ false => {
                if let Some(path) = file_path.parent() {
                    if !path.exists() {
                        fs::create_dir_all(&path)?;
                    }
                }
                let file_path = strip_cur_dir(file_path.as_path());

                info!("{:?} extracted. ({})", file_path.display(), Bytes::new(file.size()));

                let mut output_file = fs::File::create(&file_path)?;
                io::copy(&mut file, &mut output_file)?;
            }
        }

        #[cfg(unix)]
        __unix_set_permissions(&file_path, &file)?;

        let file_path = fs::canonicalize(&file_path)?;
        unpacked_files.push(file_path);
    }

    Ok(unpacked_files)
}

/// List contents of `archive`, returning a vector of archive entries
pub fn list_archive<R>(mut archive: ZipArchive<R>) -> crate::Result<Vec<FileInArchive>>
where
    R: Read + Seek,
{
    let mut files = vec![];
    for idx in 0..archive.len() {
        let file = archive.by_index(idx)?;

        let path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let is_dir = file.is_dir();

        files.push(FileInArchive { path, is_dir });
    }
    Ok(files)
}

/// Compresses the archives given by `input_filenames` into the file given previously to `writer`.
pub fn build_archive_from_paths<W, D>(input_filenames: &[PathBuf], writer: W, mut display_handle: D) -> crate::Result<W>
where
    W: Write + Seek,
    D: Write,
{
    let mut writer = zip::ZipWriter::new(writer);
    let options = zip::write::FileOptions::default();

    // Vec of any filename that failed the UTF-8 check
    let invalid_unicode_filenames = get_invalid_utf8_paths(input_filenames);

    if !invalid_unicode_filenames.is_empty() {
        let error = FinalError::with_title("Cannot build zip archive")
            .detail("Zip archives require files to have valid UTF-8 paths")
            .detail(format!("Files with invalid paths: {}", concatenate_os_str_list(&invalid_unicode_filenames)));

        return Err(error.into());
    }

    for filename in input_filenames {
        let previous_location = cd_into_same_dir_as(filename)?;

        // Safe unwrap, input shall be treated before
        let filename = filename.file_name().unwrap();

        for entry in WalkDir::new(filename) {
            let entry = entry?;
            let path = entry.path();

            write!(display_handle, "Compressing '{}'.", to_utf(path)).unwrap();
            display_handle.flush().unwrap();

            if path.is_dir() {
                if dir_is_empty(path) {
                    writer.add_directory(path.to_str().unwrap().to_owned(), options)?;
                }
                // If a dir has files, the files are responsible for creating them.
            } else {
                writer.start_file(path.to_str().unwrap().to_owned(), options)?;
                let file_bytes = fs::read(entry.path())?;
                writer.write_all(&*file_bytes)?;
            }
        }

        env::set_current_dir(previous_location)?;
    }

    let bytes = writer.finish()?;
    Ok(bytes)
}

fn check_for_comments(file: &ZipFile) {
    let comment = file.comment();
    if !comment.is_empty() {
        info!("Found comment in {}: {}", file.name(), comment);
    }
}

#[cfg(unix)]
fn __unix_set_permissions(file_path: &Path, file: &ZipFile) -> crate::Result<()> {
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};

    if let Some(mode) = file.unix_mode() {
        fs::set_permissions(file_path, Permissions::from_mode(mode))?;
    }

    Ok(())
}
