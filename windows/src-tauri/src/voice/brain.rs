// « OK Nala »'s multi-provider conversational brain: answers open-ended voice
// questions and orchestrates Nala's tools (pills, music, service_info, open_app,
// prepare_email, web_search) across whichever AI backend the user has configured:
//   1. Cloud API keys: Anthropic Claude, OpenAI, Google Gemini, OpenRouter
//   2. Local / self-hosted servers: Ollama, LM Studio, or any OpenAI-compatible server (vLLM, llama.cpp…)
//   3. Active CLI Subscriptions (cli_relay): Claude Code, Gemini CLI, Antigravity CLI, Codex CLI, Cursor CLI, Copilot CLI
//
// The conversation lives in memory only and is forgotten after VOICE_BRAIN_FORGET_SECS.

use std::time::{Duration, Instant};

use reqwest::Url;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::settings::Settings;
use crate::speech::config::{
    VOICE_ANTHROPIC_FAST_MODEL, VOICE_BRAIN_FORGET_SECS, VOICE_BRAIN_MAX_HISTORY,
    VOICE_BRAIN_MAX_STEPS, VOICE_BRAIN_MAX_TOKENS, VOICE_BRAIN_TIMEOUT_SECS,
    VOICE_GOOGLE_FAST_MODEL, VOICE_OPENAI_FAST_MODEL, VOICE_OPENROUTER_FAST_MODEL,
};
use crate::{chat, claude, cli_relay, local_chat, net, openai_compat, secrets, settings};

const FORGET_AFTER: Duration = Duration::from_secs(VOICE_BRAIN_FORGET_SECS);
const TURN_TIMEOUT: Duration = Duration::from_secs(VOICE_BRAIN_TIMEOUT_SECS);

pub struct Brain {
    messages: Vec<Value>,
    last: Instant,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            last: Instant::now(),
        }
    }
}

pub struct Reply {
    pub text: String,
    pub acted: bool,
}

/// Which AI backend will answer a conversational « OK Nala » turn.
enum VoiceBackend {
    Anthropic {
        model: String,
        fallback_model: String,
    },
    OpenAiCloud {
        provider: &'static openai_compat::Provider,
        model: String,
    },
    LocalServer {
        server: local_chat::Server,
        model: String,
    },
    CliRelay {
        provider: &'static str,
        model: String,
    },
}

// ── Provider Readiness & Selection ────────────────────────────────────────────

/// True when the user has at least one AI backend configured:
/// - Cloud API key (Anthropic, OpenAI, Google Gemini, or OpenRouter), OR
/// - Local / custom OpenAI-compatible server (Ollama, LM Studio, custom URL, or `OLLAMA_HOST`), OR
/// - Active & installed CLI Relay subscription (Claude Code, Gemini CLI, Antigravity, Codex, Cursor, Copilot).
pub fn has_ai_provider(settings: &Settings) -> bool {
    has_cloud_api_key() || has_local_server(settings) || has_active_cli_relay(settings)
}

/// Checks provider readiness using the persisted settings (or defaults).
pub fn has_any_provider() -> bool {
    has_ai_provider(&settings::load())
}

/// Checks provider readiness using the in-memory `Shared` settings when available.
pub fn has_any_provider_for(app: &AppHandle) -> bool {
    if let Some(shared) = app.try_state::<crate::Shared>() {
        let s = shared.settings.lock().unwrap().clone();
        return has_ai_provider(&s);
    }
    has_any_provider()
}

fn has_cloud_api_key() -> bool {
    secrets::present(claude::KEY)
        || openai_compat::PROVIDERS
            .iter()
            .any(|p| secrets::present(p.key))
}

pub fn has_local_server(settings: &Settings) -> bool {
    !settings.ollama_url.trim().is_empty()
        || !settings.lmstudio_url.trim().is_empty()
        || !settings.custom_url.trim().is_empty()
        || std::env::var("OLLAMA_HOST")
            .ok()
            .is_some_and(|v| !v.trim().is_empty())
}

fn has_active_cli_relay(settings: &Settings) -> bool {
    cli_relay::PROVIDERS
        .iter()
        .any(|p| cli_relay::uses_subscription(settings, p) && cli_relay::installed(p).is_some())
}

/// Selects the active voice backend: prefers `settings.chat_provider` when it is
/// ready, otherwise falls back across configured API keys, local servers, and CLI relays.
fn resolve_backend(settings: &Settings) -> Option<VoiceBackend> {
    let preferred = settings.chat_provider.trim();
    if !preferred.is_empty() {
        if let Some(b) = backend_for_provider(settings, preferred) {
            return Some(b);
        }
    }

    // 1. Cloud API keys in priority order: Anthropic -> OpenAI -> Google -> OpenRouter
    if secrets::present(claude::KEY) {
        return Some(VoiceBackend::Anthropic {
            model: VOICE_ANTHROPIC_FAST_MODEL.into(),
            fallback_model: settings.model.trim().to_string(),
        });
    }
    for p in openai_compat::PROVIDERS {
        if secrets::present(p.key) {
            return Some(VoiceBackend::OpenAiCloud {
                provider: p,
                model: fast_cloud_model(settings, p),
            });
        }
    }

    // 2. Local / custom servers: Ollama -> LM Studio -> Custom
    for id in ["ollama", "lmstudio", "custom"] {
        if let Some(b) = local_backend(settings, id) {
            return Some(b);
        }
    }

    // 3. Active CLI Relay subscriptions
    for p in cli_relay::PROVIDERS {
        if cli_relay::uses_subscription(settings, p) && cli_relay::installed(p).is_some() {
            let model = chat::model_for(settings, p);
            return Some(VoiceBackend::CliRelay { provider: p, model });
        }
    }

    None
}

fn backend_for_provider(settings: &Settings, provider: &str) -> Option<VoiceBackend> {
    if cli_relay::uses_subscription(settings, provider) && cli_relay::installed(provider).is_some() {
        let StaticProvider(p) = static_cli_provider(provider)?;
        let model = chat::model_for(settings, p);
        return Some(VoiceBackend::CliRelay { provider: p, model });
    }
    if provider == chat::ANTHROPIC && secrets::present(claude::KEY) {
        return Some(VoiceBackend::Anthropic {
            model: VOICE_ANTHROPIC_FAST_MODEL.into(),
            fallback_model: settings.model.trim().to_string(),
        });
    }
    if let Some(p) = openai_compat::provider(provider) {
        if secrets::present(p.key) {
            return Some(VoiceBackend::OpenAiCloud {
                provider: p,
                model: fast_cloud_model(settings, p),
            });
        }
    }
    local_backend(settings, provider)
}

struct StaticProvider(&'static str);

fn static_cli_provider(id: &str) -> Option<StaticProvider> {
    cli_relay::PROVIDERS
        .iter()
        .copied()
        .find(|p| *p == id)
        .map(StaticProvider)
}

fn fast_cloud_model(settings: &Settings, p: &'static openai_compat::Provider) -> String {
    if let Some(picked) = settings.chat_models.get(p.id).map(|m| m.trim()).filter(|m| !m.is_empty()) {
        return picked.to_string();
    }
    match p.id {
        "openai" => VOICE_OPENAI_FAST_MODEL.into(),
        "google" => VOICE_GOOGLE_FAST_MODEL.into(),
        "openrouter" => VOICE_OPENROUTER_FAST_MODEL.into(),
        _ => p.default_model.into(),
    }
}

fn local_backend(settings: &Settings, id: &str) -> Option<VoiceBackend> {
    let mut srv = local_chat::server(settings, id)?;
    if srv.url.trim().is_empty() && id == "ollama" {
        if let Ok(env_url) = std::env::var("OLLAMA_HOST") {
            if !env_url.trim().is_empty() {
                srv.url = env_url;
            }
        }
    }
    if srv.url.trim().is_empty() {
        return None;
    }
    let model = settings
        .chat_models
        .get(id)
        .map(|m| m.trim().to_string())
        .unwrap_or_default();
    Some(VoiceBackend::LocalServer { server: srv, model })
}

// ── Tool Schemas & Universal Adapter ──────────────────────────────────────────

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "pills",
            "description": "Add, remove, set as main or list the pills shown in the island at the top of the screen. At most 4 pills besides the main one: remove one first if needed.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["add", "remove", "set_main", "list"] },
                    "names": { "type": "array", "items": { "type": "string" }, "description": "Pill names as listed (e.g. Stripe, GitHub, Cursor). Empty for list." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "music",
            "description": "Control Spotify: play or pause, next, previous, set the volume, or search a song, an artist or a playlist in Spotify.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["play_pause", "next", "previous", "set_volume", "search"] },
                    "query": { "type": "string", "description": "What to search, for search." },
                    "volume": { "type": "integer", "description": "0-100 for set_volume." }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "service_info",
            "description": "Read the user's real data that Nala already has, as JSON: the active pills, the agent sessions (Claude Code, Codex…) and what they are doing, Claude or Codex plan usage, Spotify now playing, and the services' latest data (Stripe, GitHub, Vercel, Resend, n8n, Notion, Cal.com).",
            "input_schema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "open_app",
            "description": "Open an app on this computer by name (Firefox, VS Code, Notion, Spotify…).",
            "input_schema": { "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }
        }),
        json!({
            "name": "prepare_email",
            "description": "Open the user's mail app with an email filled in, for them to check and send (it is never sent automatically). The recipient must be an email address.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "recipient": { "type": "string", "description": "An email address." },
                    "subject": { "type": "string" },
                    "body": { "type": "string", "description": "The full email text, written for the user." }
                },
                "required": ["recipient", "subject", "body"]
            }
        }),
        json!({ "type": "web_search_20250305", "name": "web_search", "max_uses": 3 }),
    ]
}

/// Converts Nala's canonical Anthropic tool definitions into the OpenAI-compatible
/// Function Calling format supported by OpenAI, Google Gemini, OpenRouter, Ollama,
/// LM Studio, and vLLM. Server-side Anthropic tools (`web_search_20250305`) are skipped.
pub fn openai_tools(anthropic_tools: &[Value]) -> Vec<Value> {
    anthropic_tools
        .iter()
        .filter_map(|t| {
            let name = t.get("name")?.as_str()?;
            let schema = t.get("input_schema")?;
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            Some(json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": schema
                }
            }))
        })
        .collect()
}

/// Converts the system prompt and `Brain`'s canonical conversation history into
/// OpenAI-compatible chat messages (`system`, `user`, `assistant` with `tool_calls`,
/// and `tool` with `tool_call_id`).
pub fn to_openai_messages(system: &str, messages: &[Value]) -> Vec<Value> {
    let mut out = Vec::with_capacity(messages.len() + 1);
    out.push(json!({ "role": "system", "content": system }));
    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("user");
        match (role, &msg["content"]) {
            ("user", Value::String(text)) => {
                out.push(json!({ "role": "user", "content": text }));
            }
            ("user", Value::Array(blocks)) => {
                for b in blocks {
                    if b["type"].as_str() == Some("tool_result") {
                        let call_id = b["tool_use_id"].as_str().unwrap_or("call_0");
                        let content = b["content"].as_str().unwrap_or("");
                        out.push(json!({
                            "role": "tool",
                            "tool_call_id": call_id,
                            "content": content
                        }));
                    }
                }
            }
            ("assistant", Value::Array(blocks)) => {
                let text: String = blocks
                    .iter()
                    .filter(|b| b["type"].as_str() == Some("text"))
                    .filter_map(|b| b["text"].as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let tool_calls: Vec<Value> = blocks
                    .iter()
                    .filter(|b| b["type"].as_str() == Some("tool_use"))
                    .enumerate()
                    .map(|(idx, b)| {
                        let id = b["id"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .unwrap_or_else(|| format!("call_{idx}"));
                        let name = b["name"].as_str().unwrap_or("");
                        let args = serde_json::to_string(&b["input"]).unwrap_or_else(|_| "{}".into());
                        json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args
                            }
                        })
                    })
                    .collect();
                if tool_calls.is_empty() {
                    out.push(json!({ "role": "assistant", "content": text }));
                } else {
                    let mut assistant = json!({
                        "role": "assistant",
                        "tool_calls": tool_calls
                    });
                    if !text.trim().is_empty() {
                        assistant["content"] = json!(text);
                    }
                    out.push(assistant);
                }
            }
            _ => {}
        }
    }
    out
}

/// Converts an OpenAI-compatible Chat Completions response (`/choices/0/message`)
/// into `Brain`'s canonical turn format (`{ "content": [...], "stop_reason": ... }`).
pub fn from_openai_response(response: &Value) -> Result<Value, String> {
    if let Some(msg) = response.pointer("/error/message").and_then(Value::as_str) {
        return Err(msg.to_string());
    }
    let choice = response
        .pointer("/choices/0")
        .ok_or_else(|| "missing choices[0] in response".to_string())?;
    let message = &choice["message"];
    let mut content: Vec<Value> = Vec::new();

    if let Some(raw_text) = message["content"].as_str() {
        let cleaned = strip_thinking_blocks(raw_text);
        if !cleaned.is_empty() {
            content.push(json!({ "type": "text", "text": cleaned }));
        }
    }

    if let Some(calls) = message["tool_calls"].as_array() {
        for (idx, tc) in calls.iter().enumerate() {
            let name = tc
                .pointer("/function/name")
                .or_else(|| tc.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let id = tc["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("call_{idx}"));
            let raw_args = tc
                .pointer("/function/arguments")
                .or_else(|| tc.get("arguments"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let input = match raw_args {
                Value::String(s) => serde_json::from_str::<Value>(&s).unwrap_or_else(|_| json!({})),
                Value::Object(_) => raw_args,
                _ => json!({}),
            };
            content.push(json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": input
            }));
        }
    }

    let has_tools = content.iter().any(|b| b["type"].as_str() == Some("tool_use"));
    let stop_reason = if has_tools { "tool_use" } else { "end_turn" };
    Ok(json!({
        "content": content,
        "stop_reason": stop_reason
    }))
}

/// Removes `<think>…</think>` blocks emitted by reasoning models (e.g. DeepSeek-R1, QwQ).
fn strip_thinking_blocks(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<think>") {
        out.push_str(&rest[..start]);
        match rest[start..].find("</think>") {
            Some(end) => rest = &rest[start + end + "</think>".len()..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out.trim().to_string()
}

fn system_prompt(pills: &str) -> String {
    let t = crate::platform::local_time();
    let os = if cfg!(windows) { "Windows" } else { "Linux" };
    format!(
        "You are Nala, a voice assistant living at the top of the screen of the user's {os} computer, a bit like Jarvis: \
quick, warm, a little playful. It is {:04}-{:02}-{:02} {:02}:{:02}.\n\
Everything you write is read aloud: answer in the language the user spoke, in one or two short sentences, \
no markdown, no lists, no emojis, no links.\n\
Act with the tools instead of describing: pills, music, service_info for the user's own data, open_app, \
prepare_email, and web_search for anything current (weather, news, scores, prices, facts you are not sure of). \
Never invent the user's data.\n\
Email: gather the recipient's address, the subject and what to say (write the text yourself from what the user \
wants to say, in the language they used), asking for what is missing one short question at a time, then call \
prepare_email. The email is never sent by you: the user checks it in their mail app and sends it.\n\
When you need an answer from the user, end with one short question. Otherwise never end with a question.\n\
Pills: {pills}",
        t.year, t.month, t.day, t.hour, t.minute
    )
}

// ── Execution Across Providers ────────────────────────────────────────────────

impl Brain {
    pub fn respond(&mut self, app: &AppHandle, transcript: &str) -> Option<Reply> {
        if self.last.elapsed() > FORGET_AFTER {
            self.messages.clear();
        }
        self.last = Instant::now();

        let settings = app
            .try_state::<crate::Shared>()
            .map(|s| s.settings.lock().unwrap().clone())
            .unwrap_or_else(settings::load);
        let backend = resolve_backend(&settings)?;

        self.messages.push(json!({ "role": "user", "content": transcript }));
        self.trim();
        let pills = super::ask_island(app, "pills", json!({ "action": "list" })).unwrap_or_default();
        let system = system_prompt(&pills);
        let tools = tools();
        let mut acted = false;
        let mut spoken = String::new();

        for _ in 0..VOICE_BRAIN_MAX_STEPS {
            let turn = tauri::async_runtime::block_on(dispatch_turn(
                &backend,
                &system,
                &self.messages,
                &tools,
            ));
            let turn = match turn {
                Ok(t) => t,
                Err(err) => {
                    crate::log::line(format!("voice: brain turn failed: {err}"));
                    if self.messages.last().is_some_and(|m| m["content"].is_string()) {
                        self.messages.pop();
                    }
                    return acted.then(|| Reply { text: String::new(), acted });
                }
            };
            let content = turn["content"].as_array().cloned().unwrap_or_default();
            self.messages.push(json!({ "role": "assistant", "content": content.clone() }));
            let text: String = content
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if !text.trim().is_empty() {
                spoken = text;
            }
            let stop = turn["stop_reason"].as_str().unwrap_or("end_turn");
            if stop == "pause_turn" {
                continue;
            }
            let uses: Vec<&Value> = content.iter().filter(|b| b["type"] == "tool_use").collect();
            if stop != "tool_use" || uses.is_empty() {
                break;
            }
            let mut results = Vec::new();
            for u in uses {
                let (result, did) = super::run_tool(app, u["name"].as_str().unwrap_or(""), &u["input"]);
                acted |= did;
                results.push(json!({ "type": "tool_result", "tool_use_id": u["id"], "content": result }));
            }
            self.messages.push(json!({ "role": "user", "content": results }));
            spoken.clear();
        }
        self.repair();
        Some(Reply {
            text: spoken_text(&spoken),
            acted,
        })
    }

    /// Keeps the last exchanges, never cutting between a tool call and its result.
    fn trim(&mut self) {
        if self.messages.len() <= VOICE_BRAIN_MAX_HISTORY {
            return;
        }
        let mut cut = self.messages.len() - VOICE_BRAIN_MAX_HISTORY;
        while cut < self.messages.len()
            && !(self.messages[cut]["role"] == "user" && self.messages[cut]["content"].is_string())
        {
            cut += 1;
        }
        self.messages.drain(..cut.min(self.messages.len() - 1));
    }

    /// The API refuses a history ending on a tool call with no result after it.
    fn repair(&mut self) {
        let Some(last) = self.messages.last() else { return };
        if last["role"] != "assistant" {
            return;
        }
        let calls = last["content"]
            .as_array()
            .is_some_and(|c| c.iter().any(|b| b["type"] == "tool_use"));
        if calls {
            self.messages.pop();
        }
    }
}

async fn dispatch_turn(
    backend: &VoiceBackend,
    system: &str,
    messages: &[Value],
    tools: &[Value],
) -> Result<Value, String> {
    match backend {
        VoiceBackend::Anthropic { model, fallback_model } => {
            match claude::voice_turn(model, system, messages, tools).await {
                Ok(v) => Ok(v),
                Err(err) if !fallback_model.is_empty() && fallback_model != model => {
                    crate::log::line(format!("voice: {err}; falling back to {fallback_model}"));
                    claude::voice_turn(fallback_model, system, messages, tools).await
                }
                Err(err) => Err(err),
            }
        }
        VoiceBackend::OpenAiCloud { provider, model } => {
            let key = secrets::get(provider.key)
                .ok_or_else(|| format!("{} API key missing", provider.name))?;
            let base = Url::parse(provider.base_url).map_err(|e| e.to_string())?;
            let endpoint = Url::parse(&net::join(&base, "chat/completions")).map_err(|e| e.to_string())?;
            openai_voice_turn(
                &endpoint,
                Some(&key),
                model,
                provider.max_tokens_field,
                system,
                messages,
                tools,
            )
            .await
        }
        VoiceBackend::LocalServer { server, model } => {
            let base = net::normalise_server_url(&server.url)?;
            let resolved_model = if !model.trim().is_empty() {
                model.clone()
            } else {
                local_chat::models(server)
                    .await?
                    .into_iter()
                    .next()
                    .map(|m| m.id)
                    .ok_or_else(|| format!("No chat models found on {}", server.name))?
            };
            let endpoint = Url::parse(&net::join(&base, "v1/chat/completions")).map_err(|e| e.to_string())?;
            let key = server.key.as_deref().unwrap_or("ollama");
            openai_voice_turn(
                &endpoint,
                Some(key),
                &resolved_model,
                "max_tokens",
                system,
                messages,
                tools,
            )
            .await
        }
        VoiceBackend::CliRelay { provider, model } => {
            cli_relay_voice_turn(provider, model, system, messages, tools).await
        }
    }
}

/// Calls an OpenAI-compatible `/chat/completions` endpoint (OpenAI, Google Gemini,
/// OpenRouter, Ollama, LM Studio, or vLLM) with OpenAI function tools.
/// Automatically retries without `"tools"` if the target model does not support tool calling.
async fn openai_voice_turn(
    endpoint: &Url,
    key: Option<&str>,
    model: &str,
    max_tokens_field: &str,
    system: &str,
    messages: &[Value],
    anthropic_tools: &[Value],
) -> Result<Value, String> {
    let oai_messages = to_openai_messages(system, messages);
    let oai_tools = openai_tools(anthropic_tools);

    let mut body = json!({
        "model": model,
        "messages": oai_messages,
    });
    body[max_tokens_field] = json!(VOICE_BRAIN_MAX_TOKENS);
    if !oai_tools.is_empty() {
        body["tools"] = json!(oai_tools);
    }

    match post_openai_json(endpoint, key, &body).await {
        Ok(resp) => from_openai_response(&resp),
        Err((status, detail)) if matches!(status, 400 | 404 | 422 | 501) && body.get("tools").is_some() => {
            crate::log::line(format!(
                "voice: model {model} rejected tools ({status}: {detail}); retrying without tools"
            ));
            let mut plain_body = json!({
                "model": model,
                "messages": oai_messages,
            });
            plain_body[max_tokens_field] = json!(VOICE_BRAIN_MAX_TOKENS);
            let resp = post_openai_json(endpoint, key, &plain_body)
                .await
                .map_err(|(s, d)| format!("HTTP {s}: {d}"))?;
            from_openai_response(&resp)
        }
        Err((status, detail)) => Err(format!("HTTP {status}: {detail}")),
    }
}

async fn post_openai_json(
    endpoint: &Url,
    key: Option<&str>,
    body: &Value,
) -> Result<Value, (u16, String)> {
    let mut req = net::client(endpoint, TURN_TIMEOUT)
        .map_err(|e| (0, e))?
        .post(endpoint.clone())
        .header("content-type", "application/json");
    if let Some(k) = key.filter(|k| !k.is_empty()) {
        req = req.bearer_auth(k);
    }
    let response = req
        .json(body)
        .send()
        .await
        .map_err(|e| (0, e.to_string()))?;
    let status = response.status();
    if !status.is_success() {
        let bytes = net::read_capped(response, net::MAX_ERROR_BODY)
            .await
            .unwrap_or_default();
        return Err((status.as_u16(), net::error_detail(&bytes)));
    }
    let bytes = net::read_capped(response, net::MAX_BODY)
        .await
        .map_err(|e| (status.as_u16(), e))?;
    serde_json::from_slice(&bytes).map_err(|e| (status.as_u16(), e.to_string()))
}

/// Executes a voice brain turn through an installed CLI subscription (`cli_relay`),
/// supporting structured `<tool_call>` blocks when the model invokes one of Nala's tools.
async fn cli_relay_voice_turn(
    provider: &str,
    model: &str,
    system: &str,
    messages: &[Value],
    anthropic_tools: &[Value],
) -> Result<Value, String> {
    let tool_specs = openai_tools(anthropic_tools);
    let augmented_system = format!(
        "{system}\n\n\
If you need to invoke one of Nala's local tools (pills, music, service_info, open_app, prepare_email), \
output ONLY a JSON tool call wrapped in <tool_call></tool_call> tags, for example:\n\
<tool_call>{{\"name\":\"service_info\",\"input\":{{}}}}</tool_call>\n\
Available tools: {}",
        serde_json::to_string(&tool_specs).unwrap_or_default()
    );
    let prompt = format_cli_conversation(messages);
    let raw = cli_relay::ask_prompt(provider, model, &augmented_system, &prompt, TURN_TIMEOUT).await?;
    Ok(parse_cli_reply(&raw))
}

fn format_cli_conversation(messages: &[Value]) -> String {
    let mut out = String::new();
    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("user");
        match &msg["content"] {
            Value::String(s) => out.push_str(&format!("<{role}>\n{s}\n</{role}>\n")),
            Value::Array(blocks) => {
                for b in blocks {
                    match b["type"].as_str() {
                        Some("text") => {
                            if let Some(t) = b["text"].as_str() {
                                out.push_str(&format!("<{role}>\n{t}\n</{role}>\n"));
                            }
                        }
                        Some("tool_use") => {
                            out.push_str(&format!(
                                "<assistant_tool_call name=\"{}\">{}</assistant_tool_call>\n",
                                b["name"].as_str().unwrap_or(""),
                                b["input"]
                            ));
                        }
                        Some("tool_result") => {
                            out.push_str(&format!(
                                "<tool_result>\n{}\n</tool_result>\n",
                                b["content"].as_str().unwrap_or("")
                            ));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Parses a CLI relay response into canonical `content` blocks, extracting any
/// `<tool_call>{"name": "...", "input": {...}}</tool_call>` blocks.
pub fn parse_cli_reply(raw: &str) -> Value {
    let cleaned = strip_thinking_blocks(raw);
    let mut content: Vec<Value> = Vec::new();
    let mut rest = cleaned.as_str();
    let mut idx = 0usize;

    while let Some(start) = rest.find("<tool_call>") {
        let before = rest[..start].trim();
        if !before.is_empty() {
            content.push(json!({ "type": "text", "text": before }));
        }
        let after_open = &rest[start + "<tool_call>".len()..];
        let Some(end) = after_open.find("</tool_call>") else {
            break;
        };
        let payload = after_open[..end].trim();
        if let Ok(parsed) = serde_json::from_str::<Value>(payload) {
            if let Some(name) = parsed["name"].as_str().filter(|s| !s.is_empty()) {
                let input = parsed
                    .get("input")
                    .or_else(|| parsed.get("arguments"))
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                content.push(json!({
                    "type": "tool_use",
                    "id": format!("cli_call_{idx}"),
                    "name": name,
                    "input": input
                }));
                idx += 1;
            }
        }
        rest = &after_open[end + "</tool_call>".len()..];
    }

    let trailing = rest.trim();
    if !trailing.is_empty() {
        content.push(json!({ "type": "text", "text": trailing }));
    }

    let has_tools = content.iter().any(|b| b["type"].as_str() == Some("tool_use"));
    json!({
        "content": content,
        "stop_reason": if has_tools { "tool_use" } else { "end_turn" }
    })
}

/// What the model wrote, ready to be said: citations and markdown left out.
pub fn spoken_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut skip = 0;
    for c in text.chars() {
        match c {
            '[' => skip += 1,
            ']' if skip > 0 => skip -= 1,
            '*' | '#' | '`' | '_' if skip == 0 => {}
            _ if skip == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_and_citations_are_not_read_aloud() {
        assert_eq!(
            spoken_text("**Done** — Stripe [1] is in the island."),
            "Done — Stripe is in the island."
        );
        assert_eq!(
            spoken_text("  Il fait  20 °C à Bordeaux. "),
            "Il fait 20 °C à Bordeaux."
        );
    }

    #[test]
    fn every_tool_has_a_schema() {
        for tool in tools() {
            assert!(tool["name"].is_string());
            assert!(tool.get("input_schema").is_some() || tool["type"] == "web_search_20250305");
        }
    }

    #[test]
    fn maps_tools_to_openai_function_format() {
        let mapped = openai_tools(&tools());
        // 5 local function tools (pills, music, service_info, open_app, prepare_email);
        // Anthropic's proprietary web_search_20250305 is excluded.
        assert_eq!(mapped.len(), 5);
        for item in &mapped {
            assert_eq!(item["type"], "function");
            assert!(item["function"]["name"].is_string());
            assert!(item["function"]["description"].is_string());
            assert_eq!(item["function"]["parameters"]["type"], "object");
        }
    }

    #[test]
    fn converts_messages_and_tool_calls_to_and_from_openai_format() {
        let history = vec![
            json!({ "role": "user", "content": "Putar lagu Hindia" }),
            json!({
                "role": "assistant",
                "content": [
                    {
                        "type": "tool_use",
                        "id": "call_42",
                        "name": "music",
                        "input": { "action": "search", "query": "Hindia" }
                    }
                ]
            }),
            json!({
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "call_42",
                        "content": "Spotify opened"
                    }
                ]
            }),
        ];

        let oai = to_openai_messages("SYS", &history);
        assert_eq!(oai.len(), 4);
        assert_eq!(oai[0]["role"], "system");
        assert_eq!(oai[1]["role"], "user");
        assert_eq!(oai[2]["role"], "assistant");
        assert_eq!(oai[2]["tool_calls"][0]["id"], "call_42");
        assert_eq!(oai[2]["tool_calls"][0]["function"]["name"], "music");
        assert_eq!(oai[3]["role"], "tool");
        assert_eq!(oai[3]["tool_call_id"], "call_42");

        // Parse an OpenAI tool_calls response (stringified arguments)
        let oai_tool_reply = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_99",
                        "type": "function",
                        "function": {
                            "name": "open_app",
                            "arguments": "{\"name\":\"Spotify\"}"
                        }
                    }]
                }
            }]
        });
        let canonical = from_openai_response(&oai_tool_reply).unwrap();
        assert_eq!(canonical["stop_reason"], "tool_use");
        assert_eq!(canonical["content"][0]["type"], "tool_use");
        assert_eq!(canonical["content"][0]["name"], "open_app");
        assert_eq!(canonical["content"][0]["input"]["name"], "Spotify");

        // Parse an Ollama-style final text response with <think> blocks
        let ollama_reply = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "<think>Analyzing user request...</think>Spotify sudah dibuka untuk Anda."
                }
            }]
        });
        let canonical_text = from_openai_response(&ollama_reply).unwrap();
        assert_eq!(canonical_text["stop_reason"], "end_turn");
        assert_eq!(
            canonical_text["content"][0]["text"],
            "Spotify sudah dibuka untuk Anda."
        );
    }

    #[test]
    fn parses_cli_relay_tool_call_blocks() {
        let raw = "<tool_call>{\"name\":\"pills\",\"input\":{\"action\":\"add\",\"names\":[\"Stripe\"]}}</tool_call>";
        let parsed = parse_cli_reply(raw);
        assert_eq!(parsed["stop_reason"], "tool_use");
        assert_eq!(parsed["content"][0]["name"], "pills");
        assert_eq!(parsed["content"][0]["input"]["action"], "add");

        let plain = parse_cli_reply("Hari ini cuaca cerah di Jakarta.");
        assert_eq!(plain["stop_reason"], "end_turn");
        assert_eq!(plain["content"][0]["text"], "Hari ini cuaca cerah di Jakarta.");
    }

    #[test]
    fn detects_local_servers_in_settings() {
        let mut s = Settings::default();
        assert!(!s.ollama_url.trim().is_empty() || s.lmstudio_url.trim().is_empty());
        s.ollama_url = "http://127.0.0.1:11434".into();
        assert!(has_local_server(&s));
    }
}
