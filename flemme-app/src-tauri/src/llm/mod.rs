// LLM client module - handles communication with LLM APIs

pub mod keyring_manager;

use crate::config::settings::LlmModel;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Gemini API request structure
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

/// Gemini API response structure
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: String,
}

/// OpenAI/OpenRouter API request structure
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI/OpenRouter API response structure
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

/// Call an LLM with the given system prompt and user text
///
/// Automatically detects the API type (Gemini or OpenAI/OpenRouter) based on the URL
///
/// # Arguments
/// * `model` - The LLM model configuration
/// * `api_key` - The API key for authentication
/// * `system_prompt` - The system/initial prompt
/// * `user_text` - The transcribed text to process
///
/// # Returns
/// The LLM's response text, or an error message
pub async fn call_llm(
    model: &LlmModel,
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    println!("=== LLM CALL ===");
    println!("Model: {}", model.name);
    println!("API URL: {}", model.api_url);
    println!("System prompt length: {}", system_prompt.len());
    println!("User text length: {}", user_text.len());

    // Detect API type based on URL
    let is_openrouter = model.api_url.contains("openrouter.ai");
    let is_gemini = model.api_url.contains("generativelanguage.googleapis.com");

    if is_openrouter {
        println!("Detected OpenRouter API");
        call_openrouter(model, api_key, system_prompt, user_text).await
    } else if is_gemini {
        println!("Detected Gemini API");
        call_gemini(model, api_key, system_prompt, user_text).await
    } else {
        // Default to OpenAI-compatible format
        println!("Using OpenAI-compatible format");
        call_openrouter(model, api_key, system_prompt, user_text).await
    }
}

/// Call Gemini API
async fn call_gemini(
    model: &LlmModel,
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    // Combine system prompt and user text
    let combined_text = if system_prompt.is_empty() {
        user_text.to_string()
    } else {
        format!("{}\n\n{}", system_prompt, user_text)
    };

    // Build the request for Gemini API
    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: combined_text,
            }],
        }],
    };

    // Build the full URL with API key
    let full_url = if model.api_url.contains('?') {
        format!("{}&key={}", model.api_url, api_key)
    } else {
        format!("{}?key={}", model.api_url, api_key)
    };

    println!("Making Gemini HTTP POST request...");

    // Make the HTTP request with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post(&full_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "LLM request timed out after 30 seconds".to_string()
            } else if e.is_connect() {
                format!("Failed to connect to LLM server: {}", e)
            } else {
                format!("LLM HTTP request failed: {}", e)
            }
        })?;

    // Check HTTP status
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API returned error {}: {}", status, error_text));
    }

    println!("HTTP request successful, parsing Gemini response...");

    // Parse the response
    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    // Extract the text from the first candidate
    if let Some(candidate) = gemini_response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            println!("LLM response received ({} chars)", part.text.len());
            return Ok(part.text.clone());
        }
    }

    Err("LLM response did not contain expected text".to_string())
}

/// Call OpenRouter/OpenAI-compatible API
async fn call_openrouter(
    model: &LlmModel,
    api_key: &str,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    // Build messages array with system and user messages
    let mut messages = Vec::new();

    if !system_prompt.is_empty() {
        messages.push(OpenAIMessage {
            role: String::from("system"),
            content: system_prompt.to_string(),
        });
    }

    messages.push(OpenAIMessage {
        role: String::from("user"),
        content: user_text.to_string(),
    });

    // Build the request for OpenAI/OpenRouter API
    let request_body = OpenAIRequest {
        model: model.model_name.clone(),
        messages,
    };

    println!("Making OpenRouter HTTP POST request...");

    // Make the HTTP request with timeout and Authorization header
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post(&model.api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "LLM request timed out after 30 seconds".to_string()
            } else if e.is_connect() {
                format!("Failed to connect to LLM server: {}", e)
            } else {
                format!("LLM HTTP request failed: {}", e)
            }
        })?;

    // Check HTTP status
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API returned error {}: {}", status, error_text));
    }

    println!("HTTP request successful, parsing OpenRouter response...");

    // Parse the response
    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    // Extract the text from the first choice
    if let Some(choice) = openai_response.choices.first() {
        println!("LLM response received ({} chars)", choice.message.content.len());
        return Ok(choice.message.content.clone());
    }

    Err("LLM response did not contain expected text".to_string())
}
