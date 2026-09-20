use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

// 解压缩
pub fn unzip(zip_path: &str, extract_to: Option<&str>) -> anyhow::Result<String> {
    let path = Path::new(zip_path);
    if !path.exists() {
        anyhow::bail!("file not found ={}", path.display());
    }

    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let dest = extract_to
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./"));
    fs::create_dir_all(&dest)?;

    let mut extracted = 0u32;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // 过滤非法路径（绝对路径 / `..` 逃逸），防止 Zip 路径穿越
        let Some(rel_path) = entry.enclosed_name() else {
            tracing::warn!("skip entry with unsafe path at index {i}");
            continue;
        };

        let out_path = dest.join(rel_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output = File::create(&out_path)?;
        io::copy(&mut entry, &mut output)?;
        extracted += 1;
    }

    Ok(format!(
        "extracted {} files into {}",
        extracted,
        dest.display()
    ))
}
