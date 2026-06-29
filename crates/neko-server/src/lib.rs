use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use neko_core::{
    config::{AppConfig, Mode, config_path},
    llm::OpenAiCompatibleEnricher,
    models::{AddWordResult, DueReview, ExportData, Grade, Review},
    repository::{SqliteRepository, WordRepository},
    service,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    repo: SqliteRepository,
    llm: OpenAiCompatibleEnricher,
    auth_token: Option<String>,
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = self.0.to_string();
        let status = if message.contains("not found") {
            StatusCode::NOT_FOUND
        } else if message.contains("no review history") || message.contains("unknown review grade")
        {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(serde_json::json!({ "detail": message }))).into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

#[derive(Deserialize)]
struct WordInput {
    word: String,
    #[serde(default = "default_tag")]
    tag: String,
}

#[derive(Deserialize)]
struct DueQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_tag")]
    tag: String,
}

#[derive(Deserialize)]
struct ReviewLog {
    grade: Grade,
}

pub async fn run_from_config() -> Result<()> {
    let path = config_path()?;
    let cfg = AppConfig::load(&path)
        .with_context(|| format!("server requires config file at {}", path.display()))?;
    run(cfg).await
}

pub async fn run(cfg: AppConfig) -> Result<()> {
    if !matches!(cfg.mode, Some(Mode::Server)) {
        anyhow::bail!("server command requires mode = \"server\" in config.toml");
    }
    let db_url = cfg.server_db_url()?;
    let server = cfg.server.context("missing [server] config")?;
    let llm = cfg.llm.context("missing [llm] config")?;
    if llm.api_key.is_empty()
        || llm.base_url.is_empty()
        || llm.model.is_empty()
        || llm.target_language.is_empty()
    {
        anyhow::bail!(
            "server requires llm.api_key, llm.base_url, llm.model, and llm.target_language"
        );
    }
    let repo = SqliteRepository::connect(&db_url).await?;
    repo.migrate().await?;

    let auth_token = server.auth_token.filter(|t| !t.is_empty());
    if auth_token.is_none() {
        eprintln!(
            "warning: [server].auth_token is not set; the API (including /export and /import) is unauthenticated"
        );
    }
    let state = AppState {
        repo,
        llm: OpenAiCompatibleEnricher::new(llm),
        auth_token,
    };
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&server.bind)
        .await
        .with_context(|| format!("failed to bind {}", server.bind))?;
    let addr: SocketAddr = listener.local_addr()?;
    println!("Neko Words listening on http://{addr}");
    println!("Web UI: http://{addr}");
    println!("API: http://{addr}/api/v1");
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/words/", post(add_word))
        .route("/reviews/due", get(due_reviews))
        .route("/reviews/{word_id}/log", post(log_review))
        .route("/reviews/{word_id}/undo", post(undo_review))
        .route("/export", get(export))
        .route("/import", post(import))
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state);
    Router::new()
        .route("/", get(root))
        .nest("/api/v1", api)
        .layer(CorsLayer::permissive())
}

async fn auth(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(expected) = state.auth_token.as_deref() else {
        return next.run(request).await;
    };
    let provided = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if provided == Some(expected) {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "detail": "missing or invalid authorization token" })),
        )
            .into_response()
    }
}

async fn root() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

async fn add_word(
    State(state): State<AppState>,
    Json(input): Json<WordInput>,
) -> Result<Json<AddWordResult>, ApiError> {
    Ok(Json(
        service::add_word(&state.repo, &state.llm, &input.word, &input.tag).await?,
    ))
}

async fn due_reviews(
    State(state): State<AppState>,
    Query(query): Query<DueQuery>,
) -> Result<Json<Vec<DueReview>>, ApiError> {
    Ok(Json(state.repo.due_reviews(&query.tag, query.limit).await?))
}

async fn log_review(
    State(state): State<AppState>,
    Path(word_id): Path<String>,
    Json(log): Json<ReviewLog>,
) -> Result<Json<Review>, ApiError> {
    Ok(Json(state.repo.log_review(&word_id, log.grade).await?))
}

async fn undo_review(
    State(state): State<AppState>,
    Path(word_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let grade = state.repo.undo_review(&word_id).await?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "undone_grade": grade
    })))
}

async fn export(State(state): State<AppState>) -> Result<Json<ExportData>, ApiError> {
    Ok(Json(state.repo.export_all().await?))
}

async fn import(
    State(state): State<AppState>,
    Json(data): Json<ExportData>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.repo.import_all(&data).await?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "words": data.words.len(),
        "reviews": data.reviews.len(),
    })))
}

fn default_tag() -> String {
    "default".to_string()
}

fn default_limit() -> i64 {
    50
}
