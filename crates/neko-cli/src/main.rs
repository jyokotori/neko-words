use std::{
    io::{self, IsTerminal, Write},
    net::SocketAddr,
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use console::{Key, Term};
use neko_core::{
    config::{AppConfig, ClientServerConfig, LlmConfig, Mode, ServerConfig, config_path},
    llm::OpenAiCompatibleEnricher,
    models::{AddWordResult, DueReview, ExportData, Grade},
    repository::{SqliteRepository, WordRepository},
    service,
};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "neko-words", version, about = "Neko Words vocabulary CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    Review(ReviewArgs),
    Mode {
        mode: ModeArg,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Server(ServerArgs),
    Export(ExportArgs),
    Import(ImportArgs),
}

#[derive(Args)]
struct ExportArgs {
    /// Write JSON to this file instead of stdout
    #[arg(long)]
    out: Option<String>,
}

#[derive(Args)]
struct ImportArgs {
    /// Path to a JSON file produced by `export`
    file: String,
}

#[derive(Args)]
struct AddArgs {
    word: Option<String>,
    #[arg(long)]
    tag: Option<String>,
    #[arg(long)]
    batch: bool,
}

#[derive(Args)]
struct ReviewArgs {
    #[arg(long)]
    tag: Option<String>,
    #[arg(long, default_value_t = 50)]
    limit: i64,
    /// Use line input instead of immediate single-key grading
    #[arg(long)]
    line: bool,
}

#[derive(Args)]
struct ServerArgs {
    /// Bind address for this server process, for example 0.0.0.0:8002
    #[arg(long)]
    bind: Option<String>,
}

#[derive(Clone, ValueEnum)]
enum ModeArg {
    Local,
    Server,
}

#[derive(Subcommand)]
enum ConfigCommand {
    Get { key: Option<String> },
    Set { key: String, value: String },
    Path,
    Init(InitConfigArgs),
}

#[derive(Args)]
struct InitConfigArgs {
    /// Initialize local mode with this OpenAI API key without prompting
    #[arg(long)]
    api_key: Option<String>,
    /// OpenAI-compatible API base URL
    #[arg(long)]
    base_url: Option<String>,
    /// Model name
    #[arg(long)]
    model: Option<String>,
    /// Language for generated translations and example translations
    #[arg(long)]
    target_language: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Add(args)) => add(args).await,
        Some(Commands::Review(args)) => review(args).await,
        Some(Commands::Mode { mode }) => set_mode(mode),
        Some(Commands::Config { command }) => config_command(command),
        Some(Commands::Server(args)) => server(args).await,
        Some(Commands::Export(args)) => export(args).await,
        Some(Commands::Import(args)) => import(args).await,
        None => first_run(),
    }
}

fn first_run() -> Result<()> {
    let _cfg = ensure_config(false)?;
    println!();
    println!("Neko Words is ready.");
    println!("Config: {}", config_path()?.display());
    println!();
    println!("Next:");
    println!("  neko-words add hello --tag default");
    println!("  neko-words review --tag default");
    Ok(())
}

async fn server(args: ServerArgs) -> Result<()> {
    let mut cfg = ensure_config(true)?;
    if let Some(bind) = args.bind {
        cfg.server.as_mut().context("missing [server] config")?.bind = bind;
    }
    ensure_server_auth_token(&mut cfg)?;
    cfg.mode = Some(Mode::Server);
    neko_server::run(cfg).await
}

fn ensure_server_auth_token(cfg: &mut AppConfig) -> Result<()> {
    let server = cfg.server.as_ref().context("missing [server] config")?;
    if !bind_exposes_network(&server.bind) {
        return Ok(());
    }

    if let Some(token) = server
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        println!("Web UI auth token: {token}");
        return Ok(());
    }

    println!(
        "Server bind {} is reachable from other devices. Set an auth token before exposing it.",
        server.bind
    );
    let token = prompt("server auth token (press Enter to generate)", None)?;
    let token = if token.trim().is_empty() {
        generate_auth_token()
    } else {
        token
    };
    cfg.server
        .as_mut()
        .context("missing [server] config")?
        .auth_token = Some(token.clone());

    let path = config_path()?;
    let mut saved_cfg = load_or_default()?;
    let saved_server = saved_cfg
        .server
        .as_mut()
        .context("missing [server] config in saved config")?;
    saved_server.auth_token = Some(token.clone());
    saved_cfg.save(&path)?;

    println!("Auth token saved to {}", path.display());
    println!("Web UI auth token: {token}");
    Ok(())
}

fn bind_exposes_network(bind: &str) -> bool {
    if let Ok(addr) = bind.parse::<SocketAddr>() {
        return !addr.ip().is_loopback();
    }

    let host = bind_host(bind);
    !matches!(host.as_str(), "127.0.0.1" | "::1" | "localhost")
}

fn bind_host(bind: &str) -> String {
    let bind = bind.trim();
    if let Some(rest) = bind.strip_prefix('[') {
        return rest
            .split_once(']')
            .map_or(rest, |(host, _)| host)
            .trim()
            .to_string();
    }
    bind.rsplit_once(':')
        .map_or(bind, |(host, _)| host)
        .trim()
        .to_string()
}

fn generate_auth_token() -> String {
    Uuid::new_v4().simple().to_string()[..16].to_string()
}

async fn add(args: AddArgs) -> Result<()> {
    let cfg = ensure_config(false)?;
    let tag = selected_tag(args.tag.as_deref(), &cfg);
    match cfg.mode.clone().context("missing mode")? {
        Mode::Local => {
            let repo = local_repo(&cfg).await?;
            let llm = OpenAiCompatibleEnricher::new(cfg.llm.context("missing [llm] config")?);
            if args.batch || args.word.is_none() {
                loop {
                    let word = prompt("Word to add (press Enter to finish)", None)?;
                    if word.trim().is_empty() {
                        break;
                    }
                    print_add_result(service::add_word(&repo, &llm, &word, &tag).await?);
                }
            } else if let Some(word) = args.word {
                print_add_result(service::add_word(&repo, &llm, &word, &tag).await?);
            }
        }
        Mode::Server => {
            let client_server = cfg
                .client_server
                .context("missing [client_server] config")?;
            let api = client_server.api_base_url;
            let token = client_server.auth_token;
            if args.batch || args.word.is_none() {
                loop {
                    let word = prompt("Word to add (press Enter to finish)", None)?;
                    if word.trim().is_empty() {
                        break;
                    }
                    print_add_result(add_word_http(&api, token.as_deref(), &word, &tag).await?);
                }
            } else if let Some(word) = args.word {
                print_add_result(add_word_http(&api, token.as_deref(), &word, &tag).await?);
            }
        }
    }
    Ok(())
}

async fn review(args: ReviewArgs) -> Result<()> {
    let cfg = ensure_config(false)?;
    let tag = selected_tag(args.tag.as_deref(), &cfg);
    let single_key = !args.line && io::stdin().is_terminal() && io::stdout().is_terminal();
    let due = match cfg.mode.clone().context("missing mode")? {
        Mode::Local => {
            let repo = local_repo(&cfg).await?;
            review_local(&repo, &tag, args.limit, single_key).await?
        }
        Mode::Server => {
            let client_server = cfg
                .client_server
                .context("missing [client_server] config")?;
            review_http(
                &client_server.api_base_url,
                client_server.auth_token.as_deref(),
                &tag,
                args.limit,
                single_key,
            )
            .await?
        }
    };

    if due.is_empty() {
        println!("No words are due for review.");
    }
    Ok(())
}

fn selected_tag(explicit: Option<&str>, cfg: &AppConfig) -> String {
    explicit
        .or_else(|| {
            cfg.cli
                .as_ref()
                .map(|cli| cli.default_tag.as_str())
                .filter(|tag| !tag.trim().is_empty())
        })
        .unwrap_or("default")
        .to_string()
}

async fn review_local(
    repo: &SqliteRepository,
    tag: &str,
    limit: i64,
    single_key: bool,
) -> Result<Vec<DueReview>> {
    let due = repo.due_reviews(tag, limit).await?;
    for item in &due {
        print_due(item);
        let Some(grade) = prompt_grade(single_key)? else {
            println!("Review stopped.");
            break;
        };
        service::log_review(repo, &item.word.id, grade).await?;
    }
    Ok(due)
}

async fn review_http(
    api: &str,
    token: Option<&str>,
    tag: &str,
    limit: i64,
    single_key: bool,
) -> Result<Vec<DueReview>> {
    let client = reqwest::Client::new();
    let due: Vec<DueReview> = bearer(
        client
            .get(format!("{}/reviews/due", api.trim_end_matches('/')))
            .query(&[("tag", tag), ("limit", &limit.to_string())]),
        token,
    )
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
    for item in &due {
        print_due(item);
        let Some(grade) = prompt_grade(single_key)? else {
            println!("Review stopped.");
            break;
        };
        bearer(
            client.post(format!(
                "{}/reviews/{}/log",
                api.trim_end_matches('/'),
                item.word.id
            )),
            token,
        )
        .json(&serde_json::json!({ "grade": grade }))
        .send()
        .await?
        .error_for_status()?;
    }
    Ok(due)
}

async fn local_repo(cfg: &AppConfig) -> Result<SqliteRepository> {
    let repo = SqliteRepository::connect(&cfg.local_db_url()?).await?;
    repo.migrate().await?;
    Ok(repo)
}

async fn export(args: ExportArgs) -> Result<()> {
    let cfg = ensure_config(false)?;
    let data = match cfg.mode.clone().context("missing mode")? {
        Mode::Local => local_repo(&cfg).await?.export_all().await?,
        Mode::Server => {
            let client_server = cfg
                .client_server
                .context("missing [client_server] config")?;
            export_http(
                &client_server.api_base_url,
                client_server.auth_token.as_deref(),
            )
            .await?
        }
    };
    let json = serde_json::to_string_pretty(&data)?;
    match args.out {
        Some(path) => {
            std::fs::write(&path, json).with_context(|| format!("failed to write {path}"))?;
            println!(
                "exported {} words, {} reviews -> {path}",
                data.words.len(),
                data.reviews.len()
            );
        }
        None => println!("{json}"),
    }
    Ok(())
}

async fn import(args: ImportArgs) -> Result<()> {
    let cfg = ensure_config(false)?;
    let text = std::fs::read_to_string(&args.file)
        .with_context(|| format!("failed to read {}", args.file))?;
    let data: ExportData = serde_json::from_str(&text).context("failed to parse export JSON")?;
    match cfg.mode.clone().context("missing mode")? {
        Mode::Local => local_repo(&cfg).await?.import_all(&data).await?,
        Mode::Server => {
            let client_server = cfg
                .client_server
                .context("missing [client_server] config")?;
            import_http(
                &client_server.api_base_url,
                client_server.auth_token.as_deref(),
                &data,
            )
            .await?;
        }
    }
    println!(
        "imported {} words, {} reviews",
        data.words.len(),
        data.reviews.len()
    );
    Ok(())
}

async fn export_http(api: &str, token: Option<&str>) -> Result<ExportData> {
    let data = bearer(
        reqwest::Client::new().get(format!("{}/export", api.trim_end_matches('/'))),
        token,
    )
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
    Ok(data)
}

async fn import_http(api: &str, token: Option<&str>, data: &ExportData) -> Result<()> {
    bearer(
        reqwest::Client::new().post(format!("{}/import", api.trim_end_matches('/'))),
        token,
    )
    .json(data)
    .send()
    .await?
    .error_for_status()?;
    Ok(())
}

async fn add_word_http(
    api: &str,
    token: Option<&str>,
    word: &str,
    tag: &str,
) -> Result<AddWordResult> {
    let result = bearer(
        reqwest::Client::new().post(format!("{}/words/", api.trim_end_matches('/'))),
        token,
    )
    .json(&serde_json::json!({ "word": word, "tag": tag }))
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
    Ok(result)
}

fn bearer(builder: reqwest::RequestBuilder, token: Option<&str>) -> reqwest::RequestBuilder {
    match token.filter(|t| !t.is_empty()) {
        Some(token) => builder.bearer_auth(token),
        None => builder,
    }
}

fn set_mode(mode: ModeArg) -> Result<()> {
    let path = config_path()?;
    let mut cfg = load_or_default()?;
    cfg.mode = Some(match mode {
        ModeArg::Local => Mode::Local,
        ModeArg::Server => Mode::Server,
    });
    cfg.save(&path)?;
    println!("{}", path.display());
    Ok(())
}

fn config_command(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key } => {
            let cfg = load_or_default()?;
            let value = toml::Value::try_from(&cfg)?;
            if let Some(key) = key {
                let selected = get_toml_key(&value, &key).context("config key not found")?;
                println!("{selected}");
            } else {
                println!("{}", toml::to_string_pretty(&cfg)?);
            }
        }
        ConfigCommand::Set { key, value } => {
            let path = config_path()?;
            let mut cfg_value = toml::Value::try_from(load_or_default()?)?;
            set_toml_key(&mut cfg_value, &key, toml::Value::String(value))?;
            let cfg: AppConfig = cfg_value.try_into()?;
            cfg.save(&path)?;
            println!("{}", path.display());
        }
        ConfigCommand::Path => println!("{}", config_path()?.display()),
        ConfigCommand::Init(args) => {
            let cfg = if args.has_values() {
                init_config_from_args(args)?
            } else {
                init_config(true, false)?
            };
            cfg.save(&config_path()?)?;
            println!("{}", config_path()?.display());
        }
    }
    Ok(())
}

impl InitConfigArgs {
    fn has_values(&self) -> bool {
        self.api_key.is_some()
            || self.base_url.is_some()
            || self.model.is_some()
            || self.target_language.is_some()
    }
}

fn init_config_from_args(args: InitConfigArgs) -> Result<AppConfig> {
    let mut cfg = load_or_default()?;
    let existing_llm = cfg.llm.unwrap_or_default();
    let defaults = LlmConfig::default();
    let api_key = args.api_key.unwrap_or(existing_llm.api_key);
    if api_key.trim().is_empty() {
        anyhow::bail!("--api-key is required when no API key is already configured");
    }

    cfg.mode = Some(Mode::Local);
    cfg.local = Some(cfg.local.unwrap_or_default());
    cfg.llm = Some(LlmConfig {
        api_key,
        base_url: args
            .base_url
            .unwrap_or_else(|| non_empty_or(existing_llm.base_url, defaults.base_url)),
        model: args
            .model
            .unwrap_or_else(|| non_empty_or(existing_llm.model, defaults.model)),
        target_language: args.target_language.unwrap_or_else(|| {
            non_empty_or(existing_llm.target_language, defaults.target_language)
        }),
    });
    Ok(cfg)
}

fn non_empty_or(value: String, fallback: String) -> String {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn ensure_config(server_process: bool) -> Result<AppConfig> {
    let cfg = load_or_default()?;
    if has_required_config(&cfg, server_process) {
        return Ok(cfg);
    }
    let cfg = init_config(false, server_process)?;
    cfg.save(&config_path()?)?;
    Ok(cfg)
}

fn load_or_default() -> Result<AppConfig> {
    let path = config_path()?;
    if path.exists() {
        AppConfig::load(&path)
    } else {
        Ok(AppConfig::default())
    }
}

fn init_config(force: bool, server_process: bool) -> Result<AppConfig> {
    let mut cfg = if force {
        AppConfig::default()
    } else {
        load_or_default()?
    };

    println!("Let's set up Neko Words.");
    println!("Press Enter to accept the default shown in brackets.");
    println!();

    if server_process {
        cfg.server = Some(cfg.server.clone().unwrap_or_default());
        if !has_complete_llm_config(&cfg) {
            cfg.llm = Some(prompt_llm(cfg.llm.as_ref())?);
        }
        return Ok(cfg);
    }

    if cfg.mode.is_none() || force {
        let mode = prompt("Storage mode (local/server)", Some("local"))?;
        cfg.mode = Some(if mode.trim().eq_ignore_ascii_case("server") {
            Mode::Server
        } else {
            Mode::Local
        });
    }

    match cfg.mode.clone().unwrap_or(Mode::Local) {
        Mode::Local => {
            cfg.local = Some(cfg.local.clone().unwrap_or_default());
            cfg.llm = Some(prompt_llm(cfg.llm.as_ref())?);
        }
        Mode::Server => {
            cfg.client_server = Some(ClientServerConfig {
                api_base_url: prompt(
                    "API base URL",
                    cfg.client_server
                        .as_ref()
                        .map(|v| v.api_base_url.as_str())
                        .or(Some("http://localhost:8002/api/v1")),
                )?,
                auth_token: cfg
                    .client_server
                    .as_ref()
                    .and_then(|v| v.auth_token.clone()),
            });
            if force {
                cfg.server = Some(ServerConfig {
                    bind: prompt(
                        "server bind",
                        cfg.server
                            .as_ref()
                            .map(|v| v.bind.as_str())
                            .or(Some("127.0.0.1:8002")),
                    )?,
                    db_path: prompt(
                        "server SQLite path",
                        cfg.server
                            .as_ref()
                            .map(|v| v.db_path.as_str())
                            .or(Some("~/.neko-words/neko-words.sqlite3")),
                    )?,
                    auth_token: cfg.server.as_ref().and_then(|v| v.auth_token.clone()),
                });
                cfg.llm = Some(prompt_llm(cfg.llm.as_ref())?);
            }
        }
    }
    Ok(cfg)
}

fn prompt_llm(existing: Option<&LlmConfig>) -> Result<LlmConfig> {
    Ok(LlmConfig {
        api_key: prompt_required("OpenAI API key", existing.map(|v| v.api_key.as_str()))?,
        base_url: prompt(
            "OpenAI-compatible API base URL",
            existing
                .map(|v| v.base_url.as_str())
                .or(Some("https://api.openai.com/v1")),
        )?,
        model: prompt(
            "Model name",
            existing.map(|v| v.model.as_str()).or(Some("gpt-5.5")),
        )?,
        target_language: prompt(
            "Translation target language",
            existing
                .map(|v| v.target_language.as_str())
                .or(Some("Chinese")),
        )?,
    })
}

fn has_required_config(cfg: &AppConfig, server_process: bool) -> bool {
    if server_process {
        return cfg
            .server
            .as_ref()
            .is_some_and(|v| !v.bind.is_empty() && !v.db_path.is_empty())
            && has_complete_llm_config(cfg);
    }

    match cfg.mode {
        Some(Mode::Local) => {
            cfg.local.as_ref().is_some_and(|v| !v.db_path.is_empty())
                && has_complete_llm_config(cfg)
        }
        Some(Mode::Server) => cfg
            .client_server
            .as_ref()
            .is_some_and(|v| !v.api_base_url.is_empty()),
        None => false,
    }
}

fn has_complete_llm_config(cfg: &AppConfig) -> bool {
    cfg.llm.as_ref().is_some_and(|v| {
        !v.api_key.trim().is_empty()
            && !v.base_url.trim().is_empty()
            && !v.model.trim().is_empty()
            && !v.target_language.trim().is_empty()
    })
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(default) if !default.is_empty() => print!("{label} [{default}]: "),
        _ => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    Ok(if trimmed.is_empty() {
        default.unwrap_or_default().to_string()
    } else {
        trimmed.to_string()
    })
}

fn prompt_required(label: &str, default: Option<&str>) -> Result<String> {
    loop {
        match default {
            Some(default) if !default.is_empty() => print!("{label} [{default}]: "),
            _ => print!("{label}: "),
        }
        io::stdout().flush()?;
        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            anyhow::bail!("{label} is required");
        }
        let trimmed = input.trim();
        let value = if trimmed.is_empty() {
            default.unwrap_or_default().to_string()
        } else {
            trimmed.to_string()
        };
        if !value.trim().is_empty() {
            return Ok(value);
        }
        println!("{label} is required.");
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReviewAction {
    Grade(Grade),
    Quit,
}

fn prompt_grade(single_key: bool) -> Result<Option<Grade>> {
    if single_key {
        return prompt_grade_single_key();
    }

    loop {
        let input = prompt("grade [1=again/2=hard/3=good/4=easy/q=quit]", Some("3"))?;
        if matches!(input.to_ascii_lowercase().as_str(), "q" | "quit") {
            return Ok(None);
        }
        match input.parse() {
            Ok(grade) => return Ok(Some(grade)),
            Err(error) => println!("{error}"),
        }
    }
}

fn prompt_grade_single_key() -> Result<Option<Grade>> {
    print!("grade [1=again/2=hard/3=good/4=easy, Enter=good, q=quit]: ");
    io::stdout().flush()?;
    let term = Term::stdout();

    loop {
        if let Some(action) = review_action_from_key(term.read_key()?) {
            return match action {
                ReviewAction::Grade(grade) => {
                    println!("{} ({})", grade_key(grade), grade_name(grade));
                    Ok(Some(grade))
                }
                ReviewAction::Quit => {
                    println!("q");
                    Ok(None)
                }
            };
        }
    }
}

fn review_action_from_key(key: Key) -> Option<ReviewAction> {
    match key {
        Key::Char('1') => Some(ReviewAction::Grade(Grade::Again)),
        Key::Char('2') => Some(ReviewAction::Grade(Grade::Hard)),
        Key::Char('3') | Key::Enter => Some(ReviewAction::Grade(Grade::Good)),
        Key::Char('4') => Some(ReviewAction::Grade(Grade::Easy)),
        Key::Char('q' | 'Q') | Key::Escape | Key::CtrlC => Some(ReviewAction::Quit),
        _ => None,
    }
}

fn grade_name(grade: Grade) -> &'static str {
    match grade {
        Grade::Again => "again",
        Grade::Hard => "hard",
        Grade::Good => "good",
        Grade::Easy => "easy",
    }
}

fn grade_key(grade: Grade) -> char {
    match grade {
        Grade::Again => '1',
        Grade::Hard => '2',
        Grade::Good => '3',
        Grade::Easy => '4',
    }
}

fn print_add_result(result: AddWordResult) {
    let status = if result.duplicate {
        "duplicate"
    } else {
        "added"
    };
    println!(
        "{}: {} - {}",
        status, result.word.word, result.word.translation
    );
}

fn print_due(item: &DueReview) {
    println!("\n{} - {}", item.word.word, item.word.translation);
    for example in &item.word.examples {
        println!("  {} / {}", example.sentence, example.translation);
    }
}

fn get_toml_key<'a>(value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    key.split('.')
        .try_fold(value, |current, segment| current.get(segment))
}

fn set_toml_key(value: &mut toml::Value, key: &str, new_value: toml::Value) -> Result<()> {
    let mut current = value;
    let mut parts = key.split('.').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current
                .as_table_mut()
                .context("config root is not a table")?
                .insert(part.to_string(), new_value);
            return Ok(());
        }
        current = current
            .as_table_mut()
            .context("config root is not a table")?
            .entry(part)
            .or_insert_with(|| toml::Value::Table(Default::default()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ReviewAction, review_action_from_key, selected_tag};
    use console::Key;
    use neko_core::config::{AppConfig, CliConfig};
    use neko_core::models::Grade;

    #[test]
    fn review_keys_map_to_grades_without_enter() {
        assert_eq!(
            review_action_from_key(Key::Char('1')),
            Some(ReviewAction::Grade(Grade::Again))
        );
        assert_eq!(
            review_action_from_key(Key::Char('2')),
            Some(ReviewAction::Grade(Grade::Hard))
        );
        assert_eq!(
            review_action_from_key(Key::Char('3')),
            Some(ReviewAction::Grade(Grade::Good))
        );
        assert_eq!(
            review_action_from_key(Key::Char('4')),
            Some(ReviewAction::Grade(Grade::Easy))
        );
    }

    #[test]
    fn review_keys_keep_default_and_quit_shortcuts() {
        assert_eq!(
            review_action_from_key(Key::Enter),
            Some(ReviewAction::Grade(Grade::Good))
        );
        assert_eq!(
            review_action_from_key(Key::Char('q')),
            Some(ReviewAction::Quit)
        );
        assert_eq!(review_action_from_key(Key::CtrlC), Some(ReviewAction::Quit));
        assert_eq!(review_action_from_key(Key::ArrowLeft), None);
    }

    #[test]
    fn configured_tag_is_used_unless_explicitly_overridden() {
        let cfg = AppConfig {
            cli: Some(CliConfig {
                default_tag: "en".to_string(),
            }),
            ..AppConfig::default()
        };

        assert_eq!(selected_tag(None, &cfg), "en");
        assert_eq!(selected_tag(Some("jp"), &cfg), "jp");
    }

    #[test]
    fn missing_or_empty_configured_tag_uses_legacy_default() {
        assert_eq!(selected_tag(None, &AppConfig::default()), "default");

        let cfg = AppConfig {
            cli: Some(CliConfig {
                default_tag: String::new(),
            }),
            ..AppConfig::default()
        };
        assert_eq!(selected_tag(None, &cfg), "default");
    }
}
