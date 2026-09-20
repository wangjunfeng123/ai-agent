use std::{fs, path::Path};

// 根据路径获取路径下的所有文件
pub fn list_file(path: &str) -> anyhow::Result<String> {
    let path = Path::new(path);
    if !path.exists() {
        anyhow::bail!("file not found path={}", path.display());
    }
    if !path.is_dir() {
        anyhow::bail!("not a directory path={}", path.display());
    }

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(".") {
            continue;
        }
        if entry.file_type()?.is_dir() {
            dirs.push(format!("{name}/"));
        } else {
            files.push(name);
        }
    }
    dirs.sort();
    files.sort();

    let mut result = format!("Directory:{}\n", path.display());

    for item in dirs.into_iter().chain(files) {
        result.push_str(&format!(".{item}\n"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::list_file;
    use std::fs;

    fn setup_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("list_file_test_{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.txt"), "b").unwrap();
        fs::write(dir.join(".hidden"), "h").unwrap();
        dir
    }

    #[test]
    fn test_list_file_returns_files_and_dirs() {
        let dir = setup_dir();
        let out = list_file(dir.to_str().unwrap()).unwrap();

        assert!(out.starts_with(&format!("Directory:{}\n", dir.display())));
        assert!(out.contains("a.txt\n"));
        assert!(out.contains("b.txt\n"));
        assert!(out.contains("sub/\n"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_list_file_skips_hidden_entries() {
        let dir = setup_dir();
        let out = list_file(dir.to_str().unwrap()).unwrap();

        assert!(!out.contains(".hidden"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_list_file_on_missing_path() {
        let dir = std::env::temp_dir().join("list_file_missing_test");
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        let err = list_file(dir.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_list_file_on_file_not_dir() {
        let dir = setup_dir();
        let file = dir.join("a.txt");
        let err = list_file(file.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("not a directory"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
