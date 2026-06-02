//! Sound effects for the launcher.
//!
//! Every clip is compiled INTO the binary with `include_bytes!`, so there is no
//! sounds folder to ship at runtime — they live inside the exe. (You still keep
//! a `sounds/` folder in the project for the source files; it just gets baked in
//! at compile time.)
//!
//! Quick start in main.rs:
//!   mod audio;                                    // top of main.rs
//!   let sfx = std::rc::Rc::new(audio::Audio::new());   // before ui.run()
//!
//!   // then inside any callback closure:
//!   //   let sfx = sfx.clone();
//!   //   move || { /* ... */ sfx.select(); }

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;

use std::cell::Cell;
use std::time::{Duration, Instant};

const MIN_GAP: Duration = Duration::from_millis(250);

// --- Embedded clips ---------------------------------------------------------
// Paths are relative to THIS file (src/audio.rs), so `..\\sfx\\` points at a
// `sounds/` folder sitting next to Cargo.toml. Drop your .mp3s there.
//
// NOTE: every file listed here MUST exist or the project won't compile
// (include_bytes! reads them at build time).
const SELECT: &[u8] = include_bytes!("..\\sfx\\select.mp3");
const OPEN: &[u8] = include_bytes!("..\\sfx\\item-open.mp3");
const EQUIP: &[u8] = include_bytes!("..\\sfx\\item-equip.mp3");
const USE: &[u8] = include_bytes!("..\\sfx\\item-used.mp3");
const START: &[u8] = include_bytes!("..\\sfx\\codec-open.mp3");
const END: &[u8] = include_bytes!("..\\sfx\\codec-close.mp3");
const BAD: &[u8] = include_bytes!("..\\sfx\\doorbuzz.mp3");

/// Owns the audio output device.
///
/// Create exactly ONE and keep it alive for the whole program — when it drops,
/// audio stops. Every `play` is fire-and-forget.
pub struct Audio {
    // We never touch `_stream` again, but it MUST stay alive, so we hold it.
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    last: Cell<Instant>,
}

impl Audio {
    /// Opens the default output device. If there isn't one, the app still runs —
    /// sounds just become silent no-ops instead of crashing.
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self {
                _stream: Some(stream),
                handle: Some(handle),
                last: Cell::new(Instant::now() - MIN_GAP),
            },
            Err(e) => {
                eprintln!("audio disabled (no output device): {e}");
                Self {
                    _stream: None,
                    handle: None,
                    last: Cell::new(Instant::now() - MIN_GAP),
                }
            }
        }
    }

    fn play(&self, bytes: &'static [u8]) {
        let Some(handle) = self.handle.as_ref() else { return };
        if let Ok(sink) = Sink::try_new(handle) {
            if let Ok(source) = Decoder::new(Cursor::new(bytes)) {
                sink.append(source);
                sink.detach();
            }
        }
    }

    // --- one method per sound ---------------
    pub fn select(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < MIN_GAP {
            return;
        }
        self.last.set(now);
        
        self.play(SELECT);
    }
    pub fn open(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < MIN_GAP {
            return;
        }
        self.last.set(now);

        self.play(OPEN);
    }
    pub fn equip(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < MIN_GAP {
            return;
        }
        self.last.set(now);

        self.play(EQUIP);
    }
    pub fn item(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < Duration::from_millis(500) {
            return;
        }

        self.play(USE);
    }
    pub fn start(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < MIN_GAP {
            return;
        }
        self.last.set(now);

        self.play(START);
    }
    pub fn end(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < MIN_GAP {
            return;
        }
        self.last.set(now);
        
        self.play(END);
    }
    pub fn bad(&self) {
        let now = Instant::now();
        if now.duration_since(self.last.get()) < Duration::from_millis(800) {
            return;
        }
        self.last.set(now);
        
        self.play(BAD);
    }
}