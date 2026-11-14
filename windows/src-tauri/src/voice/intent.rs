// Deterministic local intent parser for « OK Aira » (Tahap 1 — Fast Path).
//
// Everyday voice commands (music playback, volume control, opening local apps,
// managing island pills, and cancelling a voice prompt) are parsed and executed
// in <1 ms on-device without requiring a paid Claude API key or network round-trip.
// Unrecognised or open-ended questions return `None` so the caller can fall back
// to the conversational AI brain (`brain.rs`).

use serde_json::{json, Value};
use tauri::AppHandle;

use crate::speech::config::{
    VOICE_MAX_APP_WORDS, VOICE_MAX_ARG_CHARS, VOICE_MAX_PILL_WORDS, VOICE_VOLUME_MAX,
    VOICE_VOLUME_MIN, VOICE_VOLUME_STEP_DOWN, VOICE_VOLUME_STEP_UP,
};

use super::brain::Reply;
use super::wake;

/// Structured local command recognized without an LLM call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalIntent {
    /// Cancel / dismiss the current voice prompt and return to wake listening.
    Cancel,
    /// Toggle music play / pause.
    MusicPlayPause,
    /// Skip to the next track.
    MusicNext,
    /// Return to the previous track.
    MusicPrevious,
    /// Set media/Spotify volume to a clamped percentage (0..=100).
    MusicVolume(u8),
    /// Search and play a track/artist in Spotify.
    MusicSearch(String),
    /// Launch a local application by name.
    OpenApp(String),
    /// Add one or more pills to the island.
    PillAdd(Vec<String>),
    /// Remove one or more pills from the island.
    PillRemove(Vec<String>),
    /// Set the primary workspace pill on the island.
    PillSetMain(String),
}

/// Spoken politeness prefixes stripped before matching command patterns.
const POLITENESS_PREFIXES: &[&str] = &[
    "tolongin",
    "tolong",
    "coba",
    "bisa tolong",
    "bisakah kamu",
    "bisakah",
    "bisa",
    "minta tolong",
    "mohon",
    "silakan",
    "can you please",
    "could you please",
    "would you please",
    "can you",
    "could you",
    "would you",
    "please",
    "est ce que tu peux",
    "tu peux",
    "peux tu",
    "pourrais tu",
    "s il te plait",
    "s il vous plait",
    "svp",
];

/// Trailing courtesy particles stripped from command arguments.
const TRAILING_PARTICLES: &[&str] = &[
    "s il te plait",
    "s il vous plait",
    "terima kasih",
    "makasih",
    "thank you",
    "thanks",
    "please",
    "sekarang",
    "maintenant",
    "now",
    "dong",
    "deh",
    "ya",
    "yuk",
    "svp",
];

/// Exact normalized utterances that cancel the active voice turn.
const CANCEL_PHRASES: &[&str] = &[
    "batal",
    "batalkan",
    "tidak jadi",
    "ga jadi",
    "gak jadi",
    "enggak jadi",
    "lupakan",
    "lupakan saja",
    "sudah lupakan",
    "cancel",
    "never mind",
    "nevermind",
    "forget it",
    "nothing",
    "annule",
    "annuler",
    "laisse tomber",
    "oublie",
];

/// Exact normalized utterances that toggle music play/pause.
const MUSIC_PLAY_PAUSE_PHRASES: &[&str] = &[
    // Indonesian
    "putar musik",
    "putar musiknya",
    "putar lagu",
    "mainkan musik",
    "mainkan musiknya",
    "nyalakan musik",
    "lanjutkan musik",
    "lanjutkan lagu",
    "lanjut musik",
    "lanjut",
    "jeda",
    "jeda musik",
    "jeda lagu",
    "hentikan musik",
    "hentikan lagu",
    "berhenti musik",
    "matikan musik",
    "stop musik",
    "pause musik",
    "pause lagu",
    "play musik",
    // English
    "play",
    "pause",
    "resume",
    "play music",
    "pause music",
    "resume music",
    "stop music",
    "toggle music",
    "play pause",
    "unpause",
    "start music",
    // French
    "mets la musique",
    "mets de la musique",
    "lance la musique",
    "joue de la musique",
    "reprends la musique",
    "reprendre la musique",
    "pause la musique",
    "mets en pause",
    "coupe la musique",
    "arrete la musique",
];

/// Exact normalized utterances that skip to the next track.
const MUSIC_NEXT_PHRASES: &[&str] = &[
    // Indonesian
    "lagu berikutnya",
    "lagu selanjutnya",
    "musik berikutnya",
    "musik selanjutnya",
    "trek berikutnya",
    "trek selanjutnya",
    "ganti lagu",
    "lewatkan lagu",
    "lewati lagu",
    "skip lagu",
    "next lagu",
    // English
    "next",
    "skip",
    "next song",
    "next track",
    "skip song",
    "skip track",
    "skip this song",
    // French
    "suivant",
    "morceau suivant",
    "chanson suivante",
    "titre suivant",
    "passe au suivant",
    "passe a la suivante",
];

/// Exact normalized utterances that go back to the previous track.
const MUSIC_PREV_PHRASES: &[&str] = &[
    // Indonesian
    "lagu sebelumnya",
    "musik sebelumnya",
    "trek sebelumnya",
    "kembali ke lagu sebelumnya",
    "ulang lagu sebelumnya",
    // English
    "previous",
    "prev",
    "previous song",
    "previous track",
    "last song",
    "go back a song",
    // French
    "precedent",
    "morceau precedent",
    "chanson precedente",
    "titre precedent",
];

/// Exact normalized utterances that step volume up.
const VOLUME_UP_PHRASES: &[&str] = &[
    "besarkan suara",
    "besarkan volume",
    "naikkan suara",
    "naikkan volume",
    "keraskan suara",
    "keraskan volume",
    "tambah volume",
    "volume naik",
    "volume up",
    "turn up the volume",
    "turn up volume",
    "louder",
    "increase volume",
    "monte le son",
    "augmente le son",
    "augmente le volume",
    "plus fort",
];

/// Exact normalized utterances that step volume down.
const VOLUME_DOWN_PHRASES: &[&str] = &[
    "kecilkan suara",
    "kecilkan volume",
    "turunkan suara",
    "turunkan volume",
    "pelankan suara",
    "pelankan volume",
    "kurangi volume",
    "volume turun",
    "volume down",
    "turn down the volume",
    "turn down volume",
    "quieter",
    "softer",
    "decrease volume",
    "lower volume",
    "baisse le son",
    "diminue le son",
    "diminue le volume",
    "moins fort",
];

/// Exact normalized utterances that mute volume (0%).
const VOLUME_MUTE_PHRASES: &[&str] = &[
    "bisukan",
    "bisukan suara",
    "bisukan musik",
    "matikan suara",
    "mute",
    "mute music",
    "mute volume",
    " mute sound",
    "coupe le son",
    "muet",
];

/// Exact normalized utterances that maximize volume (100%).
const VOLUME_MAX_PHRASES: &[&str] = &[
    "volume maksimal",
    "volume maksimum",
    "volume penuh",
    "suara maksimal",
    "suara penuh",
    "volume max",
    "max volume",
    "maximum volume",
    "full volume",
    "volume au maximum",
    "son au maximum",
    "son a fond",
];

/// Non-application words that must never be treated as an app name after "buka"/"open"/"ouvre".
const NON_APP_WORDS: &[&str] = &[
    // Question words
    "apa",
    "apakah",
    "bagaimana",
    "kenapa",
    "mengapa",
    "siapa",
    "kapan",
    "dimana",
    "kemana",
    "berapa",
    "what",
    "how",
    "why",
    "who",
    "when",
    "where",
    "which",
    "quoi",
    "comment",
    "pourquoi",
    "qui",
    "quand",
    "ou",
    // Non-app domain nouns handled by other tools or conversational brain
    "email",
    "e-mail",
    "mail",
    "surat",
    "pesan",
    "musik",
    "music",
    "musique",
    "lagu",
    "song",
    "chanson",
    "pil",
    "pill",
    "pills",
    "pilule",
    "pilules",
    "internet",
    "web",
    "website",
    "situs",
    "halaman",
    "file",
    "files",
    "berkas",
    "dokumen",
    "folder",
    "direktori",
    "pintasan",
    "suara",
    "volume",
];

/// Returns true if `text` is a cancel command.
pub fn is_cancel(text: &str) -> bool {
    matches!(parse(text), Some(LocalIntent::Cancel))
}

/// Parses a voice transcript into a deterministic `LocalIntent` when it clearly
/// matches a local action. Returns `None` for questions or complex requests.
pub fn parse(raw: &str) -> Option<LocalIntent> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > VOICE_MAX_ARG_CHARS * 2 {
        return None;
    }
    // Explicit questions (ending with '?') go to the conversational brain.
    if trimmed.ends_with('?') {
        return None;
    }

    let norm = wake::normalize(trimmed);
    if norm.is_empty() {
        return None;
    }
    let clean = strip_affixes(&norm);
    if clean.is_empty() {
        return None;
    }

    // 1. Cancel / dismiss
    if CANCEL_PHRASES.contains(&clean.as_str()) || CANCEL_PHRASES.contains(&norm.as_str()) {
        return Some(LocalIntent::Cancel);
    }

    // 2. Pill management (checked before music/apps so "tambah pil Spotify" or
    //    "buka pil GitHub" is never mistaken for music or app launch).
    if let Some(intent) = parse_pill_intent(&clean) {
        return Some(intent);
    }

    // 3. Volume controls (explicit percentage or step up/down/mute/max)
    if let Some(vol) = parse_volume_intent(&clean) {
        return Some(LocalIntent::MusicVolume(vol));
    }

    // 4. Music transport controls (play/pause, next, previous)
    if MUSIC_PLAY_PAUSE_PHRASES.contains(&clean.as_str()) {
        return Some(LocalIntent::MusicPlayPause);
    }
    if MUSIC_NEXT_PHRASES.contains(&clean.as_str()) {
        return Some(LocalIntent::MusicNext);
    }
    if MUSIC_PREV_PHRASES.contains(&clean.as_str()) {
        return Some(LocalIntent::MusicPrevious);
    }

    // 5. Music search ("putar lagu Hindia", "play Get Lucky on Spotify", etc.)
    if let Some(query) = parse_music_search(&clean) {
        return Some(LocalIntent::MusicSearch(query));
    }

    // 6. Open local application ("buka VS Code", "open Figma", "ouvre Discord")
    if let Some(app_name) = parse_open_app(&clean) {
        return Some(LocalIntent::OpenApp(app_name));
    }

    None
}

/// Attempts to handle `question` locally without calling the LLM.
/// Returns `Some(Reply)` when a deterministic local intent was matched and executed.
pub fn respond(app: &AppHandle, question: &str) -> Option<Reply> {
    let intent = parse(question)?;
    let lang = wake::speech_lang(question);
    Some(execute(app, intent, lang))
}

/// Executes a parsed `LocalIntent` against Aira's local tools and formats a
/// concise, natural response in the user's spoken language (`"id-ID"`, `"fr-FR"`, or English).
pub fn execute(app: &AppHandle, intent: LocalIntent, lang: Option<&'static str>) -> Reply {
    match intent {
        LocalIntent::Cancel => Reply {
            text: String::new(),
            acted: false,
        },
        LocalIntent::MusicPlayPause => {
            let (_, ok) = super::run_tool(app, "music", &json!({ "action": "play_pause" }));
            if ok {
                Reply { text: String::new(), acted: true }
            } else {
                Reply { text: spotify_not_running_msg(lang), acted: false }
            }
        }
        LocalIntent::MusicNext => {
            let (_, ok) = super::run_tool(app, "music", &json!({ "action": "next" }));
            if ok {
                Reply { text: String::new(), acted: true }
            } else {
                Reply { text: spotify_not_running_msg(lang), acted: false }
            }
        }
        LocalIntent::MusicPrevious => {
            let (_, ok) = super::run_tool(app, "music", &json!({ "action": "previous" }));
            if ok {
                Reply { text: String::new(), acted: true }
            } else {
                Reply { text: spotify_not_running_msg(lang), acted: false }
            }
        }
        LocalIntent::MusicVolume(vol) => {
            let clamped = vol.clamp(VOICE_VOLUME_MIN, VOICE_VOLUME_MAX);
            let (_, ok) = super::run_tool(
                app,
                "music",
                &json!({ "action": "set_volume", "volume": clamped }),
            );
            if ok {
                let text = match lang {
                    Some("id-ID") => format!("Volume diatur ke {clamped} persen."),
                    Some("fr-FR") => format!("Volume réglé à {clamped} pour cent."),
                    _ => format!("Volume set to {clamped} percent."),
                };
                Reply { text, acted: true }
            } else {
                Reply { text: spotify_not_running_msg(lang), acted: false }
            }
        }
        LocalIntent::MusicSearch(query) => {
            let (_, ok) = super::run_tool(
                app,
                "music",
                &json!({ "action": "search", "query": query }),
            );
            let text = if ok {
                match lang {
                    Some("id-ID") => format!("Mencari {query} di Spotify."),
                    Some("fr-FR") => format!("Recherche de {query} sur Spotify."),
                    _ => format!("Searching for {query} on Spotify."),
                }
            } else {
                spotify_not_running_msg(lang)
            };
            Reply { text, acted: ok }
        }
        LocalIntent::OpenApp(name) => {
            let (out, ok) = super::run_tool(app, "open_app", &json!({ "name": name }));
            let text = if ok {
                let label = out.strip_suffix(" opened").unwrap_or(&name);
                match lang {
                    Some("id-ID") => format!("Membuka {label}."),
                    Some("fr-FR") => format!("Ouverture de {label}."),
                    _ => format!("Opening {label}."),
                }
            } else {
                match lang {
                    Some("id-ID") => format!("Aplikasi {name} tidak ditemukan di komputer ini."),
                    Some("fr-FR") => format!("Aucune application nommée {name} sur cet ordinateur."),
                    _ => format!("Couldn't find an app named {name} on this computer."),
                }
            };
            Reply { text, acted: ok }
        }
        LocalIntent::PillAdd(names) => {
            let (out, _) = super::run_tool(
                app,
                "pills",
                &json!({ "action": "add", "names": names }),
            );
            format_pill_reply("add", &names, &out, lang)
        }
        LocalIntent::PillRemove(names) => {
            let (out, _) = super::run_tool(
                app,
                "pills",
                &json!({ "action": "remove", "names": names }),
            );
            format_pill_reply("remove", &names, &out, lang)
        }
        LocalIntent::PillSetMain(name) => {
            let list = vec![name];
            let (out, _) = super::run_tool(
                app,
                "pills",
                &json!({ "action": "set_main", "names": list }),
            );
            format_pill_reply("set_main", &list, &out, lang)
        }
    }
}

fn spotify_not_running_msg(lang: Option<&'static str>) -> String {
    match lang {
        Some("id-ID") => "Spotify belum aktif. Silakan buka Spotify terlebih dahulu.".into(),
        Some("fr-FR") => "Spotify n'est pas ouvert. Lance Spotify d'abord.".into(),
        _ => "Spotify isn't running yet. Please open Spotify first.".into(),
    }
}

fn format_pill_reply(
    action: &str,
    requested: &[String],
    raw_json: &str,
    lang: Option<&'static str>,
) -> Reply {
    if let Ok(val) = serde_json::from_str::<Value>(raw_json) {
        let done: Vec<String> = val["done"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        if !done.is_empty() {
            let joined = done.join(", ");
            let text = match (action, lang) {
                ("add", Some("id-ID")) => format!("Pil {joined} ditambahkan."),
                ("add", Some("fr-FR")) => format!("Pilule {joined} ajoutée."),
                ("add", _) => format!("Added {joined}."),
                ("remove", Some("id-ID")) => format!("Pil {joined} dihapus."),
                ("remove", Some("fr-FR")) => format!("Pilule {joined} retirée."),
                ("remove", _) => format!("Removed {joined}."),
                ("set_main", Some("id-ID")) => format!("{joined} dijadikan pil utama."),
                ("set_main", Some("fr-FR")) => format!("{joined} est maintenant la pilule principale."),
                ("set_main", _) => format!("{joined} is now the main pill."),
                _ => joined,
            };
            return Reply { text, acted: true };
        }
    }
    let fallback = requested.join(", ");
    let text = match lang {
        Some("id-ID") => format!("Pil {fallback} tidak dapat diubah."),
        Some("fr-FR") => format!("Impossible de modifier la pilule {fallback}."),
        _ => format!("Couldn't update the {fallback} pill."),
    };
    Reply { text, acted: false }
}

// ── Normalisation & Pattern Helpers ───────────────────────────────────────────

/// Strips leading politeness prefixes and trailing courtesy particles.
fn strip_affixes(norm: &str) -> String {
    let mut s = norm.trim();
    loop {
        let mut changed = false;
        for prefix in POLITENESS_PREFIXES {
            if let Some(rest) = strip_word_prefix(s, prefix) {
                s = rest;
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    loop {
        let mut changed = false;
        for suffix in TRAILING_PARTICLES {
            if let Some(rest) = strip_word_suffix(s, suffix) {
                if !rest.is_empty() {
                    s = rest;
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }
    s.to_string()
}

fn strip_word_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = s.strip_prefix(prefix)?;
    if rest.is_empty() {
        Some("")
    } else if rest.starts_with(' ') {
        Some(rest.trim_start())
    } else {
        None
    }
}

fn strip_word_suffix<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    let rest = s.strip_suffix(suffix)?;
    if rest.is_empty() {
        Some("")
    } else if rest.ends_with(' ') {
        Some(rest.trim_end())
    } else {
        None
    }
}

/// Parses volume commands: explicit percentage (`"volume 80"`, `"atur volume ke 50"`,
/// `"set volume to 65"`, `"mets le son a 40"`) or step presets.
fn parse_volume_intent(clean: &str) -> Option<u8> {
    if VOLUME_UP_PHRASES.contains(&clean) {
        return Some(VOICE_VOLUME_STEP_UP);
    }
    if VOLUME_DOWN_PHRASES.contains(&clean) {
        return Some(VOICE_VOLUME_STEP_DOWN);
    }
    if VOLUME_MUTE_PHRASES.iter().any(|p| p.trim() == clean) {
        return Some(VOICE_VOLUME_MIN);
    }
    if VOLUME_MAX_PHRASES.contains(&clean) {
        return Some(VOICE_VOLUME_MAX);
    }

    let words: Vec<&str> = clean.split_whitespace().collect();
    if words.is_empty() || words.len() > 7 {
        return None;
    }

    let has_volume_keyword = words.iter().any(|w| {
        matches!(
            *w,
            "volume" | "suara" | "sound" | "son" | "audio"
        )
    });
    if !has_volume_keyword {
        return None;
    }

    // Must only contain allowed volume grammar words + a number (or spoken Indonesian/English/French number).
    let allowed_context = [
        "volume", "suara", "sound", "son", "audio", "atur", "set", "ubah", "ganti", "naikkan",
        "turunkan", "mets", "regle", "ke", "di", "jadi", "menjadi", "to", "at", "a", "le", "la",
        "the", "persen", "percent", "pour", "cent", "pourcent",
    ];
    let number = extract_volume_number(&words, &allowed_context)?;
    Some(number.clamp(VOICE_VOLUME_MIN as u32, VOICE_VOLUME_MAX as u32) as u8)
}

fn extract_volume_number(words: &[&str], allowed_context: &[&str]) -> Option<u32> {
    let mut found: Option<u32> = None;
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if let Ok(n) = w.parse::<u32>() {
            if found.is_some() {
                return None;
            }
            found = Some(n);
            i += 1;
            continue;
        }
        // Spoken two-word Indonesian numbers like "delapan puluh" (80), "lima puluh" (50), "seratus" (100)
        if i + 1 < words.len() && words[i + 1] == "puluh" {
            if let Some(tens) = id_unit_digit(w) {
                let mut val = tens * 10;
                i += 2;
                if i < words.len() {
                    if let Some(ones) = id_unit_digit(words[i]) {
                        val += ones;
                        i += 1;
                    }
                }
                found = Some(val);
                continue;
            }
        }
        if let Some(n) = spoken_number(w) {
            if found.is_some() {
                return None;
            }
            found = Some(n);
            i += 1;
            continue;
        }
        if !allowed_context.contains(&w) {
            return None;
        }
        i += 1;
    }
    found
}

fn id_unit_digit(w: &str) -> Option<u32> {
    match w {
        "satu" => Some(1),
        "dua" => Some(2),
        "tiga" => Some(3),
        "empat" => Some(4),
        "lima" => Some(5),
        "enam" => Some(6),
        "tujuh" => Some(7),
        "delapan" => Some(8),
        "sembilan" => Some(9),
        _ => None,
    }
}

fn spoken_number(w: &str) -> Option<u32> {
    match w {
        "nol" | "zero" => Some(0),
        "sepuluh" | "ten" | "dix" => Some(10),
        "dua_puluh" | "twenty" | "vingt" => Some(20),
        "thirty" | "trente" => Some(30),
        "forty" | "quarante" => Some(40),
        "setengah" | "fifty" | "cinquante" => Some(50),
        "sixty" | "soixante" => Some(60),
        "seventy" => Some(70),
        "eighty" | "quatre_vingts" => Some(80),
        "ninety" => Some(90),
        "seratus" | "hundred" | "cent" => Some(100),
        _ => id_unit_digit(w),
    }
}

/// Parses pill management intents (`PillAdd`, `PillRemove`, `PillSetMain`).
fn parse_pill_intent(clean: &str) -> Option<LocalIntent> {
    // 1. Set main pill:
    //    ID: "jadikan cursor pil utama", "atur pil utama ke cursor", "pil utama cursor", "ganti pil utama ke cursor"
    //    EN: "set main pill to cursor", "make cursor main pill", "make cursor my main pill", "main pill cursor"
    //    FR: "pilule principale cursor", "mets cursor en pilule principale", "passe la pilule principale sur cursor"
    for prefix in &[
        "atur pil utama ke",
        "ganti pil utama ke",
        "ubah pil utama ke",
        "jadikan pil utama",
        "pil utama",
        "set main pill to",
        "set the main pill to",
        "switch main pill to",
        "switch the main pill to",
        "main pill",
        "passe la pilule principale sur",
        "change la pilule principale pour",
        "pilule principale",
    ] {
        if let Some(rest) = strip_word_prefix(clean, prefix) {
            let target = clean_pill_target(rest);
            if is_valid_pill_name(&target) {
                return Some(LocalIntent::PillSetMain(target));
            }
        }
    }
    if let Some(rest) = strip_word_prefix(clean, "jadikan") {
        for suffix in &["sebagai pil utama", "jadi pil utama", "pil utama"] {
            if let Some(mid) = strip_word_suffix(rest, suffix) {
                let target = clean_pill_target(mid);
                if is_valid_pill_name(&target) {
                    return Some(LocalIntent::PillSetMain(target));
                }
            }
        }
    }
    if let Some(rest) = strip_word_prefix(clean, "make") {
        for suffix in &["my main pill", "the main pill", "main pill"] {
            if let Some(mid) = strip_word_suffix(rest, suffix) {
                let target = clean_pill_target(mid);
                if is_valid_pill_name(&target) {
                    return Some(LocalIntent::PillSetMain(target));
                }
            }
        }
    }
    if let Some(rest) = strip_word_prefix(clean, "mets") {
        for suffix in &["en pilule principale", "en principal", "en principale"] {
            if let Some(mid) = strip_word_suffix(rest, suffix) {
                let target = clean_pill_target(mid);
                if is_valid_pill_name(&target) {
                    return Some(LocalIntent::PillSetMain(target));
                }
            }
        }
    }

    // 2. Add pill(s):
    //    ID: "tambah pil stripe", "tambahkan pil github dan vercel", "pasang pil spotify", "aktifkan pil claude"
    //    EN: "add pill stripe", "add stripe pill", "show pill github"
    //    FR: "ajoute la pilule stripe", "ajoute pilule github"
    for prefix in &[
        "tambahkan pil",
        "tambah pil",
        "pasang pil",
        "aktifkan pil",
        "tampilkan pil",
        "add the pill",
        "add pill",
        "show pill",
        "enable pill",
        "ajoute la pilule",
        "ajouter la pilule",
        "ajoute pilule",
        "affiche la pilule",
    ] {
        if let Some(rest) = strip_word_prefix(clean, prefix) {
            let names = split_pill_names(rest);
            if !names.is_empty() {
                return Some(LocalIntent::PillAdd(names));
            }
        }
    }
    // "add stripe pill" / "add the stripe pill"
    if let Some(rest) = strip_word_prefix(clean, "add") {
        if let Some(mid) = strip_word_suffix(rest, "pill").or_else(|| strip_word_suffix(rest, "pills")) {
            let names = split_pill_names(mid);
            if !names.is_empty() {
                return Some(LocalIntent::PillAdd(names));
            }
        }
    }

    // 3. Remove pill(s):
    //    ID: "hapus pil stripe", "hilangkan pil github", "lepas pil spotify", "matikan pil vercel"
    //    EN: "remove pill stripe", "remove stripe pill", "hide pill github"
    //    FR: "enleve la pilule stripe", "supprime la pilule github", "retire la pilule vercel"
    for prefix in &[
        "hilangkan pil",
        "hapuskan pil",
        "hapus pil",
        "lepas pil",
        "matikan pil",
        "sembunyikan pil",
        "remove the pill",
        "remove pill",
        "delete pill",
        "hide pill",
        "disable pill",
        "enleve la pilule",
        "enlever la pilule",
        "enleve pilule",
        "supprime la pilule",
        "supprimer la pilule",
        "retire la pilule",
        "retirer la pilule",
    ] {
        if let Some(rest) = strip_word_prefix(clean, prefix) {
            let names = split_pill_names(rest);
            if !names.is_empty() {
                return Some(LocalIntent::PillRemove(names));
            }
        }
    }
    if let Some(rest) = strip_word_prefix(clean, "remove") {
        if let Some(mid) = strip_word_suffix(rest, "pill").or_else(|| strip_word_suffix(rest, "pills")) {
            let names = split_pill_names(mid);
            if !names.is_empty() {
                return Some(LocalIntent::PillRemove(names));
            }
        }
    }

    None
}

fn clean_pill_target(s: &str) -> String {
    let mut out = s.trim();
    for article in &["the", "my", "la", "le", "les", "pil", "pill", "pilule"] {
        if let Some(r) = strip_word_prefix(out, article) {
            out = r;
        }
    }
    for loc in &["di pulau", "di notch", "in the notch", "dans les pilules", "dans le notch"] {
        if let Some(r) = strip_word_suffix(out, loc) {
            out = r;
        }
    }
    out.trim().to_string()
}

fn is_valid_pill_name(s: &str) -> bool {
    let words = s.split_whitespace().count();
    !s.is_empty() && s.len() <= VOICE_MAX_ARG_CHARS && words >= 1 && words <= VOICE_MAX_PILL_WORDS
}

fn split_pill_names(raw: &str) -> Vec<String> {
    let cleaned = clean_pill_target(raw);
    if cleaned.is_empty() {
        return Vec::new();
    }
    // Split on conjunctions: "dan", "and", "et", "serta"
    let mut parts: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for word in cleaned.split_whitespace() {
        if matches!(word, "dan" | "and" | "et" | "serta") {
            if !current.is_empty() {
                let candidate = clean_pill_target(&current.join(" "));
                if is_valid_pill_name(&candidate) {
                    parts.push(candidate);
                }
                current.clear();
            }
        } else {
            current.push(word);
        }
    }
    if !current.is_empty() {
        let candidate = clean_pill_target(&current.join(" "));
        if is_valid_pill_name(&candidate) {
            parts.push(candidate);
        } else {
            return Vec::new();
        }
    }
    parts
}

/// Parses explicit song/artist search commands on Spotify.
fn parse_music_search(clean: &str) -> Option<String> {
    // 1. Explicit song/track prefixes:
    //    ID: "putar lagu <q>", "mainkan lagu <q>", "cari lagu <q>", "setel lagu <q>"
    //    EN: "play song <q>", "play the song <q>", "play track <q>"
    //    FR: "joue la chanson <q>", "mets la chanson <q>", "mets le morceau <q>"
    for prefix in &[
        "putar lagu",
        "mainkan lagu",
        "setel lagu",
        "nyalakan lagu",
        "cari lagu",
        "dengarkan lagu",
        "play the song",
        "play song",
        "play the track",
        "play track",
        "joue la chanson",
        "joue le morceau",
        "mets la chanson",
        "mets le morceau",
    ] {
        if let Some(rest) = strip_word_prefix(clean, prefix) {
            let q = strip_spotify_suffix(rest);
            if is_valid_search_query(&q) {
                return Some(q);
            }
        }
    }

    // 2. "<verb> <query> di spotify / on spotify / sur spotify"
    for suffix in &["di spotify", "on spotify", "in spotify", "sur spotify"] {
        if let Some(before_spotify) = strip_word_suffix(clean, suffix) {
            for verb in &[
                "putar", "mainkan", "setel", "cari", "dengarkan", "play", "search", "listen to",
                "mets", "joue", "lance", "ecoute",
            ] {
                if let Some(query) = strip_word_prefix(before_spotify, verb) {
                    let q = query
                        .strip_prefix("lagu ")
                        .or_else(|| query.strip_prefix("musik "))
                        .or_else(|| query.strip_prefix("song "))
                        .unwrap_or(query)
                        .trim();
                    if is_valid_search_query(q) && !matches!(q, "musik" | "music" | "musique" | "lagu") {
                        return Some(q.to_string());
                    }
                }
            }
        }
    }

    None
}

fn strip_spotify_suffix(s: &str) -> String {
    let mut out = s.trim();
    for suffix in &["di spotify", "on spotify", "in spotify", "sur spotify"] {
        if let Some(r) = strip_word_suffix(out, suffix) {
            out = r;
        }
    }
    out.trim().to_string()
}

fn is_valid_search_query(q: &str) -> bool {
    !q.is_empty()
        && q.len() <= VOICE_MAX_ARG_CHARS
        && q.split_whitespace().count() <= 10
        && !matches!(
            q,
            "berikutnya"
                | "selanjutnya"
                | "sebelumnya"
                | "ini"
                | "itu"
                | "next"
                | "previous"
                | "suivant"
                | "precedent"
        )
}

/// Parses application launch commands (`"buka VS Code"`, `"open Figma"`, `"ouvre Discord"`).
fn parse_open_app(clean: &str) -> Option<String> {
    for prefix in &[
        "buka aplikasi",
        "jalankan aplikasi",
        "nyalakan aplikasi",
        "buka program",
        "jalankan program",
        "open the application",
        "open application",
        "open the app",
        "open app",
        "launch the app",
        "launch app",
        "start application",
        "start app",
        "ouvre l application",
        "ouvrir l application",
        "lance l application",
        "lancer l application",
        "buka",
        "jalankan",
        "open",
        "launch",
        "ouvre",
        "ouvrir",
    ] {
        if let Some(rest) = strip_word_prefix(clean, prefix) {
            let candidate = clean_app_target(rest)?;
            return Some(normalize_app_alias(&candidate));
        }
    }
    None
}

fn clean_app_target(raw: &str) -> Option<String> {
    let mut s = raw.trim();
    for article in &["aplikasi", "program", "app", "application", "the", "le", "la", "l"] {
        if let Some(r) = strip_word_prefix(s, article) {
            s = r;
        }
    }
    if s.is_empty() || s.len() > VOICE_MAX_ARG_CHARS {
        return None;
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() || words.len() > VOICE_MAX_APP_WORDS {
        return None;
    }
    // Reject if the first word or any single-word target is a non-app keyword
    // (e.g., "buka email", "buka musik", "buka internet", "buka apa").
    if NON_APP_WORDS.contains(&words[0]) {
        return None;
    }
    Some(s.to_string())
}

/// Maps common spoken shorthand names to their canonical desktop shortcut / `.desktop` names.
fn normalize_app_alias(name: &str) -> String {
    match name {
        "vs code" | "vscode" | "code" => "visual studio code".into(),
        "ms word" => "word".into(),
        "ms excel" => "excel".into(),
        "power point" => "powerpoint".into(),
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_indonesian_music_and_volume_commands() {
        assert_eq!(parse("putar musik"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("Tolong putar musik sekarang"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("jeda musik"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("pause"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("lanjut"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("lagu berikutnya"), Some(LocalIntent::MusicNext));
        assert_eq!(parse("tolong skip lagu dong"), Some(LocalIntent::MusicNext));
        assert_eq!(parse("lagu sebelumnya"), Some(LocalIntent::MusicPrevious));
        assert_eq!(parse("volume 80"), Some(LocalIntent::MusicVolume(80)));
        assert_eq!(parse("atur volume ke 50 persen"), Some(LocalIntent::MusicVolume(50)));
        assert_eq!(parse("suara delapan puluh persen"), Some(LocalIntent::MusicVolume(80)));
        assert_eq!(parse("volume 150"), Some(LocalIntent::MusicVolume(100)));
        assert_eq!(parse("besarkan suara"), Some(LocalIntent::MusicVolume(VOICE_VOLUME_STEP_UP)));
        assert_eq!(parse("kecilkan suara"), Some(LocalIntent::MusicVolume(VOICE_VOLUME_STEP_DOWN)));
        assert_eq!(parse("bisukan suara"), Some(LocalIntent::MusicVolume(0)));
        assert_eq!(
            parse("putar lagu Hindia Evaluasi"),
            Some(LocalIntent::MusicSearch("hindia evaluasi".into()))
        );
        assert_eq!(
            parse("mainkan Tulus di Spotify"),
            Some(LocalIntent::MusicSearch("tulus".into()))
        );
    }

    #[test]
    fn parses_english_and_french_music_commands() {
        assert_eq!(parse("play music"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("can you pause music please"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("next track"), Some(LocalIntent::MusicNext));
        assert_eq!(parse("previous song"), Some(LocalIntent::MusicPrevious));
        assert_eq!(parse("set volume to 65%"), Some(LocalIntent::MusicVolume(65)));
        assert_eq!(
            parse("play Daft Punk on Spotify"),
            Some(LocalIntent::MusicSearch("daft punk".into()))
        );

        assert_eq!(parse("mets la musique"), Some(LocalIntent::MusicPlayPause));
        assert_eq!(parse("morceau suivant"), Some(LocalIntent::MusicNext));
        assert_eq!(parse("morceau précédent"), Some(LocalIntent::MusicPrevious));
        assert_eq!(parse("mets le son à 40"), Some(LocalIntent::MusicVolume(40)));
    }

    #[test]
    fn parses_open_app_commands_and_normalizes_aliases() {
        assert_eq!(
            parse("buka VS Code"),
            Some(LocalIntent::OpenApp("visual studio code".into()))
        );
        assert_eq!(
            parse("tolong buka aplikasi Discord sekarang"),
            Some(LocalIntent::OpenApp("discord".into()))
        );
        assert_eq!(
            parse("open Figma"),
            Some(LocalIntent::OpenApp("figma".into()))
        );
        assert_eq!(
            parse("ouvre Spotify"),
            Some(LocalIntent::OpenApp("spotify".into()))
        );
    }

    #[test]
    fn parses_pill_commands_across_languages() {
        assert_eq!(
            parse("tambah pil Stripe"),
            Some(LocalIntent::PillAdd(vec!["stripe".into()]))
        );
        assert_eq!(
            parse("tambahkan pil GitHub dan Vercel"),
            Some(LocalIntent::PillAdd(vec!["github".into(), "vercel".into()]))
        );
        assert_eq!(
            parse("hapus pil Notion"),
            Some(LocalIntent::PillRemove(vec!["notion".into()]))
        );
        assert_eq!(
            parse("jadikan Cursor pil utama"),
            Some(LocalIntent::PillSetMain("cursor".into()))
        );
        assert_eq!(
            parse("add Stripe pill"),
            Some(LocalIntent::PillAdd(vec!["stripe".into()]))
        );
        assert_eq!(
            parse("make Cursor my main pill"),
            Some(LocalIntent::PillSetMain("cursor".into()))
        );
        assert_eq!(
            parse("ajoute la pilule GitHub"),
            Some(LocalIntent::PillAdd(vec!["github".into()]))
        );
    }

    #[test]
    fn parses_cancel_commands() {
        assert_eq!(parse("batalkan"), Some(LocalIntent::Cancel));
        assert_eq!(parse("tidak jadi"), Some(LocalIntent::Cancel));
        assert_eq!(parse("cancel"), Some(LocalIntent::Cancel));
        assert_eq!(parse("never mind"), Some(LocalIntent::Cancel));
        assert_eq!(parse("annule"), Some(LocalIntent::Cancel));
        assert!(is_cancel("batalkan"));
    }

    #[test]
    fn leaves_open_questions_and_non_app_targets_for_llm_brain() {
        assert_eq!(parse("apa kabar hari ini?"), None);
        assert_eq!(parse("jelaskan cara kerja rust async"), None);
        assert_eq!(parse("berapa bintang di repositori github saya"), None);
        assert_eq!(parse("buka email untuk budi@example.com"), None);
        assert_eq!(parse("buka apa yang harus saya pelajari hari ini"), None);
        assert_eq!(parse("open email to john"), None);
    }
}

