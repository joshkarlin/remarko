use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::any::Any;
use std::fmt;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl AsAny for File {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AsAny for Directory {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AsAny for SystemDirectory {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Hash(String);

impl Hash {
    pub fn new(hash: String) -> Self {
        Hash(hash)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    #[serde(rename = "visibleName")]
    visible_name: String,
    parent: Option<String>,
    #[serde(rename = "lastModified")]
    last_modified: String,
    #[serde(rename = "type")]
    pub type_: String,
}

pub trait Node: AsAny {
    fn get_hash(&self) -> &Hash;
    fn get_metadata(&self) -> &Metadata;
    fn get_visible_name(&self) -> &String;
    fn get_parent(&self) -> Option<&String>;
}

pub trait DirectoryNode: Node {
    fn get_directories(&self) -> &Vec<Directory>;
    fn get_files(&self) -> &Vec<File>;
    fn add_directory(&mut self, directory: Directory);
    fn add_file(&mut self, file: File);
}

#[derive(Clone, Debug)]
pub struct File {
    hash: Hash,
    metadata: Metadata,
}

#[derive(Clone, Debug)]
pub struct Directory {
    hash: Hash,
    metadata: Metadata,
    files: Vec<File>,
    directories: Vec<Directory>,
}

#[derive(Debug)]
pub struct SystemDirectory {
    name: String,
    files: Vec<File>,
    directories: Vec<Directory>,
}

impl Node for File {
    fn get_hash(&self) -> &Hash {
        &self.hash
    }

    fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn get_visible_name(&self) -> &String {
        &self.metadata.visible_name
    }

    fn get_parent(&self) -> Option<&String> {
        self.metadata.parent.as_ref()
    }
}

impl Node for Directory {
    fn get_hash(&self) -> &Hash {
        &self.hash
    }

    fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn get_visible_name(&self) -> &String {
        &self.metadata.visible_name
    }

    fn get_parent(&self) -> Option<&String> {
        self.metadata.parent.as_ref()
    }
}

impl Node for SystemDirectory {
    fn get_hash(&self) -> &Hash {
        unimplemented!()
    }

    fn get_metadata(&self) -> &Metadata {
        unimplemented!()
    }

    fn get_visible_name(&self) -> &String {
        &self.name
    }

    fn get_parent(&self) -> Option<&String> {
        None
    }
}

impl DirectoryNode for Directory {
    fn get_directories(&self) -> &Vec<Directory> {
        &self.directories
    }

    fn get_files(&self) -> &Vec<File> {
        &self.files
    }

    fn add_directory(&mut self, directory: Directory) {
        self.directories.push(directory);
    }

    fn add_file(&mut self, file: File) {
        self.files.push(file);
    }
}

impl DirectoryNode for SystemDirectory {
    fn get_directories(&self) -> &Vec<Directory> {
        &self.directories
    }

    fn get_files(&self) -> &Vec<File> {
        &self.files
    }

    fn add_directory(&mut self, directory: Directory) {
        self.directories.push(directory);
    }

    fn add_file(&mut self, file: File) {
        self.files.push(file);
    }
}

impl File {
    pub fn new(hash: Hash, metadata: Metadata) -> File {
        File { hash, metadata }
    }

    pub fn get_last_modified(&self) -> String {
        // TODO fix this to parse the timestamp correctly
        let naive_datetime = NaiveDateTime::from_timestamp_opt(
            self.metadata.last_modified.parse::<i64>().unwrap(),
            0,
        )
        .expect("Failed to parse timestamp");
        let utc_datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);
        utc_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// if file_name ends with ".pdf or .epub", remove that
    pub fn get_file_stem(&self) -> String {
        let file_name = self.get_visible_name();
        if file_name.ends_with(".pdf") || file_name.ends_with(".epub") {
            let split_file_name = file_name.split(".").collect::<Vec<&str>>();
            split_file_name[..split_file_name.len() - 1].join(".")
        } else {
            file_name.to_string()
        }
    }
}

impl Metadata {
    pub fn new(
        visible_name: String,
        parent: Option<String>,
        last_modified: String,
        type_: String,
    ) -> Metadata {
        Metadata {
            visible_name,
            parent,
            last_modified,
            type_,
        }
    }
}

impl Directory {
    pub fn new(
        hash: Hash,
        metadata: Metadata,
        files: Option<Vec<File>>,
        directories: Option<Vec<Directory>>,
    ) -> Directory {
        Directory {
            hash,
            metadata,
            files: files.unwrap_or(Vec::new()),
            directories: directories.unwrap_or(Vec::new()),
        }
    }
}

impl SystemDirectory {
    pub fn new(
        name: String,
        files: Option<Vec<File>>,
        directories: Option<Vec<Directory>>,
    ) -> SystemDirectory {
        SystemDirectory {
            name,
            files: files.unwrap_or(Vec::new()),
            directories: directories.unwrap_or(Vec::new()),
        }
    }
}
