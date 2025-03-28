use crate::task::ScanTask;

/// - Scan a directory recursively for files with specific extensions.
pub fn scan_directory(
    dir_path: &std::path::Path,
    scan_task: &mut ScanTask,
) -> std::io::Result<Vec<std::path::PathBuf>> {
    if !dir_path.is_dir() {
        tracing::error!("Directory not found: {:?}", dir_path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Directory not found",
        ));
    }

    let mut files = vec![];

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively scan subdirectory
            scan_task.dir_count += 1;
            files.extend(scan_directory(&path, scan_task)?);
        } else if path.is_file() {
            // Check if file extension matches
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if scan_task.extensions.is_empty()
                    || scan_task
                        .extensions
                        .iter()
                        .any(|e| &ext_str == e || e == "*")
                {
                    files.push(path.to_owned());
                }
            }
        }
    }

    Ok(files)
}
