use core::time::Duration;

/// Duration of the WaveSabre song blob. The duration is stored as an f64 at
/// byte offset 8 (identical layout on Windows and macOS).
pub fn song_length(data: &[u8]) -> Duration {
    #[cfg(windows)]
    {
        wavesabre_rs::length(data)
    }

    #[cfg(target_os = "macos")]
    {
        let seconds = f64::from_le_bytes(data[8..16].try_into().unwrap());
        Duration::from_secs_f64(seconds)
    }
}

#[cfg(windows)]
pub unsafe fn play(data: &[u8]) -> wavesabre_rs::Player {
    unsafe extern "C" fn wavesabre_device_factory(
        id: wavesabre_rs::device::DeviceId,
    ) -> wavesabre_rs::device::Device {
        match id {
            wavesabre_rs::device::DeviceId::Falcon => wavesabre_rs::device::falcon(),
            wavesabre_rs::device::DeviceId::Slaughter => wavesabre_rs::device::slaughter(),
            wavesabre_rs::device::DeviceId::Thunder => wavesabre_rs::device::thunder(),
            wavesabre_rs::device::DeviceId::Scissor => wavesabre_rs::device::scissor(),
            wavesabre_rs::device::DeviceId::Leveller => wavesabre_rs::device::leveller(),
            wavesabre_rs::device::DeviceId::Crusher => wavesabre_rs::device::crusher(),
            wavesabre_rs::device::DeviceId::Echo => wavesabre_rs::device::echo(),
            wavesabre_rs::device::DeviceId::Smasher => wavesabre_rs::device::smasher(),
            wavesabre_rs::device::DeviceId::Chamber => wavesabre_rs::device::chamber(),
            wavesabre_rs::device::DeviceId::Twister => wavesabre_rs::device::twister(),
            wavesabre_rs::device::DeviceId::Cathedral => wavesabre_rs::device::cathedral(),
            wavesabre_rs::device::DeviceId::Adultery => wavesabre_rs::device::adultery(),
            wavesabre_rs::device::DeviceId::Specimen => wavesabre_rs::device::specimen(),
        }
    }

    wavesabre_rs::play(wavesabre_device_factory, data)
}

#[cfg(windows)]
pub use wavesabre_rs::Player;

#[cfg(target_os = "macos")]
pub struct Player {
    queue: coreaudio_sys::AudioQueueRef,
    _buffers: Vec<coreaudio_sys::AudioQueueBufferRef>,
}

#[cfg(target_os = "macos")]
impl Drop for Player {
    fn drop(&mut self) {
        unsafe {
            let _: i32 = coreaudio_sys::AudioQueueStop(self.queue, 1);
            let _: i32 = coreaudio_sys::AudioQueueDispose(self.queue, 1);
        }
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn play(data: &[u8]) -> Player {
    use coreaudio_sys::{
        AudioQueueBufferRef, AudioQueueRef, AudioStreamBasicDescription,
        kAudioFormatFlagIsPacked, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
    };
    use wavesabre::render_stereo;

    // Render the whole song up-front (macOS has no binary-size constraint).
    let parsed = common::decode(data).expect("invalid song blob");
    let mut frames = render_stereo(&parsed);
    wavesabre::normalize_stereo(&mut frames);

    let sample_rate = parsed.sample_rate as u32;

    let mut pcm: Vec<i16> = Vec::with_capacity(frames.len() * 2);
    for frame in &frames {
        for &s in frame.iter() {
            pcm.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
    }

    let asbd = AudioStreamBasicDescription {
        mSampleRate: sample_rate as f64,
        mFormatID: kAudioFormatLinearPCM,
        mFormatFlags: kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked,
        mBytesPerPacket: 4,
        mFramesPerPacket: 1,
        mBytesPerFrame: 4,
        mChannelsPerFrame: 2,
        mBitsPerChannel: 16,
        mReserved: 0,
    };

    let mut queue: AudioQueueRef = core::ptr::null_mut();
    let status = coreaudio_sys::AudioQueueNewOutput(
        &asbd,
        Some(output_callback),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        &mut queue,
    );
    debug_assert_eq!(status, 0);

    // Prime 0.5s of audio at a time (4 bytes per stereo frame).
    let chunk_bytes = ((sample_rate as usize / 2) * 4).max(4096);
    let mut buffers: Vec<AudioQueueBufferRef> = Vec::new();
    for chunk in pcm.chunks(chunk_bytes / 2) {
        let mut buffer: AudioQueueBufferRef = core::ptr::null_mut();
        let status = coreaudio_sys::AudioQueueAllocateBuffer(queue, chunk_bytes as u32, &mut buffer);
        debug_assert_eq!(status, 0);
        core::ptr::copy_nonoverlapping(chunk.as_ptr(), (*buffer).mAudioData as *mut i16, chunk.len());
        (*buffer).mAudioDataByteSize = (chunk.len() * 2) as u32;
        let status = coreaudio_sys::AudioQueueEnqueueBuffer(queue, buffer, 0, core::ptr::null());
        debug_assert_eq!(status, 0);
        buffers.push(buffer);
    }

    let status = coreaudio_sys::AudioQueueStart(queue, core::ptr::null());
    debug_assert_eq!(status, 0);

    Player {
        queue,
        _buffers: buffers,
    }
}

/// AudioQueue callback (required by `AudioQueueNewOutput`). The whole song is
/// enqueued up-front, so the callback has nothing to refill.
#[cfg(target_os = "macos")]
unsafe extern "C" fn output_callback(
    _user_data: *mut core::ffi::c_void,
    _aq: coreaudio_sys::AudioQueueRef,
    _buffer: coreaudio_sys::AudioQueueBufferRef,
) {
}