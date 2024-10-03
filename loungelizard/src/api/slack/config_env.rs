pub fn config_env_var(name: &str) -> Result<String, String> {
    // Returns a `String` containing the value of the environment variable with the given `name`.
    // If the environment variable does not exist, returns an error.
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}