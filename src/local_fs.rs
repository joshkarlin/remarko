use std::path::Path;

use crate::nodes::{Directory, DirectoryNode, File, Hash, Metadata, Node};

pub fn build_local_directory(path: &Path) -> Result<Directory, std::io::Error> {
    let mut local_files = Vec::new();
    let mut local_directories = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        // skip hidden files or directories
        if name.starts_with(".") {
            continue;
        }

        if path.is_file() {
            let metadata = Metadata::new(name, None, "420".into(), "file".into());
            local_files.push(File::new(
                Hash::new("3rr93jfierjf-9erijfe0".into()),
                metadata,
            ));
        } else if path.is_dir() {
            local_directories.push(build_local_directory(&path)?);
        }
    }

    Ok(Directory::new(
        Hash::new("e11o-420-69".into()),
        Metadata::new(
            path.file_name().unwrap().to_string_lossy().into_owned(),
            None,
            "420".into(),
            "directory".into(),
        ),
        Some(local_files),
        Some(local_directories),
    ))
}

pub fn remove_common_files_and_directories(
    dir1: &Directory,
    dir2: &Directory,
) -> (Directory, Directory) {
    let mut dir1_diff_files = Vec::new();
    let mut dir2_diff_files = Vec::new();
    let mut dir1_diff_directories = Vec::new();
    let mut dir2_diff_directories = Vec::new();

    let dir1_files: Vec<_> = dir1.get_files().iter().map(|f| f.get_file_stem()).collect();

    let dir2_files: Vec<_> = dir2.get_files().iter().map(|f| f.get_file_stem()).collect();

    for file in dir1.get_files() {
        if !dir2_files.contains(&file.get_file_stem()) {
            dir1_diff_files.push(file.clone());
        }
    }

    for file in dir2.get_files() {
        if !dir1_files.contains(&file.get_file_stem()) {
            dir2_diff_files.push(file.clone());
        }
    }

    // Recursively compare sub-directories
    for subdir1 in dir1.get_directories() {
        if let Some(subdir2) = dir2
            .get_directories()
            .iter()
            .find(|d| d.get_visible_name() == subdir1.get_visible_name())
        {
            let (sub_diff1, sub_diff2) = remove_common_files_and_directories(subdir1, subdir2);

            if !sub_diff1.get_files().is_empty() || !sub_diff1.get_directories().is_empty() {
                dir1_diff_directories.push(sub_diff1);
            }
            if !sub_diff2.get_files().is_empty() || !sub_diff2.get_directories().is_empty() {
                dir2_diff_directories.push(sub_diff2);
            }
        } else {
            dir1_diff_directories.push(subdir1.clone());
        }
    }

    for subdir2 in dir2.get_directories() {
        if dir1
            .get_directories()
            .iter()
            .find(|d| d.get_visible_name() == subdir2.get_visible_name())
            .is_none()
        {
            dir2_diff_directories.push(subdir2.clone());
        }
    }

    let diff_dir1 = Directory::new(
        dir1.get_hash().clone(),
        dir1.get_metadata().clone(),
        Some(dir1_diff_files),
        Some(dir1_diff_directories),
    );

    let diff_dir2 = Directory::new(
        dir2.get_hash().clone(),
        dir2.get_metadata().clone(),
        Some(dir2_diff_files),
        Some(dir2_diff_directories),
    );

    (diff_dir1, diff_dir2)
}

pub fn compare_directories(dir1: &Directory, dir2: &Directory) -> Vec<String> {
    let mut diffs = Vec::new();

    let dir1_files: Vec<_> = dir1
        .get_files()
        .iter()
        .map(|f| f.get_visible_name().as_str())
        .collect();
    let dir2_files: Vec<_> = dir2
        .get_files()
        .iter()
        .map(|f| f.get_visible_name().as_str())
        .collect();

    for file in &dir1_files {
        if !dir2_files.contains(&file) {
            diffs.push(format!(
                "File {} is missing in {}.",
                file,
                dir2.get_visible_name()
            ));
        }
    }

    for file in &dir2_files {
        if !dir1_files.contains(&file) {
            diffs.push(format!(
                "File {} is missing in {}.",
                file,
                dir1.get_visible_name()
            ));
        }
    }

    // Recursively compare sub-directories
    for subdir1 in dir1.get_directories() {
        if let Some(subdir2) = dir2
            .get_directories()
            .iter()
            .find(|d| d.get_visible_name() == subdir1.get_visible_name())
        {
            diffs.extend(compare_directories(subdir1, subdir2));
        } else {
            diffs.push(format!(
                "Directory {} is missing in {}.",
                subdir1.get_visible_name(),
                dir2.get_visible_name()
            ));
        }
    }

    for subdir2 in dir2.get_directories() {
        if dir1
            .get_directories()
            .iter()
            .find(|d| d.get_visible_name() == subdir2.get_visible_name())
            .is_none()
        {
            diffs.push(format!(
                "Directory {} is missing in {}.",
                subdir2.get_visible_name(),
                dir1.get_visible_name()
            ));
        }
    }

    diffs
}
