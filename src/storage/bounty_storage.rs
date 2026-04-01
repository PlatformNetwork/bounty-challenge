fn write_registration(github_username: &str, hotkey_ss58: &str) -> bool {
    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let registration = UserRegistration {
        hotkey: String::from(hotkey_ss58),
        github_username: String::from(github_username),
        registered_epoch: current_epoch,
    };

    let data = match bincode::serialize(&registration) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let user_key = make_key(b"user:", hotkey_ss58);
    let user_dir = std::path::Path::new(&user_key);
    if !user_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(user_dir.parent().unwrap()) {
            eprintln!("Error creating directory: {}", e);
            return false;
        }
    }

    if host_storage_set(&user_key, &data).is_err() {
        return false;
    }

    let github_key = make_key(b"github:", &github_username.to_lowercase());
    if host_storage_set(&github_key, hotkey_ss58.as_bytes()).is_err() {
        return false;
    }

    true
}