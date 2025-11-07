// File validation utilities

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100 MB

pub fn validate_file_size(size: usize) -> Result<(), String> {
    if size > MAX_FILE_SIZE {
        return Err(format!("File size {} exceeds maximum {}", size, MAX_FILE_SIZE));
    }
    Ok(())
}

pub fn validate_file_type(filename: &str) -> Result<(), String> {
    let allowed_extensions = vec![
        "exe", "dll", "pdf", "doc", "docx", "zip", "rar", "apk", "elf", "so",
    ];

    let extension = filename
        .rsplit('.')
        .next()
        .ok_or("No file extension found")?
        .to_lowercase();

    if allowed_extensions.contains(&extension.as_str()) {
        Ok(())
    } else {
        Err(format!("File type '{}' is not allowed", extension))
    }
}

pub fn validate_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err("Invalid URL format".to_string())
    }
}
