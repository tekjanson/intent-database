#![cfg(feature = "gemini-model-adapter")]

use intent_database::ai::TemplateModelAdapter;
use intent_database::ai_manager::AiManager;
use intent_database::GeminiModelAdapter;
use std::sync::Arc;

#[test]
fn test_default_is_gemini_when_registered() {
    let ai = AiManager::new();
    // register a dummy adapter under the name "gemini"
    ai.register("gemini", Arc::new(TemplateModelAdapter::new()));
    // server logic uses ai.get("gemini").is_some() to choose default
    assert!(ai.get("gemini").is_some(), "gemini should be registered");
}

#[tokio::test]
async fn test_gemini_adapter_against_local_endpoint() {
    // Start a tiny HTTP responder on an ephemeral port that returns a
    // Google Generative Language shaped JSON response.
    use std::thread;
    use tiny_http::{Response, Server};

    let server = Server::http("127.0.0.1:0").expect("start tiny server");
    let addr = server.server_addr().to_string();
    let url = format!("http://{}/v1beta/models/gemini-2.0-flash:generateContent", addr);

    // spawn server thread that serves one or more requests
    thread::spawn(move || {
        for request in server.incoming_requests() {
            // always respond with a structured candidates/content/text JSON
            let body = r#"{"candidates":[{"content":[{"text":"Hello from Gemini"}]}]}"#;
            let resp = Response::from_string(body).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .unwrap(),
            );
            let _ = request.respond(resp);
        }
    });

    // Create an adapter pointing at our local server
    let boxed = GeminiModelAdapter(url, Some("test-key".to_string()));
    let ai = AiManager::new();
    ai.register("gemini", boxed.clone());

    // Call generate via the manager (async)
    let out = ai.generate_with("gemini", "hello").await.expect("generate ok");
    assert!(out.contains("Hello from Gemini"));
}
