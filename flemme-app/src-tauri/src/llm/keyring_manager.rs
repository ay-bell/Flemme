// Secure storage for LLM API keys using OS keyring

use keyring::Entry;

const SERVICE_NAME: &str = "FlemmeApp";

/// Store an API key in the OS keyring
///
/// # Arguments
/// * `llm_id` - The unique identifier of the LLM model
/// * `api_key` - The API key to store
///
/// # Returns
/// Ok(()) if successful, Err with error message otherwise
pub fn store_api_key(llm_id: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, llm_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(api_key)
        .map_err(|e| format!("Failed to store API key in keyring: {}", e))?;

    println!("API key stored securely for LLM: {}", llm_id);
    Ok(())
}

/// Retrieve an API key from the OS keyring
///
/// # Arguments
/// * `llm_id` - The unique identifier of the LLM model
///
/// # Returns
/// The API key if found, None if not found, or Err with error message
pub fn get_api_key(llm_id: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, llm_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => {
            println!("API key retrieved from keyring for LLM: {}", llm_id);
            Ok(Some(password))
        }
        Err(keyring::Error::NoEntry) => {
            println!("No API key found in keyring for LLM: {}", llm_id);
            Ok(None)
        }
        Err(e) => Err(format!("Failed to retrieve API key from keyring: {}", e)),
    }
}

/// Delete an API key from the OS keyring
///
/// # Arguments
/// * `llm_id` - The unique identifier of the LLM model
///
/// # Returns
/// Ok(()) if successful or if key didn't exist, Err with error message on failure
pub fn delete_api_key(llm_id: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, llm_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_password() {
        Ok(()) => {
            println!("API key deleted from keyring for LLM: {}", llm_id);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            println!("No API key to delete for LLM: {}", llm_id);
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete API key from keyring: {}", e)),
    }
}
