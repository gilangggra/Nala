// Speech to text on this computer: OpenAI's Whisper (multilingual with Indonesian)
// or NVIDIA's Parakeet (25 languages) through sherpa-onnx, and the small models
// that tell the voices of a call apart. The Mac runs Parakeet through FluidAudio.
//
// Nothing is downloaded until the user clicks Download in Nala → Settings.
// The files come from Hugging Face at a pinned revision, are checked against
// their SHA-256 and kept in %LOCALAPPDATA%\Coucou\Models (Windows) or
// ~/.local/share/coucou/Models (Linux). From then on everything runs offline.

pub mod audio;

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineSpeakerDiarization, OfflineSpeakerDiarizationConfig,
    OfflineTransducerModelConfig, OfflineWhisperModelConfig,
};
use tauri::{AppHandle, Emitter};

use crate::notes::builder::{TimedWord, Turn};

/// Every model works on 16 kHz mono.
pub const SAMPLE_RATE: u32 = 16_000;

struct ModelFile {
    name: &'static str,
    url: &'static str,
    size: u64,
    sha256: &'static str,
}

/// A set of files downloaded together.
pub struct ModelSet {
    pub id: &'static str,
    files: &'static [ModelFile],
}

impl ModelSet {
    pub fn dir(&self) -> PathBuf {
        models_dir().join(self.id)
    }

    fn path(&self, name: &str) -> String {
        self.dir().join(name).to_string_lossy().to_string()
    }

    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Every file is there with its size (the hash was checked when it came in).
    pub fn is_present(&self) -> bool {
        self.files.iter().all(|f| std::fs::metadata(self.dir().join(f.name)).map(|m| m.len() == f.size).unwrap_or(false))
    }
}

/// OpenAI Whisper Large-v3-Turbo int8: state-of-the-art multilingual speech recognition
/// supporting Indonesian and 99+ languages with high accuracy and automatic punctuation (about 1000 MB).
pub static WHISPER: ModelSet = ModelSet {
    id: "whisper-large-v3-turbo-int8",
    files: &[
        ModelFile {
            name: "turbo-encoder.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-turbo/resolve/2ca6ff69fc878651b770880507669577ac41c2ff/turbo-encoder.int8.onnx",
            size: 674_716_297,
            sha256: "b02dcdf54f348741e93fe732b67d933c8dcb6735655f710640143081db38878b",
        },
        ModelFile {
            name: "turbo-decoder.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-turbo/resolve/2ca6ff69fc878651b770880507669577ac41c2ff/turbo-decoder.int8.onnx",
            size: 361_080_764,
            sha256: "20accd02388482eb3a46bd615631adfdc85e1eb2c7db9ea3f02a40ffe6b81547",
        },
        ModelFile {
            name: "turbo-tokens.txt",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-turbo/resolve/2ca6ff69fc878651b770880507669577ac41c2ff/turbo-tokens.txt",
            size: 816_730,
            sha256: "b34b360dbb493e781e479794586d661700670d65564001f23024971d1f2fa126",
        },
    ],
};

/// Parakeet TDT 0.6B v3, int8: the dictation and the calls' words (about 670 MB).
pub static PARAKEET: ModelSet = ModelSet {
    id: "parakeet-tdt-0.6b-v3-int8",
    files: &[
        ModelFile {
            name: "encoder.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/encoder.int8.onnx",
            size: 652_184_281,
            sha256: "acfc2b4456377e15d04f0243af540b7fe7c992f8d898d751cf134c3a55fd2247",
        },
        ModelFile {
            name: "decoder.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/decoder.int8.onnx",
            size: 11_845_275,
            sha256: "179e50c43d1a9de79c8a24149a2f9bac6eb5981823f2a2ed88d655b24248db4e",
        },
        ModelFile {
            name: "joiner.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/joiner.int8.onnx",
            size: 6_355_277,
            sha256: "3164c13fc2821009440d20fcb5fdc78bff28b4db2f8d0f0b329101719c0948b3",
        },
        ModelFile {
            name: "tokens.txt",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/tokens.txt",
            size: 93_939,
            sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
        },
    ],
};

/// pyannote's segmentation and a speaker embedding model: who speaks when, in a
/// call (about 30 MB).
pub static DIARIZER: ModelSet = ModelSet {
    id: "diarizer",
    files: &[
        ModelFile {
            name: "segmentation.int8.onnx",
            url: "https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0/resolve/9403a6902bb58e3d5ae8c7e77c3422de279db2e0/model.int8.onnx",
            size: 1_540_506,
            sha256: "d582f4b4c6b48205de7e0643c57df0df5615a3c176189be3fc461e9d18827b5d",
        },
        ModelFile {
            name: "embedding.onnx",
            url: "https://huggingface.co/csukuangfj/speaker-embedding-models/resolve/0743f301363dec56491a490f6d6cbc9d67f9a3bf/3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx",
            size: 28_281_164,
            sha256: "aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2",
        },
    ],
};

pub fn models_dir() -> PathBuf {
    crate::settings::local_dir().join("Models")
}

// ── Download state ────────────────────────────────────────────────────────────

/// What the Settings page shows about the models.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum ModelState {
    Missing,
    Downloading { fraction: f64 },
    Ready,
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsStatus {
    pub parakeet: ModelState,
    pub whisper: ModelState,
    pub active_engine: String,
    pub language: String,
    /// Size of the active download in MB.
    pub download_mb: u64,
}

struct Downloads {
    running: bool,
    fraction: f64,
    error: Option<String>,
    target: String,
}

fn downloads() -> &'static Mutex<Downloads> {
    static D: OnceLock<Mutex<Downloads>> = OnceLock::new();
    D.get_or_init(|| Mutex::new(Downloads {
        running: false,
        fraction: 0.0,
        error: None,
        target: "whisper".into(),
    }))
}

struct SpeechPrefs {
    engine: String,
    language: String,
}

fn speech_prefs() -> &'static Mutex<SpeechPrefs> {
    static P: OnceLock<Mutex<SpeechPrefs>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(SpeechPrefs {
        engine: "whisper".into(),
        language: "auto".into(),
    }))
}

/// Keeps the speech module in step with Flow's preferred engine and language.
pub fn set_prefs(engine: &str, language: &str) {
    let mut p = speech_prefs().lock().unwrap_or_else(|e| e.into_inner());
    p.engine = if engine.trim() == "parakeet" { "parakeet".into() } else { "whisper".into() };
    let lang = language.trim();
    p.language = if lang.is_empty() { "auto".into() } else { lang.into() };
}

/// Picks the engine to run: the requested or preferred one when its models are
/// on disk, else whichever model set is already installed, else the wanted one.
pub fn resolve_engine(requested: Option<&str>, whisper_present: bool, parakeet_present: bool) -> &'static str {
    let wanted = match requested.map(str::trim).filter(|s| !s.is_empty()) {
        Some("parakeet") => "parakeet",
        Some("whisper") => "whisper",
        _ => {
            let pref = speech_prefs().lock().unwrap_or_else(|e| e.into_inner()).engine.clone();
            if pref == "parakeet" { "parakeet" } else { "whisper" }
        }
    };
    match (wanted, whisper_present, parakeet_present) {
        ("parakeet", _, true) => "parakeet",
        ("whisper", true, _) => "whisper",
        (_, true, false) => "whisper",
        (_, false, true) => "parakeet",
        ("parakeet", false, false) => "parakeet",
        _ => "whisper",
    }
}

/// Normalizes a dictation language preference to Whisper's ISO 639-1 code, or
/// `None` when Whisper should auto-detect the spoken language from audio.
///
/// When `requested` is `"auto"` (or empty/`None` and the stored preference is
/// `"auto"`): returns `Some("id")` when the interface/system language `ui_lang`
/// is Indonesian (`"id"`), so short Indonesian phrases and wake words are not
/// misdetected as Malay or English; otherwise returns `None` so Whisper runs
/// multilingual auto-detection instead of forcing Indonesian on non-Indonesian users.
pub fn resolve_whisper_language(requested: Option<&str>, ui_lang: &str) -> Option<String> {
    let raw = match requested.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => s.to_string(),
        None => speech_prefs().lock().unwrap_or_else(|e| e.into_inner()).language.clone(),
    };
    let tag = raw.trim().replace('_', "-").to_lowercase();
    if tag.is_empty() || tag == "auto" {
        let ui_base = ui_lang.trim().replace('_', "-").to_lowercase();
        let ui_primary = ui_base.split('-').next().unwrap_or("");
        if ui_primary == "id" {
            return Some("id".into());
        }
        return None;
    }
    let primary = tag.split('-').next().unwrap_or("");
    if primary.is_empty() {
        return None;
    }
    Some(primary.to_string())
}

/// Computes the state of models for settings and status events.
pub fn status() -> ModelsStatus {
    let d = downloads().lock().unwrap();
    let pref = speech_prefs().lock().unwrap_or_else(|e| e.into_inner());

    let parakeet = if d.running && d.target == "parakeet" {
        ModelState::Downloading { fraction: d.fraction }
    } else if PARAKEET.is_present() && DIARIZER.is_present() {
        ModelState::Ready
    } else if d.target == "parakeet" {
        if let Some(error) = &d.error {
            ModelState::Failed { error: error.clone() }
        } else {
            ModelState::Missing
        }
    } else {
        ModelState::Missing
    };

    let whisper = if d.running && d.target == "whisper" {
        ModelState::Downloading { fraction: d.fraction }
    } else if WHISPER.is_present() && DIARIZER.is_present() {
        ModelState::Ready
    } else if d.target == "whisper" {
        if let Some(error) = &d.error {
            ModelState::Failed { error: error.clone() }
        } else {
            ModelState::Missing
        }
    } else {
        ModelState::Missing
    };

    let active_engine = resolve_engine(Some(&pref.engine), WHISPER.is_present(), PARAKEET.is_present()).to_string();

    let download_mb = if active_engine == "whisper" {
        (WHISPER.total_size() + DIARIZER.total_size()) / 1_000_000
    } else {
        (PARAKEET.total_size() + DIARIZER.total_size()) / 1_000_000
    };

    ModelsStatus {
        parakeet,
        whisper,
        active_engine,
        language: pref.language.clone(),
        download_mb,
    }
}

pub fn is_ready() -> bool {
    WHISPER.is_present() || PARAKEET.is_present()
}

pub fn emit_status(app: &AppHandle) {
    let _ = app.emit("speech-models", status());
    crate::voice::sync(app);
}

/// Downloads what is missing for the default/whisper engine, with progress in `speech-models` events.
#[allow(dead_code)]
pub fn download(app: AppHandle) {
    download_engine(app, "whisper");
}

/// Downloads what is missing for the given engine ("whisper" or "parakeet").
pub fn download_engine(app: AppHandle, engine: &str) {
    let target = if engine == "parakeet" { "parakeet" } else { "whisper" }.to_string();
    {
        let mut d = downloads().lock().unwrap();
        if d.running {
            return;
        }
        d.running = true;
        d.fraction = 0.0;
        d.error = None;
        d.target = target.clone();
    }
    emit_status(&app);
    tauri::async_runtime::spawn(async move {
        let result = download_all(&app, &target).await;
        {
            let mut d = downloads().lock().unwrap();
            d.running = false;
            if let Err(err) = &result {
                d.error = Some(err.clone());
            }
        }
        match result {
            Ok(()) => {
                crate::log::line(format!("{target} speech models downloaded"));
                preload();
            }
            Err(err) => crate::log::line(format!("{target} speech models download failed: {err}")),
        }
        emit_status(&app);
    });
}

async fn download_all(app: &AppHandle, target_engine: &str) -> Result<(), String> {
    let sets: &[&ModelSet] = if target_engine == "parakeet" {
        &[&PARAKEET, &DIARIZER]
    } else {
        &[&WHISPER, &DIARIZER]
    };
    let total: u64 = sets.iter().map(|s| s.total_size()).sum();
    let mut done: u64 = 0;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(60))
        .user_agent(concat!("Nala/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let mut last_emit = Instant::now();
    for set in sets {
        let dir = set.dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        for file in set.files {
            let path = dir.join(file.name);
            if std::fs::metadata(&path).map(|m| m.len() == file.size).unwrap_or(false) {
                done += file.size;
                continue;
            }
            let part = dir.join(format!("{}.part", file.name));
            let mut response = client.get(file.url).send().await.map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                return Err(format!("{} — HTTP {}", file.name, response.status().as_u16()));
            }
            let mut out = std::fs::File::create(&part).map_err(|e| e.to_string())?;
            let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
            let mut got: u64 = 0;
            while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
                out.write_all(&chunk).map_err(|e| e.to_string())?;
                hash.update(&chunk);
                got += chunk.len() as u64;
                if got > file.size {
                    break;
                }
                if last_emit.elapsed() > Duration::from_millis(250) {
                    last_emit = Instant::now();
                    downloads().lock().unwrap().fraction = (done + got) as f64 / total as f64;
                    emit_status(app);
                }
            }
            out.flush().map_err(|e| e.to_string())?;
            drop(out);
            let digest = hash.finish();
            let hex: String = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect();
            if got != file.size || hex != file.sha256 {
                let _ = std::fs::remove_file(&part);
                return Err(format!("{}: the file that came in is not the expected one", file.name));
            }
            std::fs::rename(&part, &path).map_err(|e| e.to_string())?;
            done += file.size;
        }
    }
    Ok(())
}

/// Deletes the models (Settings → Remove the models).
pub fn remove(app: &AppHandle) {
    unload();
    let _ = std::fs::remove_dir_all(PARAKEET.dir());
    let _ = std::fs::remove_dir_all(WHISPER.dir());
    let _ = std::fs::remove_dir_all(DIARIZER.dir());
    downloads().lock().unwrap().error = None;
    emit_status(app);
}

// ── Recognition ───────────────────────────────────────────────────────────────

struct ActiveRecognizer {
    engine: String,
    language: String,
    recognizer: Arc<OfflineRecognizer>,
}

fn recognizer_slot() -> &'static Mutex<Option<ActiveRecognizer>> {
    static R: OnceLock<Mutex<Option<ActiveRecognizer>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(None))
}

/// Serialises loading: two callers never load the model twice.
fn load_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

fn threads() -> i32 {
    std::thread::available_parallelism().map(|n| n.get().clamp(1, 4) as i32).unwrap_or(2)
}

/// The loaded recognizer, loading it on first use (about 2 s).
pub fn recognizer() -> Option<Arc<OfflineRecognizer>> {
    recognizer_with(None, None)
}

/// The loaded recognizer for a given engine and language.
pub fn recognizer_with(engine: Option<&str>, language: Option<&str>) -> Option<Arc<OfflineRecognizer>> {
    let eng = resolve_engine(engine, WHISPER.is_present(), PARAKEET.is_present());
    let whisper_lang = resolve_whisper_language(language, crate::i18n::current());
    let effective_lang = if eng == "whisper" {
        whisper_lang.as_deref().unwrap_or("auto").to_string()
    } else {
        "all".to_string()
    };

    {
        let slot = recognizer_slot().lock().unwrap();
        if let Some(active) = slot.as_ref() {
            if active.engine == eng && active.language == effective_lang {
                return Some(active.recognizer.clone());
            }
        }
    }

    let _guard = load_lock().lock().unwrap();
    {
        let slot = recognizer_slot().lock().unwrap();
        if let Some(active) = slot.as_ref() {
            if active.engine == eng && active.language == effective_lang {
                return Some(active.recognizer.clone());
            }
        }
    }

    let r = if eng == "whisper" {
        if !WHISPER.is_present() {
            return None;
        }
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.whisper = OfflineWhisperModelConfig {
            encoder: Some(WHISPER.path("turbo-encoder.int8.onnx")),
            decoder: Some(WHISPER.path("turbo-decoder.int8.onnx")),
            language: whisper_lang,
            task: Some("transcribe".into()),
            tail_paddings: 0,
            enable_segment_timestamps: true,
            enable_token_timestamps: true,
        };
        config.model_config.tokens = Some(WHISPER.path("turbo-tokens.txt"));
        config.model_config.model_type = Some("whisper".into());
        config.model_config.num_threads = threads();
        let started = Instant::now();
        let rec = OfflineRecognizer::create(&config).map(Arc::new);
        match &rec {
            Some(_) => crate::log::line(format!("Whisper ({effective_lang}) loaded in {:?}", started.elapsed())),
            None => crate::log::line("Whisper could not be loaded"),
        }
        rec
    } else {
        if !PARAKEET.is_present() {
            return None;
        }
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.transducer = OfflineTransducerModelConfig {
            encoder: Some(PARAKEET.path("encoder.int8.onnx")),
            decoder: Some(PARAKEET.path("decoder.int8.onnx")),
            joiner: Some(PARAKEET.path("joiner.int8.onnx")),
        };
        config.model_config.tokens = Some(PARAKEET.path("tokens.txt"));
        config.model_config.model_type = Some("nemo_transducer".into());
        config.model_config.num_threads = threads();
        let started = Instant::now();
        let rec = OfflineRecognizer::create(&config).map(Arc::new);
        match &rec {
            Some(_) => crate::log::line(format!("Parakeet loaded in {:?}", started.elapsed())),
            None => crate::log::line("Parakeet could not be loaded"),
        }
        rec
    };

    if let Some(loaded) = &r {
        *recognizer_slot().lock().unwrap() = Some(ActiveRecognizer {
            engine: eng.to_string(),
            language: effective_lang,
            recognizer: loaded.clone(),
        });
    }
    r
}

/// Loads the model in the background, so the first dictation is not slowed down.
pub fn preload() {
    if !is_ready() {
        return;
    }
    let eng = resolve_engine(None, WHISPER.is_present(), PARAKEET.is_present());
    let whisper_lang = resolve_whisper_language(None, crate::i18n::current());
    let effective_lang = if eng == "whisper" {
        whisper_lang.as_deref().unwrap_or("auto")
    } else {
        "all"
    };
    if recognizer_slot()
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|a| a.engine == eng && a.language == effective_lang)
    {
        return;
    }
    std::thread::spawn(|| {
        let _ = recognizer();
    });
}

/// Frees the model's memory (dictation turned off, models removed).
pub fn unload() {
    *recognizer_slot().lock().unwrap() = None;
}

/// What was said, and each word with its place in the audio.
#[derive(Debug, Clone, Default)]
pub struct Transcript {
    pub text: String,
    pub words: Vec<TimedWord>,
}

/// Longest stretch given to the model at once. Longer audio is cut where it is
/// quietest, near this length.
const MAX_CHUNK_SECONDS: f64 = 28.0;

/// Silence given to the model before and after each stretch, in samples.
///
/// A dictation starts the moment the key goes down: the microphone opens with
/// a little click and the first word is already under way. Half a second of
/// silence first helps model hearing the correct start of speech.
const LEAD_SILENCE: usize = SAMPLE_RATE as usize / 2;
const TAIL_SILENCE: usize = SAMPLE_RATE as usize * 3 / 10;

/// `chunk` between the silences the model is given it with.
pub fn padded(chunk: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(LEAD_SILENCE + chunk.len() + TAIL_SILENCE);
    out.resize(LEAD_SILENCE, 0.0);
    out.extend_from_slice(chunk);
    out.resize(out.len() + TAIL_SILENCE, 0.0);
    out
}

/// Transcribes 16 kHz mono audio using the default or active model.
pub fn transcribe(samples: &[f32]) -> Option<Transcript> {
    transcribe_with(samples, None, None)
}

/// Transcribes 16 kHz mono audio with a specified engine and language.
pub fn transcribe_with(samples: &[f32], engine: Option<&str>, language: Option<&str>) -> Option<Transcript> {
    let rec = recognizer_with(engine, language)?;
    let mut out = Transcript::default();
    for (offset, chunk) in audio::split_for_model(samples, SAMPLE_RATE, MAX_CHUNK_SECONDS) {
        if chunk.len() < (SAMPLE_RATE as usize) / 4 {
            continue;
        }
        let stream = rec.create_stream();
        let audio = padded(chunk);
        stream.accept_waveform(SAMPLE_RATE as i32, &audio);
        rec.decode(&stream);
        let Some(result) = stream.get_result() else { continue };
        // The words' times count from the start of the silence: take it off.
        let start = (offset as f64 - LEAD_SILENCE as f64) / SAMPLE_RATE as f64;
        let mut words = words_from_tokens(&result.tokens, result.timestamps.as_deref().unwrap_or(&[]), start, audio.len() as f64 / SAMPLE_RATE as f64);
        // A word the model places a hair into the silence starts with the audio.
        let from = offset as f64 / SAMPLE_RATE as f64;
        for w in &mut words {
            w.start = w.start.max(from);
            w.end = w.end.max(w.start);
        }
        let text = result.text.trim();
        // If words is empty (e.g. models without token timestamps), approximate from text
        if words.is_empty() && !text.is_empty() {
            let split_words: Vec<&str> = text.split_whitespace().collect();
            let duration = chunk.len() as f64 / SAMPLE_RATE as f64;
            let word_dur = duration / split_words.len().max(1) as f64;
            for (idx, w) in split_words.into_iter().enumerate() {
                let w_start = from + idx as f64 * word_dur;
                let w_end = w_start + word_dur;
                words.push(TimedWord {
                    text: w.to_string(),
                    start: w_start,
                    end: w_end,
                });
            }
        }
        if !text.is_empty() {
            if !out.text.is_empty() {
                out.text.push(' ');
            }
            out.text.push_str(text);
        }
        out.words.extend(words);
    }
    Some(out)
}

/// Tokens (" Ne", " vous", " dem", "ande", "z", "Ġhalo") → words with times.
/// A token starting with a space, '▁' or 'Ġ' starts a word; a word ends where the next
/// token begins (or a moment after its last token, for the last word).
pub fn words_from_tokens(tokens: &[String], timestamps: &[f32], offset: f64, length: f64) -> Vec<TimedWord> {
    let mut out: Vec<TimedWord> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let at = offset + timestamps.get(i).copied().unwrap_or(0.0) as f64;
        let starts_word = token.starts_with(' ') || token.starts_with('▁') || token.starts_with('Ġ') || out.is_empty();
        let piece = token.trim_start_matches([' ', '▁', 'Ġ']);
        if starts_word {
            if let Some(last) = out.last_mut() {
                last.end = last.end.max(at);
            }
            if piece.is_empty() {
                continue;
            }
            out.push(TimedWord { text: piece.to_string(), start: at, end: at + 0.08 });
        } else if let Some(last) = out.last_mut() {
            last.text.push_str(piece);
            last.end = at + 0.08;
        }
    }
    if let Some(last) = out.last_mut() {
        last.end = (last.end + 0.25).min(offset + length).max(last.start);
    }
    out.retain(|w| !w.text.is_empty());
    out
}

// ── Who speaks when ───────────────────────────────────────────────────────────

/// The voices of a recording, as turns "0", "1"… Empty when it can't tell.
pub fn diarize(samples: &[f32]) -> Vec<Turn> {
    if !DIARIZER.is_present() || samples.len() < SAMPLE_RATE as usize * 2 {
        return Vec::new();
    }
    let mut config = OfflineSpeakerDiarizationConfig::default();
    config.segmentation.pyannote.model = Some(DIARIZER.path("segmentation.int8.onnx"));
    config.segmentation.num_threads = threads();
    config.embedding.model = Some(DIARIZER.path("embedding.onnx"));
    config.embedding.num_threads = threads();
    config.clustering.num_clusters = -1;
    // Higher than the default 0.5: a voice is not split in two over a call.
    config.clustering.threshold = 0.7;
    let Some(d) = OfflineSpeakerDiarization::create(&config) else {
        crate::log::line("diarizer could not be loaded");
        return Vec::new();
    };
    let Some(result) = d.process(samples) else { return Vec::new() };
    result
        .sort_by_start_time()
        .into_iter()
        .map(|s| Turn { speaker: s.speaker.to_string(), start: s.start as f64, end: s.end as f64 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn tokens_make_words_with_their_times() {
        let words = words_from_tokens(
            &toks(&[" Ne", " vous", " dem", "ande", "z", " pas", "."]),
            &[0.0, 0.16, 0.32, 0.4, 0.56, 0.72, 0.8],
            10.0,
            2.0,
        );
        let texts: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(texts, ["Ne", "vous", "demandez", "pas."]);
        assert!((words[2].start - 10.32).abs() < 1e-6);
        assert!(words[2].end >= 10.72 - 1e-9);
        assert!(words[3].end <= 12.0);
    }

    #[test]
    fn the_downloads_add_up() {
        assert_eq!(PARAKEET.total_size(), 652_184_281 + 11_845_275 + 6_355_277 + 93_939);
        assert_eq!(WHISPER.total_size(), 674_716_297 + 361_080_764 + 816_730);
        assert!(DIARIZER.total_size() < 40_000_000);
        for set in [&PARAKEET, &WHISPER, &DIARIZER] {
            for f in set.files {
                assert_eq!(f.sha256.len(), 64, "{}", f.name);
                assert!(f.url.starts_with("https://huggingface.co/"), "{}", f.name);
            }
        }
    }

    #[test]
    fn whisper_language_follows_preference_and_ui_fallback() {
        // Auto mode pins "id" when the UI language is Indonesian, and uses Whisper native
        // language auto-detection (None) for other UI languages.
        assert_eq!(resolve_whisper_language(Some("auto"), "id").as_deref(), Some("id"));
        assert_eq!(resolve_whisper_language(Some("auto"), "id-ID").as_deref(), Some("id"));
        assert_eq!(resolve_whisper_language(Some(""), "id").as_deref(), Some("id"));
        assert_eq!(resolve_whisper_language(Some("auto"), "en"), None);
        assert_eq!(resolve_whisper_language(Some("auto"), "fr"), None);

        // Explicit language selection overrides UI language and normalizes regional tags.
        assert_eq!(resolve_whisper_language(Some("id"), "en").as_deref(), Some("id"));
        assert_eq!(resolve_whisper_language(Some("en"), "id").as_deref(), Some("en"));
        assert_eq!(resolve_whisper_language(Some("zh-Hans"), "en").as_deref(), Some("zh"));
        assert_eq!(resolve_whisper_language(Some("pt-BR"), "id").as_deref(), Some("pt"));
    }

    #[test]
    fn engine_resolution_falls_back_to_installed_model() {
        assert_eq!(resolve_engine(Some("whisper"), true, false), "whisper");
        assert_eq!(resolve_engine(Some("whisper"), false, true), "parakeet");
        assert_eq!(resolve_engine(Some("parakeet"), false, true), "parakeet");
        assert_eq!(resolve_engine(Some("parakeet"), true, false), "whisper");
        assert_eq!(resolve_engine(Some("whisper"), false, false), "whisper");
        assert_eq!(resolve_engine(Some("parakeet"), false, false), "parakeet");
    }
}

/// With the models downloaded: `COUCOU_TEST_WAV=… cargo test -- --ignored speech`.
#[cfg(test)]
mod with_models {
    #[test]
    fn the_model_hears_silence_around_the_words() {
        let chunk = vec![0.5f32; 1000];
        let audio = super::padded(&chunk);
        assert_eq!(audio.len(), super::LEAD_SILENCE + 1000 + super::TAIL_SILENCE);
        assert!(audio[..super::LEAD_SILENCE].iter().all(|&s| s == 0.0));
        assert_eq!(&audio[super::LEAD_SILENCE..super::LEAD_SILENCE + 1000], &chunk[..]);
        assert!(audio[super::LEAD_SILENCE + 1000..].iter().all(|&s| s == 0.0));
    }

    /// A dictation that starts with the microphone's click and the first word
    /// already under way (a French one, from `COUCOU_SUDDEN_WAV`, 16 kHz):
    /// Parakeet used to write half of it in English.
    /// `cargo test -p coucou --lib a_sudden_start -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn a_sudden_start_keeps_its_language() {
        let path = std::env::var("COUCOU_SUDDEN_WAV").expect("COUCOU_SUDDEN_WAV");
        let samples = super::audio::read_wav(std::path::Path::new(&path)).unwrap();
        let t = super::transcribe(&samples).expect("models");
        println!("{}", t.text);
        let english = ["that", "is", "for", "it", "when", "the"];
        let words: Vec<String> = t.text.split(|c: char| !c.is_alphabetic()).map(str::to_lowercase).collect();
        assert!(!words.iter().any(|w| english.contains(&w.as_str())), "{}", t.text);
        assert!(t.words.first().is_some_and(|w| w.start >= 0.0));
    }

    #[test]
    #[ignore]
    fn transcribes_a_recording() {
        let path = std::env::var("COUCOU_TEST_WAV").expect("COUCOU_TEST_WAV");
        let bytes = std::fs::read(&path).unwrap();
        let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let pcm: Vec<f32> = bytes[44..].chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).collect();
        let mut r = super::audio::Resampler::new(rate);
        let mut samples = Vec::new();
        r.process(&pcm, &mut samples);
        let t = super::transcribe(&samples).expect("models");
        println!("{}\n{:?}", t.text, t.words.iter().map(|w| (&w.text, (w.start * 100.0).round() / 100.0)).collect::<Vec<_>>());
        assert!(!t.text.is_empty());
        let turns = super::diarize(&samples);
        println!("{turns:?}");
    }
}
