use std::path::Path;
use std::process::Command;

pub fn open_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    
    Ok(())
}

pub fn open_folder(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let folder_path = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(folder_path)
            .spawn()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(folder_path)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(folder_path)
            .spawn()?;
    }
    
    Ok(())
}