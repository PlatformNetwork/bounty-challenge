// ... existing code ...

/// Opens a terminal window at the specified project location.
/// 
/// This function ensures that the terminal's working directory is set to the
/// project root directory upon opening, addressing the issue where the terminal
/// would open in an unspecified or incorrect location.
pub fn open_terminal_at_project(project_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Validate that the project path exists and is a directory
    if !project_path.exists() || !project_path.is_dir() {
        return Err(format!("Project path does not exist or is not a directory: {:?}", project_path).into());
    }

    // Determine the terminal command based on the operating system
    #[cfg(target_os = "windows")]
    let command = "cmd";
    
    #[cfg(target_os = "macos")]
    let command = "open";
    
    #[cfg(target_os = "linux")]
    let command = "xterm"; // or gnome-terminal, konsole, etc.

    // Spawn the terminal process with the working directory set to project_path
    std::process::Command::new(command)
        .current_dir(project_path) // This is the key fix: set the working directory
        .spawn()?;

    println!("Terminal opened at project location: {:?}", project_path);
    Ok(())
}

// ... existing code ...
