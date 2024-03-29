use gitignore;
use log::warn;
use sass_rs::{self, Options, OutputStyle};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub type Cache = Arc<RwLock<HashMap<String, String>>>;

pub fn compile(styles_dir: String) -> HashMap<String, String> {
    let styles_dir_path = PathBuf::from(styles_dir.clone())
        .canonicalize()
        .unwrap_or(PathBuf::from(format!("/sekond/{}", styles_dir)));

    let pathbufs: Vec<PathBuf> = match gitignore::File::new(&styles_dir_path.join(".gitignore"))
        .map(|gitignore_file| gitignore_file.included_files())
    {
        // Found .gitignore and getting files succeeded
        Ok(Ok(included_files)) => included_files,
        // Something went wrong so use everything
        _ => get_files(PathBuf::from(styles_dir.clone())),
    };

    let paths: Vec<String> = pathbufs
        .iter()
        .filter_map(|file| {
            let filename = file.display().to_string();

            if file
                .file_name()
                .map(|filename| filename.to_string_lossy().to_string())
                .unwrap_or("_".to_string())
                .starts_with("_")
            {
                // Do not include module files
                None
            } else if [".css", ".scss", ".sass"]
                .iter()
                .any(|ext| filename.ends_with(ext))
            {
                // Only include stylesheet files
                Some(filename)
            } else {
                None
            }
        })
        .collect();

    let mut compiled = HashMap::new();

    let include_paths: Vec<String> = paths.clone();
    for path in paths {
        match sass_rs::compile_file(
            path.as_str(),
            Options {
                output_style: OutputStyle::Compressed,
                precision: 3,
                indented_syntax: false,
                include_paths: include_paths.clone(),
            },
        ) {
            Ok(css) => {
                compiled.insert(
                    path.trim_start_matches(&styles_dir_path.display().to_string()) // Remove absolute path
                        .trim_start_matches("./styles/") // Relative path too
                        .trim_start_matches(".")
                        .trim_start_matches("/")
                        .trim_end_matches(".scss")
                        .replace("/", ":")
                        .to_string(),
                    css,
                );
            }
            Err(error) => warn!("Failed to compile this style file: {}", error),
        }
    }

    compiled
}

fn get_files(folder: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];

    if let Ok(list) = fs::read_dir(folder) {
        for entry in list {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    files.append(&mut get_files(path))
                } else {
                    files.push(path)
                }
            }
        }
    }

    files
}
