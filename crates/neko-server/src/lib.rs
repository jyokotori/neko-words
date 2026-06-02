use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use neko_core::{
    config::{config_path, AppConfig, Mode},
    llm::OpenAiCompatibleEnricher,
    models::{AddWordResult, DueReview, Grade, Review},
    repository::{SqlxRepository, WordRepository},
    service,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    repo: SqlxRepository,
    llm: OpenAiCompatibleEnricher,
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
    #[serde(default = "default_language")]
    language: String,
}

#[derive(Deserialize)]
struct DueQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_language")]
    language: String,
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
    let server = cfg.server.context("missing [server] config")?;
    let llm = cfg.llm.context("missing [llm] config")?;
    if llm.api_key.is_empty() || llm.base_url.is_empty() || llm.model.is_empty() {
        anyhow::bail!("server requires llm.api_key, llm.base_url, and llm.model");
    }
    let repo = SqlxRepository::connect(&server.database_url).await?;
    repo.migrate().await?;

    let state = AppState {
        repo,
        llm: OpenAiCompatibleEnricher::new(llm),
    };
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&server.bind)
        .await
        .with_context(|| format!("failed to bind {}", server.bind))?;
    let addr: SocketAddr = listener.local_addr()?;
    println!("Neko Words API listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .nest(
            "/api/v1",
            Router::new()
                .route("/words/", post(add_word))
                .route("/reviews/due", get(due_reviews))
                .route("/reviews/{word_id}/log", post(log_review))
                .route("/reviews/{word_id}/undo", post(undo_review)),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Welcome to Neko Words API",
        "api": "/api/v1"
    }))
}

async fn add_word(
    State(state): State<AppState>,
    Json(input): Json<WordInput>,
) -> Result<Json<AddWordResult>, ApiError> {
    Ok(Json(
        service::add_word(&state.repo, &state.llm, &input.word, &input.language).await?,
    ))
}

async fn due_reviews(
    State(state): State<AppState>,
    Query(query): Query<DueQuery>,
) -> Result<Json<Vec<DueReview>>, ApiError> {
    Ok(Json(
        state.repo.due_reviews(&query.language, query.limit).await?,
    ))
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

fn default_language() -> String {
    "en".to_string()
}

fn default_limit() -> i64 {
    50
}
