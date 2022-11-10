use crate::util::ErrToStr;
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::PathBuf,
};

pub type FileLines = Lines<BufReader<File>>;

const SCENE_EXTENSION: &str = "unity";
const ASSET_EXTENSION: &str = "asset";
const PREFAB_EXTENSION: &str = "prefab";

#[derive(Default, Debug)]
pub struct ProjectFiles {
    pub scenes: Vec<PathBuf>,
    pub assets: Vec<PathBuf>,
    pub prefabs: Vec<PathBuf>,
}

impl ProjectFiles {
    pub fn append(&mut self, other: &mut ProjectFiles) {
        self.scenes.append(&mut other.scenes);
        self.assets.append(&mut other.assets);
        self.prefabs.append(&mut other.prefabs);
    }
}

pub fn find_project_files(path: &PathBuf) -> Result<ProjectFiles, String> {
    let mut file_paths = ProjectFiles::default();

    for entry in path.read_dir().err_to_str()? {
        let path = entry.err_to_str()?.path();

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

pub fn get_file_lines(path: &PathBuf) -> Result<FileLines, String> {
    let file = File::open(path).err_to_str()?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

pub fn path_to_relative(full: &PathBuf, base: &PathBuf) -> Result<PathBuf, String> {
    full.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .err_to_str()
}

pub fn path_to_absolute(relative: &PathBuf, base: &PathBuf) -> PathBuf {
    base.join(relative)
}
