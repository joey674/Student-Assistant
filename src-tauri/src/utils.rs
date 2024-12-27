use std::path::PathBuf;

pub fn has_wav_files(dir_path: &PathBuf) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "wav" {
                            println!("Found a .wav file: {}", &path.display());
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    return None;
}
