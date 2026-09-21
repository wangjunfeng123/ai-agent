use std::{fs, path::Path};

use crate::tools::read_images::r#impl::ReadImagesArgs;

pub fn analyze_images(arg: &ReadImagesArgs, model: &String) -> anyhow::Result<String> {
    let path = Path::new(&arg.file_path);
    if !path.exists() {
        anyhow::bail!("file not found path={}", path.display());
    }
    let bytes = fs::read(path);

    // 拼装扩展名
    let ext = match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    !todo!()
}
