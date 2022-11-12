use crate::util::ErrToStr;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error, Lines},
    path::{Path, PathBuf},
};

pub type FileLines = Lines<BufReader<File>>;

const SCENE_EXTENSION: &str = "unity";
const ASSET_EXTENSION: &str = "asset";
const PREFAB_EXTENSION: &str = "prefab";

#[derive(Debug)]
pub struct ProjectFiles {
    pub base_path: PathBuf,
    pub scenes: Vec<PathBuf>,
    pub assets: Vec<PathBuf>,
    pub prefabs: Vec<PathBuf>,
}

impl ProjectFiles {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            scenes: vec![],
            assets: vec![],
            prefabs: vec![],
        }
    }

    pub fn append(&mut self, other: &mut ProjectFiles) {
        self.scenes.append(&mut other.scenes);
        self.assets.append(&mut other.assets);
        self.prefabs.append(&mut other.prefabs);
    }

    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty() && self.assets.is_empty() && self.prefabs.is_empty()
    }
}

pub fn find_project_files(path: &Path) -> Result<ProjectFiles, Error> {
    let mut file_paths = ProjectFiles::new(path);

    for entry in path.read_dir()? {
        let path = entry?.path();

        if path.is_file() {
            if let Some(Some(s)) = path.extension().map(|os_str| os_str.to_str()) {
                match s {
                    SCENE_EXTENSION => file_paths.scenes.push(path),
                    ASSET_EXTENSION => file_paths.assets.push(path),
                    PREFAB_EXTENSION => file_paths.prefabs.push(path),
                    _ => {}
                }
            }
        } else if path.is_dir() {
            file_paths.append(&mut find_project_files(&path)?);
        }
    }

    Ok(file_paths)
}

pub fn get_file_lines(path: &Path) -> Result<FileLines, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

#[allow(dead_code)]
pub fn path_to_relative(full: &Path, base: &Path) -> Result<PathBuf, String> {
    full.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .err_to_str()
}

#[allow(dead_code)]
pub fn path_to_absolute(relative: &Path, base: &Path) -> PathBuf {
    base.join(relative)
}
