// LLM client module - handles communication with LLM APIs

pub mod keyring_manager;

use crate::config::settings::{LlmModel, LlmServiceType};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LM Studio models response
#[derive(Debug, Deserialize)]
struct LMStudioModelsResponse {
    data: Vec<LMStudioModel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LMStudioModel {
    pub id: String,
    #[serde(rename = "type")]
    pub model_type: String,
    pub state: String, // "loaded" or "not-loaded"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_length: Option<u32>,
}

/// Ollama models response
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaModelDetails {
    pub format: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
}

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
/// Routes the request to the appropriate API based on the service type
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
    println!("Service Type: {:?}", model.service_type);
    println!("API URL: {}", model.api_url);
    println!("System prompt length: {}", system_prompt.len());
    println!("User text length: {}", user_text.len());

    // Route to appropriate API based on service type
    match model.service_type {
        LlmServiceType::Gemini => {
            println!("Using Gemini API");
            call_gemini(model, api_key, system_prompt, user_text).await
        }
        LlmServiceType::LMStudio => {
            println!("Using LM Studio API");
            call_lm_studio(model, api_key, system_prompt, user_text).await
        }
        LlmServiceType::Ollama => {
            println!("Using Ollama API");
            call_ollama(model, api_key, system_prompt, user_text).await
        }
        LlmServiceType::OpenRouter | LlmServiceType::OpenAI => {
            println!("Using OpenAI-compatible API");
            call_openrouter(model, api_key, system_prompt, user_text).await
        }
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

/// Get available models from LM Studio
pub async fn get_lm_studio_models(port: Option<u16>) -> Result<Vec<LMStudioModel>, String> {
    let port = port.unwrap_or(1234);
    let url = format!("http://localhost:{}/api/v0/models", port);

    println!("Fetching LM Studio models from: {}", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                "LM Studio is not running. Please start LM Studio.".to_string()
            } else {
                format!("Failed to connect to LM Studio: {}", e)
            }
        })?;

    let models: LMStudioModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LM Studio response: {}", e))?;

    println!("Found {} LM Studio models", models.data.len());
    Ok(models.data)
}

/// Get available models from Ollama
pub async fn get_ollama_models(port: Option<u16>) -> Result<Vec<OllamaModel>, String> {
    let port = port.unwrap_or(11434);
    let url = format!("http://localhost:{}/api/tags", port);

    println!("Fetching Ollama models from: {}", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                "Ollama is not running. Please start Ollama service.".to_string()
            } else {
                format!("Failed to connect to Ollama: {}", e)
            }
        })?;

    let tags: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    println!("Found {} Ollama models", tags.models.len());
    Ok(tags.models)
}

/// Call LM Studio API
async fn call_lm_studio(
    model: &LlmModel,
    _api_key: &str, // Not used for LM Studio
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    println!("=== LM STUDIO CALL ===");

    // Build messages array
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

    let request_body = OpenAIRequest {
        model: model.model_name.clone(),
        messages,
    };

    println!("Making LM Studio HTTP POST request to {}...", model.api_url);

    // Use longer timeout for local inference (5 minutes)
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post(&model.api_url)
        .header("Content-Type", "application/json")
        // No Authorization header needed for LM Studio
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "LM Studio request timed out. The model may be too slow or the prompt too complex.".to_string()
            } else if e.is_connect() {
                "LM Studio is not running. Please start LM Studio and load a model.".to_string()
            } else {
                format!("LM Studio request failed: {}", e)
            }
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LM Studio error {}: {}", status, error_text));
    }

    println!("HTTP request successful, parsing LM Studio response...");

    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LM Studio response: {}", e))?;

    if let Some(choice) = openai_response.choices.first() {
        println!("LM Studio response received ({} chars)", choice.message.content.len());
        return Ok(choice.message.content.clone());
    }

    Err("LM Studio response did not contain expected text".to_string())
}

/// Call Ollama API
async fn call_ollama(
    model: &LlmModel,
    _api_key: &str, // Not used for Ollama
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    println!("=== OLLAMA CALL ===");

    // Build messages array
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

    let request_body = OpenAIRequest {
        model: model.model_name.clone(),
        messages,
    };

    // Use OpenAI-compatible endpoint
    let url = if model.api_url.contains("/v1/chat/completions") {
        model.api_url.clone()
    } else {
        // Assume base URL, construct full endpoint
        format!("{}/v1/chat/completions", model.api_url.trim_end_matches('/'))
    };

    println!("Making Ollama HTTP POST request to {}...", url);

    // Use longer timeout for local inference (5 minutes)
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Ollama request timed out. The model may be too slow or the prompt too complex.".to_string()
            } else if e.is_connect() {
                "Ollama is not running. Please start Ollama service.".to_string()
            } else {
                format!("Ollama request failed: {}", e)
            }
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

        // Provide helpful error for model not found
        if status == 404 {
            return Err(format!(
                "Model '{}' not found. Pull it first with: ollama pull {}",
                model.model_name, model.model_name
            ));
        }

        return Err(format!("Ollama error {}: {}", status, error_text));
    }

    println!("HTTP request successful, parsing Ollama response...");

    let openai_response: OpenAIResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    if let Some(choice) = openai_response.choices.first() {
        println!("Ollama response received ({} chars)", choice.message.content.len());
        return Ok(choice.message.content.clone());
    }

    Err("Ollama response did not contain expected text".to_string())
}
