use crate::shared;

/// Replaces constants ending with PLAYBACK/CAPTURE as well as
/// INPUT/OUTPUT
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
    #[cfg(feature = "speaker")]
    Playback,
    #[cfg(feature = "mic")]
    Capture,
}

/// Used to restrict hw parameters. In case the submitted
/// value is unavailable, in which direction should one search
/// for available values?
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValueOr {
    /// The value set is the submitted value, or the nearest
    Nearest = 0,
}

mod error;

pub mod pcm;

mod alsa;
pub(crate) use self::alsa::Context;
pub(crate) use self::alsa::CONTEXT;

use std::ptr::NonNull;

/// Standard Audio Hz for Opus.
pub const HZ_48K: u32 = 48_000;

pub(crate) fn lazy_init_alsa() {
    if (unsafe { CONTEXT }).is_none() {
        let context = Box::new(shared::alsa::Context::new());
        unsafe {
            CONTEXT = Some(NonNull::new(Box::into_raw(context)).unwrap());
        }
    }
}

pub(crate) fn context() -> &'static mut Context {
    unsafe { ::std::mem::transmute(CONTEXT) }
}

pub(crate) fn set_settings(pcm: &shared::alsa::pcm::PCM, stereo: bool) {
    // Set hardware parameters: 48000 Hz / Mono / 16 bit
    let hwp = shared::alsa::pcm::HwParams::any(context(), pcm).unwrap();
    hwp.set_channels(context(), if stereo { 2 } else { 1 })
        .unwrap();
    hwp.set_rate(context(), HZ_48K, shared::alsa::ValueOr::Nearest)
        .unwrap();
    let rate = hwp.get_rate(context()).unwrap();
    assert_eq!(rate, HZ_48K);
    hwp.set_format(context(), {
        if cfg!(target_endian = "little") {
            2
        } else if cfg!(target_endian = "big") {
            3
        } else {
            unreachable!()
        }
    })
    .unwrap();
    hwp.set_access(context(), shared::alsa::pcm::Access::RWInterleaved)
        .unwrap();
    pcm.hw_params(context(), &hwp).unwrap();
    hwp.drop(context());
}
