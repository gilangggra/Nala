// « OK Nala »: hearing the wake phrase in what the speech model wrote, and
// telling speech from silence on the microphone. Nothing here touches the OS.

use crate::speech::config;

/// Lower case, accents gone, everything but letters and digits a space.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        let base = match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ÿ' | 'ý' => 'y',
            'ñ' => 'n',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        };
        out.push(base);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// How Parakeet may write "OK".
fn is_ok(word: &str) -> bool {
    matches!(word, "ok" | "okay" | "oke" | "okey" | "hey" | "hi" | "hai" | "halo" | "hello")
}

/// How Parakeet may write "coucou", in one word. Said with a Brazilian
/// Portuguese accent it comes out as "cocku" (#813), with a Spanish accent as
/// "coco" (the Mac's spotter takes "coco" too); the others are the close spellings
/// of those accents. Not "cool": "okay, cool" is too common to wake on, nor
/// "gogo": "okay, go go" is something people say.
fn is_coucou(word: &str) -> bool {
    matches!(
        word,
        "coucou"
            | "coucous"
            | "cuckoo"
            | "cucu"
            | "kuku"
            | "koukou"
            | "cookoo"
            | "coocoo"
            | "cocou"
            | "kukuu"
            | "couscous"
            | "cocku"
            | "cocu"
            | "cuco"
            | "cuku"
            | "coku"
            | "kuko"
            | "coco"
    )
}

/// How Parakeet may write "nala", in one word (retained for backward compatibility).
fn is_nala(word: &str) -> bool {
    matches!(
        word,
        "nala"
            | "nahla"
            | "nalla"
            | "nalah"
            | "nallah"
            | "nalas"
            | "mala"
            | "malla"
    )
}

/// How Whisper/Parakeet may write "aira", in one word.
fn is_aira(word: &str) -> bool {
    matches!(
        word,
        "aira"
            | "ayra"
            | "eyra"
            | "aera"
            | "airah"
            | "ayrah"
            | "eyrah"
            | "ira"
    )
}

/// Matches target name (Aira, Nala, or Coucou).
fn is_wake_name(word: &str) -> bool {
    is_aira(word) || is_nala(word) || is_coucou(word)
}

/// …or in two.
fn is_coucou_pair(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("cou", "cou") | ("coo", "coo") | ("ku", "ku") | ("cu", "cu") | ("koo", "koo") | ("cuck", "oo") | ("co", "co")
    )
}

fn is_nala_pair(a: &str, b: &str) -> bool {
    matches!((a, b), ("na", "la") | ("nah", "la"))
}

fn is_aira_pair(a: &str, b: &str) -> bool {
    matches!((a, b), ("ai", "ra") | ("ay", "ra"))
}

fn is_wake_pair(a: &str, b: &str) -> bool {
    is_aira_pair(a, b) || is_nala_pair(a, b) || is_coucou_pair(a, b)
}

/// When `text` holds the wake phrase, what was said after it (maybe nothing).
/// The words keep their case and punctuation, as the model wrote them.
pub fn after_wake(text: &str) -> Option<String> {
    if crate::speech::is_hallucination(text) {
        return None;
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    let norm: Vec<String> = words.iter().map(|w| normalize(w)).collect();
    // "o.k." and "o k" become two tokens: glue them.
    let mut i = 0;
    while i < norm.len() {
        let mut ok_end = None;
        if is_ok(&norm[i]) {
            ok_end = Some(i + 1);
        } else if norm[i] == "o k" || (norm[i] == "o" && norm.get(i + 1).map(String::as_str) == Some("k")) {
            ok_end = Some(if norm[i] == "o" { i + 2 } else { i + 1 });
        }
        if let Some(j) = ok_end {
            // A word made of two ("ok coucou" / "ok nala" / "ok aira" written as one token is rare but happens).
            if let Some(w) = norm.get(j) {
                let parts: Vec<&str> = w.split(' ').collect();
                if parts.len() == 1 && is_wake_name(parts[0]) {
                    return Some(words[j + 1..].join(" ").trim_start_matches([',', '.', '!', ' ']).trim().to_string());
                }
                if let Some(next) = norm.get(j + 1) {
                    if is_wake_pair(w, next) {
                        return Some(words[j + 2..].join(" ").trim_start_matches([',', '.', '!', ' ']).trim().to_string());
                    }
                }
            }
        }
        // "okcoucou" / "oknala" / "okaira" in one piece.
        if let Some(rest) = norm[i]
            .strip_prefix("okay")
            .or_else(|| norm[i].strip_prefix("ok"))
            .or_else(|| norm[i].strip_prefix("hey"))
            .or_else(|| norm[i].strip_prefix("hai"))
            .or_else(|| norm[i].strip_prefix("halo"))
        {
            if is_wake_name(rest.trim()) {
                return Some(words[i + 1..].join(" ").trim().to_string());
            }
        }
        // Direct invocation at sentence start, e.g. "Aira, play music" or "Aira! What's the time"
        if i == 0 && words.len() > 1 && (is_aira(&norm[0]) || is_nala(&norm[0])) {
            return Some(words[1..].join(" ").trim_start_matches([',', '.', '!', ' ']).trim().to_string());
        }
        i += 1;
    }
    None
}

/// Speech or silence, 30 ms at a time, against a noise floor that follows the
/// room. A stretch of speech is handed out once followed by `hangover` frames
/// of silence, or when it gets too long.
pub struct Vad {
    floor: f32,
    in_speech: bool,
    speech_frames: usize,
    silent_frames: usize,
    frame: Vec<f32>,
    /// The current stretch, with a little audio from before it began.
    current: Vec<f32>,
    preroll: std::collections::VecDeque<f32>,
    /// Silence that ends a stretch, in frames.
    pub hangover: usize,
}

pub const FRAME: usize = config::VAD_FRAME_SAMPLES; // 30 ms at 16 kHz
const PREROLL: usize = config::VAD_PREROLL_SAMPLES;
const MAX_SEGMENT: usize = config::VAD_MAX_SEGMENT_SAMPLES;

impl Default for Vad {
    fn default() -> Self {
        Self {
            floor: config::VAD_INITIAL_FLOOR,
            in_speech: false,
            speech_frames: 0,
            silent_frames: 0,
            frame: Vec::with_capacity(FRAME),
            current: Vec::new(),
            preroll: std::collections::VecDeque::with_capacity(PREROLL),
            hangover: config::VAD_DEFAULT_HANGOVER_FRAMES,
        }
    }
}

impl Vad {
    #[cfg(test)]
    pub fn in_speech(&self) -> bool {
        self.in_speech
    }

    /// Feeds 16 kHz samples; the stretches of speech that ended.
    pub fn feed(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        let mut done = Vec::new();
        for &s in samples {
            self.frame.push(s);
            if self.frame.len() < FRAME {
                continue;
            }
            let rms = frame_rms(&self.frame);
            let speech = rms > (self.floor * config::VAD_FLOOR_MULTIPLIER).max(config::VAD_MIN_SPEECH_RMS);
            if !speech && !self.in_speech {
                self.floor = self.floor * config::VAD_FLOOR_DECAY + rms * config::VAD_FLOOR_ATTACK;
            }
            if self.in_speech {
                self.current.extend_from_slice(&self.frame);
                if speech {
                    self.silent_frames = 0;
                } else {
                    self.silent_frames += 1;
                }
                if self.silent_frames >= self.hangover || self.current.len() >= MAX_SEGMENT {
                    done.push(std::mem::take(&mut self.current));
                    self.in_speech = false;
                    self.speech_frames = 0;
                    self.silent_frames = 0;
                }
            } else if speech {
                self.speech_frames += 1;
                if self.speech_frames >= config::VAD_ONSET_FRAMES {
                    self.in_speech = true;
                    self.current = self.preroll.drain(..).collect();
                    self.current.extend_from_slice(&self.frame);
                }
            } else {
                self.speech_frames = 0;
            }
            if !self.in_speech {
                for &x in &self.frame {
                    if self.preroll.len() == PREROLL {
                        self.preroll.pop_front();
                    }
                    self.preroll.push_back(x);
                }
            }
            self.frame.clear();
        }
        done
    }

    /// Forgets what is being heard (Nala speaks, or dictation starts).
    pub fn reset(&mut self) {
        self.in_speech = false;
        self.speech_frames = 0;
        self.silent_frames = 0;
        self.current.clear();
        self.preroll.clear();
        self.frame.clear();
    }
}

fn frame_rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt()
}

/// Fast pre-inference gate for the « OK Nala » loop.
///
/// Because `Vad::feed` includes 300 ms of pre-roll and 750–1200 ms of silent
/// hangover in every emitted segment, a 90 ms keyboard click or desk tap
/// produces a segment > 1.1 s long. Counting the actual voiced frames (`rms >=
/// VAD_MIN_SPEECH_RMS`) and peak amplitude rejects brief transients before the
/// ~1 GB Whisper model is ever invoked.
pub fn is_plausible_speech(samples: &[f32], wake_mode: bool) -> bool {
    if samples.len() < config::WAKE_MIN_SEGMENT_SAMPLES {
        return false;
    }
    let min_frames = if wake_mode {
        config::WAKE_MIN_ACTIVE_FRAMES
    } else {
        config::COMMAND_MIN_ACTIVE_FRAMES
    };
    let min_peak = if wake_mode {
        config::WAKE_MIN_PEAK
    } else {
        config::COMMAND_MIN_PEAK
    };
    let mut active_frames = 0usize;
    let mut peak = 0.0f32;
    for chunk in samples.chunks(FRAME) {
        if chunk.len() < FRAME / 2 {
            continue;
        }
        for &s in chunk {
            let abs = s.abs();
            if abs > peak {
                peak = abs;
            }
        }
        if frame_rms(chunk) >= config::VAD_MIN_SPEECH_RMS {
            active_frames += 1;
        }
    }
    active_frames >= min_frames && peak >= min_peak
}

/// Trims leading and trailing silent frames from a VAD segment, keeping up to
/// `VAD_TRIM_MARGIN_FRAMES` (90 ms) of natural margin on each side so the
/// recognizer does not decode a full second of trailing hangover silence.
pub fn trim_silence(samples: &[f32]) -> &[f32] {
    if samples.is_empty() {
        return &[];
    }
    let total_frames = samples.len().div_ceil(FRAME);
    let mut first_voiced = None;
    let mut last_voiced = None;
    for idx in 0..total_frames {
        let start = idx * FRAME;
        let end = (start + FRAME).min(samples.len());
        if frame_rms(&samples[start..end]) >= config::VAD_MIN_SPEECH_RMS {
            if first_voiced.is_none() {
                first_voiced = Some(idx);
            }
            last_voiced = Some(idx);
        }
    }
    let (Some(first), Some(last)) = (first_voiced, last_voiced) else {
        return &[];
    };
    let start_frame = first.saturating_sub(config::VAD_TRIM_MARGIN_FRAMES);
    let end_frame = (last + 1 + config::VAD_TRIM_MARGIN_FRAMES).min(total_frames);
    let start_sample = (start_frame * FRAME).min(samples.len());
    let end_sample = (end_frame * FRAME).min(samples.len());
    if start_sample >= end_sample {
        return &[];
    }
    &samples[start_sample..end_sample]
}

/// "fr-FR" for a French answer, "id-ID" for an Indonesian answer, else None (the system's language).
pub fn speech_lang(text: &str) -> Option<&'static str> {
    let n = normalize(text);
    let words: Vec<&str> = n.split(' ').collect();
    let french = ["le", "la", "les", "est", "tu", "vous", "je", "un", "une", "des", "pour", "pas", "c", "et", "sur", "avec", "ton", "ta"];
    let count = words.iter().filter(|w| french.contains(w)).count();
    if count * 6 >= words.len().max(1) && count >= 1 {
        return Some("fr-FR");
    }
    let indonesian = [
        "yang", "dan", "di", "ini", "itu", "untuk", "saya", "kamu", "bisa", "sudah",
        "tidak", "ada", "dengan", "akan", "dari", "ke", "pada", "oleh", "juga", "hanya",
        "tentang", "atau", "baik", "lagu", "hari", "catatan", "telah", "anda"
    ];
    let count_id = words.iter().filter(|w| indonesian.contains(w)).count();
    if count_id * 6 >= words.len().max(1) && count_id >= 1 {
        return Some("id-ID");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wake_phrase_however_parakeet_writes_it() {
        assert_eq!(after_wake("OK Coucou, play some music.").as_deref(), Some("play some music."));
        assert_eq!(after_wake("Okay, coucou! Quelle heure est-il ?").as_deref(), Some("Quelle heure est-il ?"));
        assert_eq!(after_wake("O.K. Cuckoo what's the weather").as_deref(), Some("what's the weather"));
        assert_eq!(after_wake("ok cou cou ajoute Stripe").as_deref(), Some("ajoute Stripe"));
        assert_eq!(after_wake("OK Coucou.").as_deref(), Some(""));
        assert_eq!(after_wake("Hey coucou, ça va ?").as_deref(), Some("ça va ?"));
        // A Brazilian speaker's « OK Coucou », as Parakeet wrote it (#813).
        assert_eq!(after_wake("Okay, cocku.").as_deref(), Some(""));
        assert_eq!(after_wake("Ok, cocku, que horas são?").as_deref(), Some("que horas são?"));
        assert_eq!(after_wake("Okay, cuco.").as_deref(), Some(""));
        // "cool" stays out: it is what people say, not the wake phrase.
        assert_eq!(after_wake("Okay, cool."), None);
        assert_eq!(after_wake("Okay, cool cool."), None);
        // A Spanish speaker's « OK Coucou », as Parakeet wrote it.
        assert_eq!(after_wake("Okay, coco.").as_deref(), Some(""));
        assert_eq!(after_wake("Okay, Co Co, ¿qué hora es?").as_deref(), Some("¿qué hora es?"));
        // "go go" stays out, and another assistant's wake phrase never wakes Coucou.
        assert_eq!(after_wake("Okay, go go."), None);
        assert_eq!(after_wake("Okay, Google."), None);
        assert_eq!(after_wake("on a dit coucou à Paul"), None);
        assert_eq!(after_wake("ok let's go"), None);
        // Aira wake phrases
        assert_eq!(after_wake("OK Aira, play some music.").as_deref(), Some("play some music."));
        assert_eq!(after_wake("Hai Aira, apa kabar?").as_deref(), Some("apa kabar?"));
        assert_eq!(after_wake("Hey Aira, buka VS Code.").as_deref(), Some("buka VS Code."));
        assert_eq!(after_wake("Okay, Aira!").as_deref(), Some(""));
        assert_eq!(after_wake("O.K. Ayra what's the weather").as_deref(), Some("what's the weather"));
        assert_eq!(after_wake("ok ai ra buka Spotify").as_deref(), Some("buka Spotify"));
        assert_eq!(after_wake("Aira, play some jazz.").as_deref(), Some("play some jazz."));
        assert_eq!(after_wake("Aira! Bukakan file laporan.").as_deref(), Some("Bukakan file laporan."));
        // Legacy Nala wake phrases (supported during transition)
        assert_eq!(after_wake("OK Nala, play some music.").as_deref(), Some("play some music."));
        assert_eq!(after_wake("Hey Nala, apa kabar?").as_deref(), Some("apa kabar?"));
        assert_eq!(after_wake("Okay, Nala!").as_deref(), Some(""));
        assert_eq!(after_wake("O.K. Nahla what's the weather").as_deref(), Some("what's the weather"));
        assert_eq!(after_wake("ok na la buka VS Code").as_deref(), Some("buka VS Code"));
        assert_eq!(after_wake("Nala, play some jazz.").as_deref(), Some("play some jazz."));
        assert_eq!(after_wake("Nala! Bukakan file laporan.").as_deref(), Some("Bukakan file laporan."));
        assert_eq!(normalize("Ça VA, Élodie ?"), "ca va elodie");
    }

    #[test]
    fn speech_is_cut_after_a_pause() {
        let mut vad = Vad::default();
        let silence = vec![0.0f32; 16_000];
        let tone: Vec<f32> = (0..16_000).map(|i| (i as f32 * 0.1).sin() * 0.3).collect();
        assert!(vad.feed(&silence).is_empty());
        assert!(vad.feed(&tone).is_empty());
        assert!(vad.in_speech());
        let out = vad.feed(&silence);
        assert_eq!(out.len(), 1);
        // The tone, a pre-roll before it and the hangover after it.
        assert!(out[0].len() > 16_000 && out[0].len() <= 16_000 + 25 * FRAME + PREROLL + 2 * FRAME);
        assert!(!vad.in_speech());
    }

    #[test]
    fn french_answers_are_spoken_in_french() {
        assert_eq!(speech_lang("C'est fait, j'ai ajouté Stripe pour toi."), Some("fr-FR"));
        assert_eq!(speech_lang("Done, Stripe is in the island."), None);
    }

    #[test]
    fn indonesian_answers_are_spoken_in_indonesian() {
        assert_eq!(speech_lang("Sudah saya tambahkan catatan untuk kamu."), Some("id-ID"));
        assert_eq!(speech_lang("Baik, musik sedang diputar di Spotify."), Some("id-ID"));
    }

    #[test]
    fn short_clicks_and_transients_are_rejected_before_inference() {
        let mut vad = Vad::default();
        let silence = vec![0.0f32; 16_000];
        // A 90 ms keyboard click (3 frames of 480 samples) followed by silence:
        // Vad emits a segment (> 1.1 s including preroll + hangover), but
        // `is_plausible_speech` rejects it in Wake mode because only 3 frames are active.
        let click: Vec<f32> = (0..(3 * FRAME)).map(|i| (i as f32 * 0.2).sin() * 0.1).collect();
        let _ = vad.feed(&silence);
        let _ = vad.feed(&click);
        let segments = vad.feed(&silence);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].len() > config::WAKE_MIN_SEGMENT_SAMPLES);
        assert!(!is_plausible_speech(&segments[0], true));

        // A real wake phrase stretch (~600 ms of voiced speech = 20 frames) passes both Wake and Command gates,
        // and `trim_silence` removes the long trailing hangover silence.
        let speech_stretch: Vec<f32> = (0..(20 * FRAME)).map(|i| (i as f32 * 0.1).sin() * 0.25).collect();
        let _ = vad.feed(&speech_stretch);
        let voiced_segments = vad.feed(&silence);
        assert_eq!(voiced_segments.len(), 1);
        assert!(is_plausible_speech(&voiced_segments[0], true));
        assert!(is_plausible_speech(&voiced_segments[0], false));

        let trimmed = trim_silence(&voiced_segments[0]);
        assert!(!trimmed.is_empty());
        assert!(trimmed.len() < voiced_segments[0].len());
        assert_eq!(trim_silence(&silence).len(), 0);
    }

    #[test]
    fn hallucinated_repetitions_do_not_trigger_wake() {
        assert_eq!(after_wake("OK Aira ya ya ya ya"), None);
        assert_eq!(after_wake("OK Nala ya ya ya ya"), None);
        assert_eq!(after_wake("[Musik]"), None);
        assert_eq!(after_wake("Terima kasih telah menonton"), None);
    }
}
