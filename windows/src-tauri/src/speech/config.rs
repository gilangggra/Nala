// Thresholds and tuning constants for offline speech recognition, voice activity
// detection (VAD), wake-word gating, and Whisper anti-hallucination filtering.
// Centralised here so neither `speech/` nor `voice/` hardcodes magic numbers.

/// Internal audio processing sample rate (16 kHz mono).
pub const SAMPLE_RATE: u32 = 16_000;

/// Maximum number of CPU threads allocated to ONNX Runtime inference.
pub const MAX_RECOGNIZER_THREADS: usize = 4;

// ── Chunking & Padding ────────────────────────────────────────────────────────

/// Longest stretch of audio decoded in a single pass (seconds).
pub const MAX_CHUNK_SECONDS: f64 = 28.0;

/// Minimum chunk length sent to the recognizer (250 ms at 16 kHz).
pub const MIN_CHUNK_SAMPLES: usize = (SAMPLE_RATE as usize) / 4;

/// Leading silence prepended before each chunk (500 ms) so the model cleanly
/// locks onto the first phoneme after a key press or VAD cut.
pub const LEAD_SILENCE_SAMPLES: usize = (SAMPLE_RATE as usize) / 2;

/// Trailing silence appended after each chunk (300 ms).
pub const TAIL_SILENCE_SAMPLES: usize = (SAMPLE_RATE as usize) * 3 / 10;

/// Minimum peak amplitude required for a chunk to be decoded (rejects dead silence).
pub const MIN_TRANSCRIBE_PEAK: f32 = 0.004;

/// Minimum RMS energy required for a chunk to be decoded (rejects low-level hum).
pub const MIN_TRANSCRIBE_RMS: f32 = 0.0012;

// ── Speaker Diarization ───────────────────────────────────────────────────────

/// Minimum audio length required to run speaker diarization (2 seconds).
pub const DIARIZER_MIN_SAMPLES: usize = (SAMPLE_RATE as usize) * 2;

/// Agglomerative clustering threshold for speaker embeddings.
pub const DIARIZER_CLUSTER_THRESHOLD: f32 = 0.7;

// ── Voice Activity Detection (VAD) & Wake Gating ──────────────────────────────

/// VAD frame size: 30 ms at 16 kHz (480 samples).
pub const VAD_FRAME_SAMPLES: usize = 480;

/// Pre-roll buffer kept before speech onset (300 ms = 4,800 samples).
pub const VAD_PREROLL_SAMPLES: usize = (SAMPLE_RATE as usize) * 3 / 10;

/// Maximum duration of a single continuous VAD segment (12 seconds).
pub const VAD_MAX_SEGMENT_SAMPLES: usize = (SAMPLE_RATE as usize) * 12;

/// Initial adaptive noise floor RMS estimate.
pub const VAD_INITIAL_FLOOR: f32 = 0.004;

/// Multiplier over the adaptive noise floor required to classify a frame as speech.
pub const VAD_FLOOR_MULTIPLIER: f32 = 3.0;

/// Absolute minimum RMS threshold for a frame to count as speech.
pub const VAD_MIN_SPEECH_RMS: f32 = 0.012;

/// Exponential moving average retention factor for the noise floor during silence.
pub const VAD_FLOOR_DECAY: f32 = 0.97;

/// Exponential moving average update weight for the noise floor during silence.
pub const VAD_FLOOR_ATTACK: f32 = 0.03;

/// Consecutive voiced frames required to trigger speech state (3 frames = 90 ms).
pub const VAD_ONSET_FRAMES: usize = 3;

/// Silent frames that end a speech stretch in Wake mode (25 frames = 750 ms).
pub const VAD_DEFAULT_HANGOVER_FRAMES: usize = 25;

/// Silent frames that end a speech stretch in Command / FollowUp mode (40 frames = 1.2 s).
pub const VAD_COMMAND_HANGOVER_FRAMES: usize = 40;

/// Margin of silent frames retained on each side when trimming a VAD segment (3 frames = 90 ms).
pub const VAD_TRIM_MARGIN_FRAMES: usize = 3;

/// Minimum total segment length in Wake/Command loop (350 ms).
pub const WAKE_MIN_SEGMENT_SAMPLES: usize = (SAMPLE_RATE as usize) * 35 / 100;

/// Minimum number of active voiced frames (30 ms each) required in Wake mode
/// before invoking the speech model (6 frames = 180 ms of actual voiced audio).
/// Filters out short keyboard clicks, mouse taps, and coughs.
pub const WAKE_MIN_ACTIVE_FRAMES: usize = 6;

/// Minimum number of active voiced frames required in Command / FollowUp mode
/// (4 frames = 120 ms) so short replies like "Ya", "No", or "Batal" are accepted.
pub const COMMAND_MIN_ACTIVE_FRAMES: usize = 4;

/// Minimum peak sample amplitude required for a Wake segment to run inference.
pub const WAKE_MIN_PEAK: f32 = 0.018;

/// Minimum peak sample amplitude required for a Command / FollowUp segment.
pub const COMMAND_MIN_PEAK: f32 = 0.012;

/// Seconds to wait for the command after hearing « OK Nala » alone.
pub const COMMAND_TIMEOUT_SECS: u64 = 7;

/// Seconds to wait for a follow-up answer after Nala asks a question.
pub const FOLLOWUP_TIMEOUT_SECS: u64 = 8;

// ── Whisper Anti-Hallucination Filter ─────────────────────────────────────────

/// Maximum consecutive identical normalized words allowed before treating a
/// segment as a Whisper repetition loop (e.g. "ya ya ya ya" -> 4).
pub const HALLUCINATION_MAX_WORD_REPEATS: usize = 4;

/// Maximum consecutive identical 2- or 3-word phrases allowed (e.g.
/// "terima kasih terima kasih terima kasih" -> 3).
pub const HALLUCINATION_MAX_NGRAM_REPEATS: usize = 3;

/// Minimum word count before checking global unique-word diversity ratio.
pub const HALLUCINATION_DIVERSITY_MIN_WORDS: usize = 8;

/// Minimum ratio of unique words (`unique * DEN >= total * NUM`, i.e. 1/4 = 25%).
pub const HALLUCINATION_UNIQUE_RATIO_NUM: usize = 1;
pub const HALLUCINATION_UNIQUE_RATIO_DEN: usize = 4;

/// Non-speech bracketed/parenthetical markers that Whisper emits on noise or music
/// (matched case-insensitively after stripping surrounding brackets/parentheses/stars).
pub const HALLUCINATION_TAGS: &[&str] = &[
    "blank_audio",
    "blank audio",
    "musik",
    "music",
    "applause",
    "tepuk tangan",
    "laughter",
    "tertawa",
    "tawa",
    "silence",
    "hening",
    "sepi",
    "noise",
    "derau",
    "suara bising",
    "inaudible",
    "tidak terdengar",
    "cough",
    "batuk",
    "sigh",
    "hembusan napas",
];

/// Full-utterance phrases that Whisper commonly hallucinates on near-silent or
/// noisy segments (trained on YouTube subtitle outros). Matched against the
/// normalized full transcript only, so normal "terima kasih" or "thank you"
/// in conversation is never blocked.
pub const HALLUCINATION_PHRASES: &[&str] = &[
    "terima kasih telah menonton",
    "terima kasih sudah menonton",
    "terima kasih telah menyaksikan",
    "jangan lupa like comment dan subscribe",
    "jangan lupa subscribe",
    "sampai jumpa di video berikutnya",
    "subtitle oleh",
    "diterjemahkan oleh",
    "thank you for watching",
    "thanks for watching",
    "thank you for listening",
    "please subscribe",
    "like and subscribe",
    "subtitles by the amara org community",
    "subtitles by amara org",
    "amara org",
    "transcribed by",
    "sous titres realises par la communaute d amara org",
    "merci d avoir regarde",
    "merci d avoir regarde cette video",
    "abonnez vous",
];

// ── Local Voice Intent Parser (« OK Nala » Fast Path) ─────────────────────────

/// Minimum allowed media/music volume percentage.
pub const VOICE_VOLUME_MIN: u8 = 0;

/// Maximum allowed media/music volume percentage.
pub const VOICE_VOLUME_MAX: u8 = 100;

/// Target volume percentage when the user says "besarkan suara" / "volume up"
/// without specifying an exact number.
pub const VOICE_VOLUME_STEP_UP: u8 = 75;

/// Target volume percentage when the user says "kecilkan suara" / "volume down"
/// without specifying an exact number.
pub const VOICE_VOLUME_STEP_DOWN: u8 = 25;

/// Maximum character length of a target argument (app name, pill name, or song
/// query) accepted by the deterministic local intent parser. Longer utterances
/// are routed to the conversational AI brain.
pub const VOICE_MAX_ARG_CHARS: usize = 120;

/// Maximum word count for an application name in a local "open <app>" command.
/// Prevents conversational sentences starting with "buka..." / "open..." from
/// being misclassified as application launches.
pub const VOICE_MAX_APP_WORDS: usize = 5;

/// Maximum word count for a single pill name in a local pill command.
pub const VOICE_MAX_PILL_WORDS: usize = 4;

// ── Multi-Provider Voice Brain (« OK Nala » Conversational Adapter) ───────────

/// Default low-latency Anthropic model for « OK Nala ».
pub const VOICE_ANTHROPIC_FAST_MODEL: &str = "claude-haiku-4-5";

/// Default low-latency OpenAI model for « OK Nala ».
pub const VOICE_OPENAI_FAST_MODEL: &str = "gpt-4o-mini";

/// Default low-latency Google Gemini model for « OK Nala ».
pub const VOICE_GOOGLE_FAST_MODEL: &str = "gemini-2.0-flash";

/// Default OpenRouter model for « OK Nala ».
pub const VOICE_OPENROUTER_FAST_MODEL: &str = "openrouter/auto";

/// Seconds of inactivity before « OK Nala » forgets the in-memory voice conversation.
pub const VOICE_BRAIN_FORGET_SECS: u64 = 90;

/// Maximum tool-execution turns per single voice question.
pub const VOICE_BRAIN_MAX_STEPS: usize = 5;

/// Maximum messages retained in the voice brain's rolling history.
pub const VOICE_BRAIN_MAX_HISTORY: usize = 24;

/// Maximum tokens requested per voice brain turn.
pub const VOICE_BRAIN_MAX_TOKENS: u32 = 1024;

/// HTTP timeout (seconds) for a single voice brain completion request.
pub const VOICE_BRAIN_TIMEOUT_SECS: u64 = 45;



