use intent_database::ai_manager::AiManager;
use intent_database::ai_sim::SimulatedModelAdapter;
use intent_database::database::AsyncIntentDatabase;
use intent_database::storage::SledStorage;
use intent_database::web_helpers::PendingReview;
use intent_database::web_routes;
use intent_database::web_routes::WebContext;
use intent_database::TemplateModelAdapter;
#[cfg(feature = "gemini-model-adapter")]
use intent_database::GeminiModelAdapter;
use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::sync::Mutex as StdMutex;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server};

fn main() {
    let ai = Arc::new(AiManager::new());
    ai.register("template", Arc::new(TemplateModelAdapter::new()));
    ai.register("sim", Arc::new(SimulatedModelAdapter::new(42, "sim")));

    let gemini_key: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // Try to auto-load a gitignored secrets/gemini_key file if present
    if let Ok(mut f) = std::fs::File::open("secrets/gemini_key") {
        use std::io::Read;
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            let key = s.lines().next().unwrap_or("").trim().to_string();
            if !key.is_empty() {
                let mut g = gemini_key.lock().unwrap();
                *g = Some(key.clone());
                #[cfg(feature = "gemini-model-adapter")]
                {
                    let endpoint = env::var("GEMINI_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:8081/gemini".to_string());
                    let boxed = GeminiModelAdapter(endpoint, Some(key));
                    ai.register("gemini", boxed);
                    println!("Registered Gemini adapter from secrets/gemini_key");
                }
            }
        }
    }

    let port = env::var("WEB_SERVER_PORT").ok().and_then(|s| s.parse::<u16>().ok()).unwrap_or(3000);
    let bind = format!("0.0.0.0:{}", port);
    // prepare storage and async DB
    let db_path = PathBuf::from("data/intent-db");
    std::fs::create_dir_all(&db_path).ok();
    let storage = SledStorage::new(db_path.clone());
    let db_inner = intent_database::database::IntentDatabase::new();
    let intent_db = Arc::new(AsyncIntentDatabase::new(db_inner, Arc::new(storage)));
    // prepare logging file (append-only json-lines)
    let logs_dir = PathBuf::from("logs");
    std::fs::create_dir_all(&logs_dir).ok();
    let log_path = logs_dir.join("web_server.log");
    // Try to open the log file; if that fails, fallback to a no-op sink.
    // This prevents panics in environments where logs can't be created.
    let lf_handle =
        OpenOptions::new().create(true).append(true).open(&log_path).unwrap_or_else(|_| {
            // Fallback to a write sink: on Unix use /dev/null. On Windows attempt
            // to create a temporary in-memory sink.
            #[cfg(unix)]
            {
                std::fs::OpenOptions::new().write(true).open("/dev/null").expect("open /dev/null")
            }
            #[cfg(windows)]
            {
                // Try to create the file in the logs dir with a different name.
                // If that fails, create a tempfile.
                {
                    let b = tempfile::Builder::new();
                    let b = b.prefix("web_server_log_fallback");
                    let tf = b.tempfile().expect("tmpfile");
                    tf.into_file()
                }
            }
        });
    let log_file = Arc::new(StdMutex::new(lf_handle));
    let log_counter = Arc::new(AtomicU64::new(1));

    // Try to bind to the requested port; on failure (e.g., Address in use),
    // attempt to bind to an ephemeral port (0) so tests and environments can
    // still run the server without panicking.
    let server = match Server::http(&bind) {
        Ok(s) => {
            println!("Listening on http://0.0.0.0:{}", port);
            s
        }
        Err(e) => {
            eprintln!("Failed to bind to {}: {}. Falling back to an ephemeral port.", bind, e,);
            let fallback_bind = "0.0.0.0:0".to_string();
            let s = Server::http(&fallback_bind).expect("start fallback server");
            // Try to retrieve the actual port the server is listening on.
            // tiny_http exposes this via addr().
            let addr = s.server_addr();
            println!("Listening on http://{}", addr);
            s
        }
    };

    // Create a small runtime to run async adapter calls from this sync loop
    let rt =
        Arc::new(tokio::runtime::Builder::new_current_thread().enable_all().build().expect("rt"));

    // SSE broadcaster list: when graph updates occur we send JSON strings to
    // all connected SSE clients via mpsc channels.
    let broadcasters: Arc<Mutex<Vec<Sender<String>>>> = Arc::new(Mutex::new(Vec::new()));
    // Simple in-memory review queue for human-in-the-loop training approvals
    let review_queue: Arc<Mutex<Vec<PendingReview>>> = Arc::new(Mutex::new(Vec::new()));

    // Build a shared web context passed to handlers that need many dependencies
    // decide default adapter: use gemini if registered, otherwise template
    let default_adapter = if ai.get("gemini").is_some() {
        "gemini".to_string()
    } else {
        "template".to_string()
    };

    let web_ctx = Arc::new(WebContext {
        ai: ai.clone(),
        intent_db: intent_db.clone(),
        broadcasters: broadcasters.clone(),
        review_queue: review_queue.clone(),
        log_file: log_file.clone(),
        log_path: log_path.clone(),
        log_counter: log_counter.clone(),
        rt: rt.clone(),
        default_adapter: default_adapter.clone(),
    });

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        match (url.as_str(), request.method()) {
            ("/", _) => {
                let html = include_str!("../../static/ui.html");
                let resp = Response::from_string(html).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
                );
                let _ = request.respond(resp);
            }

            ("/set_gemini", &Method::Post) => {
                web_routes::handle_set_gemini(request, gemini_key.clone())
            }

            ("/chat", &Method::Post) => web_routes::handle_chat(request, web_ctx.clone()),

            ("/events", &Method::Get) => web_routes::handle_events(request, broadcasters.clone()),

            ("/graph", &Method::Get) => web_routes::handle_graph(request, intent_db.clone(), &rt),

            ("/review/pending", &Method::Get) => {
                web_routes::handle_review_pending(request, review_queue.clone())
            }
            ("/review/approve", &Method::Post) => web_routes::handle_review_approve(
                request,
                review_queue.clone(),
                intent_db.clone(),
                &rt,
            ),
            ("/review/reject", &Method::Post) => {
                web_routes::handle_review_reject(request, review_queue.clone())
            }
            ("/review/evaluate", &Method::Post) => web_routes::handle_review_evaluate(request),

            ("/logs", &Method::Get) => {
                // return last 200 lines
                let content = std::fs::read_to_string(&log_path).unwrap_or_default();
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
                let out = lines[start..].join("\n");
                let _ = request.respond(Response::from_string(out).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
                ));
            }

            _ => {
                let _ = request.respond(Response::from_string("not found").with_status_code(404));
            }
        }
    }
}
