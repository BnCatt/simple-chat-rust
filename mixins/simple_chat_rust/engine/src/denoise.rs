//! Noise suppression with rnnoise (xiph/rnnoise, BSD-3-Clause), as Simple
//! Voice Chat's "denoiser": loaded at run time like libopus, from the mod's
//! data folder (`rnnoise.dll`) or the system. Without it the microphone's
//! sound goes out as it is.
use crate::dylib;
use crate::opus::FRAME;
use libloading::Library;
use std::ffi::{c_int, c_void};
use std::sync::OnceLock;

/// rnnoise's frame: 10 ms at 48 kHz, so two per Opus frame.
pub const RNN_FRAME: usize = 480;

type Create = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type Process = unsafe extern "C" fn(*mut c_void, *mut f32, *const f32) -> f32;
type Destroy = unsafe extern "C" fn(*mut c_void);
type FrameSize = unsafe extern "C" fn() -> c_int;

struct Api {
    _lib: Library,
    create: Create,
    process: Process,
    destroy: Destroy,
}

const LOCAL: &[&str] = if cfg!(windows) {
    &["rnnoise.dll", "librnnoise-0.dll"]
} else if cfg!(target_os = "macos") {
    &["librnnoise.0.dylib", "librnnoise.dylib"]
} else {
    &["librnnoise.so.0", "librnnoise.so"]
};

const NAMES: &[&str] = &[
    "librnnoise.so.0",
    "librnnoise.so",
    "librnnoise.0.dylib",
    "/opt/homebrew/lib/librnnoise.0.dylib",
    "/usr/local/lib/librnnoise.0.dylib",
    "rnnoise.dll",
    "librnnoise-0.dll",
];

fn api() -> Result<&'static Api, String> {
    static API: OnceLock<Result<Api, String>> = OnceLock::new();
    API.get_or_init(load).as_ref().map_err(Clone::clone)
}

fn load() -> Result<Api, String> {
    let lib = dylib::load(LOCAL, NAMES).map_err(|e| {
        log::info!(target: "horizon_voicechat", "Voice Chat: no rnnoise ({e})");
        if cfg!(windows) { "rnnoise.dll not found".to_owned() } else { "librnnoise not found".to_owned() }
    })?;
    // SAFETY: the symbols have rnnoise's signatures.
    unsafe {
        let frame: Option<FrameSize> = lib.get(b"rnnoise_get_frame_size\0").ok().map(|s| *s);
        if let Some(size) = frame
            && size() as usize != RNN_FRAME
        {
            return Err(format!("rnnoise works on frames of {}, not {RNN_FRAME}", size()));
        }
        let get = |name: &[u8]| lib.get::<*const c_void>(name).map(|s| *s).map_err(|e| format!("rnnoise: {e}"));
        let (create, process, destroy) = (get(b"rnnoise_create\0")?, get(b"rnnoise_process_frame\0")?, get(b"rnnoise_destroy\0")?);
        Ok(Api {
            create: std::mem::transmute::<*const c_void, Create>(create),
            process: std::mem::transmute::<*const c_void, Process>(process),
            destroy: std::mem::transmute::<*const c_void, Destroy>(destroy),
            _lib: lib,
        })
    }
}

/// Whether rnnoise loads, and why not.
pub fn available() -> Result<(), String> {
    api().map(|_| ())
}

pub struct Denoiser {
    api: &'static Api,
    state: *mut c_void,
}

// SAFETY: a denoiser is used by one thread at a time (the mic thread).
unsafe impl Send for Denoiser {}

impl Denoiser {
    pub fn new() -> Result<Self, String> {
        let api = api()?;
        // SAFETY: a null model is rnnoise's built-in one.
        let state = unsafe { (api.create)(std::ptr::null_mut()) };
        if state.is_null() {
            return Err("rnnoise_create failed".into());
        }
        Ok(Self { api, state })
    }

    /// One Opus frame (samples -1..1), cleaned in place.
    pub fn process(&mut self, frame: &mut [f32; FRAME]) {
        let mut input = [0f32; RNN_FRAME];
        let mut output = [0f32; RNN_FRAME];
        for half in frame.chunks_exact_mut(RNN_FRAME) {
            // rnnoise takes 16 bit sized samples, as floats.
            for (i, s) in half.iter().enumerate() {
                input[i] = s * 32768.0;
            }
            // SAFETY: both buffers hold one rnnoise frame.
            unsafe { (self.api.process)(self.state, output.as_mut_ptr(), input.as_ptr()) };
            for (s, o) in half.iter_mut().zip(&output) {
                *s = (o / 32768.0).clamp(-1.0, 1.0);
            }
        }
    }
}

impl Drop for Denoiser {
    fn drop(&mut self) {
        // SAFETY: made by rnnoise_create, destroyed once.
        unsafe { (self.api.destroy)(self.state) };
    }
}

/// The mic thread's denoiser: made the first time it is wanted; `None`
/// inside when rnnoise isn't there, and the frame stays as it is.
#[derive(Default)]
pub struct Lazy(Option<Option<Denoiser>>);

impl Lazy {
    /// Cleans `frame` when `on` and rnnoise loads.
    pub fn apply(&mut self, on: bool, frame: &mut [f32; FRAME]) {
        if !on {
            return;
        }
        let d = self.0.get_or_insert_with(|| match Denoiser::new() {
            Ok(d) => Some(d),
            Err(e) => {
                log::warn!(target: "horizon_voicechat", "Voice Chat: noise suppression off: {e}");
                None
            }
        });
        if let Some(d) = d {
            d.process(frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(s: &[f32]) -> f32 {
        (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt()
    }

    /// A small, repeatable noise.
    fn noise(seed: &mut u32) -> f32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        (*seed as f32 / u32::MAX as f32 - 0.5) * 0.2
    }

    #[test]
    fn without_rnnoise_the_sound_goes_through_as_it_is() {
        // Before the first load, so the tests agree on where rnnoise is.
        dylib::test_dir();
        if available().is_ok() {
            return;
        }
        assert!(Denoiser::new().is_err());
        let mut lazy = Lazy::default();
        let mut frame: [f32; FRAME] = std::array::from_fn(|i| (i as f32 * 0.01).sin() * 0.5);
        let before = frame;
        lazy.apply(true, &mut frame);
        assert_eq!(frame, before);
    }

    #[test]
    fn hiss_is_turned_down_and_a_voice_like_tone_stays() {
        dylib::test_dir();
        if available().is_err() {
            eprintln!("no rnnoise here: skipped");
            return;
        }
        let mut d = Denoiser::new().unwrap();
        let mut seed = 0x1234_5678;
        // A second of steady hiss: its last half after the denoiser settles.
        let (mut before, mut after) = (Vec::new(), Vec::new());
        for f in 0..50 {
            let mut frame: [f32; FRAME] = std::array::from_fn(|_| noise(&mut seed));
            if f >= 25 {
                before.extend_from_slice(&frame);
            }
            d.process(&mut frame);
            if f >= 25 {
                after.extend_from_slice(&frame);
            }
        }
        let drop_db = 20.0 * (rms(&before) / rms(&after).max(1e-9)).log10();
        assert!(drop_db >= 6.0, "the hiss fell only {drop_db:.1} dB");
        // A loud 440 Hz tone with harmonics under the same hiss keeps most of itself.
        let mut d = Denoiser::new().unwrap();
        let (mut tone_in, mut tone_out) = (Vec::new(), Vec::new());
        for f in 0..50 {
            let mut frame: [f32; FRAME] = std::array::from_fn(|i| {
                let t = (f * FRAME + i) as f32 / 48_000.0;
                let w = t * 440.0 * std::f32::consts::TAU;
                (w.sin() * 0.3 + (2.0 * w).sin() * 0.15 + (3.0 * w).sin() * 0.1) + noise(&mut seed) * 0.3
            });
            if f >= 25 {
                tone_in.extend_from_slice(&frame);
            }
            d.process(&mut frame);
            if f >= 25 {
                tone_out.extend_from_slice(&frame);
            }
        }
        let kept_db = 20.0 * (rms(&tone_out) / rms(&tone_in)).log10();
        assert!(kept_db > -10.0, "the tone fell {kept_db:.1} dB");
        eprintln!("rnnoise: hiss -{drop_db:.1} dB, tone {kept_db:.1} dB");
    }
}
