use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 获取系统默认音频输出设备名称
pub fn get_default_output_device_name() -> Result<String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("未找到默认音频输出设备")?;
    device.name().context("无法获取设备名称")
}

/// 获取系统默认音频输出设备的采样率
pub fn get_default_output_sample_rate() -> Result<u32> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("未找到默认音频输出设备")?;
    let config = device
        .default_output_config()
        .context("无法获取音频输出配置")?;
    Ok(config.sample_rate().0)
}

/// 包装 cpal::Stream 使其可以跨线程传递
/// cpal::Stream 包含 *mut () 指针，标记为 !Send
/// 但 cpal 的流实际上是线程安全的（回调在 cpal 内部线程执行）
#[allow(dead_code)]
struct StreamWrapper(cpal::Stream);
unsafe impl Send for StreamWrapper {}

/// 音频捕获状态
pub struct AudioCaptureHandle {
    /// 捕获的音频数据缓冲区
    pub buffer: Arc<Mutex<Vec<f32>>>,
    /// 当前状态
    pub state: Arc<Mutex<super::CaptureState>>,
    /// 启动时间
    start_time: Instant,
}

/// 启动音频捕获（在独立线程中运行，因为 cpal::Stream 不是 Send）
pub fn start_capture(
    target_sample_rate: u32,
    buffer: Arc<Mutex<Vec<f32>>>,
    state: Arc<Mutex<super::CaptureState>>,
) -> Result<AudioCaptureHandle> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .context("未找到默认音频输出设备")?;

    tracing::info!("音频设备: {}", device.name().unwrap_or_default());

    let supported_config = device
        .default_output_config()
        .context("无法获取音频输出配置")?;

    let system_sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels() as usize;
    let sample_format = supported_config.sample_format();

    tracing::info!(
        "系统音频: {}Hz, {} 壁道, {:?}",
        system_sample_rate,
        channels,
        sample_format
    );

    let config: cpal::StreamConfig = supported_config.into();
    let buf_clone = buffer.clone();
    let state_clone = state.clone();
    let callback_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let callback_count_clone = callback_count.clone();

    // 根据采样格式构建对应的输入流
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let count = callback_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if count < 5 || count % 100 == 0 {
                        let max_val = data.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
                        tracing::info!("音频回调 #{}: {} 采样, 最大振幅: {:.6}", count, data.len(), max_val);
                    }
                    process_audio(data, channels, system_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("音频捕获错误: {}", err),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    process_audio(&f32_data, channels, system_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("音频捕获错误: {}", err),
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                    process_audio(&f32_data, channels, system_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("音频捕获错误: {}", err),
                None,
            )
        }
        _ => {
            return Err(anyhow::anyhow!("不支持的音频格式: {:?}", sample_format));
        }
    }
    .context("构建音频流失败")?;

    stream.play().context("启动音频流失败")?;

    // cpal::Stream 不是 Send，但它实际上是线程安全的
    // 将其包装在 Send 类型中，然后移到独立线程持有
    let wrapper = StreamWrapper(stream);
    std::thread::spawn(move || {
        let _w = wrapper;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });

    *state.lock().unwrap() = super::CaptureState::Running;

    tracing::info!("音频捕获已启动");

    Ok(AudioCaptureHandle {
        buffer,
        state,
        start_time: Instant::now(),
    })
}

impl AudioCaptureHandle {
    /// 暂停捕获（静音）
    pub fn mute(&self) {
        *self.state.lock().unwrap() = super::CaptureState::Muted;
    }

    /// 恢复捕获
    pub fn unmute(&self) {
        *self.state.lock().unwrap() = super::CaptureState::Running;
    }

    /// 切换静音状态
    pub fn toggle_mute(&self) -> super::CaptureState {
        let mut state = self.state.lock().unwrap();
        match *state {
            super::CaptureState::Running => {
                *state = super::CaptureState::Muted;
                super::CaptureState::Muted
            }
            super::CaptureState::Muted => {
                *state = super::CaptureState::Running;
                super::CaptureState::Running
            }
            other => other,
        }
    }

    /// 获取并清空缓冲区中的音频数据
    pub fn drain_buffer(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// 获取当前状态
    pub fn state(&self) -> super::CaptureState {
        *self.state.lock().unwrap()
    }

    /// 获取已运行时长 (ms)
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// 启动麦克风捕获（录制用户自己的声音）
pub fn start_microphone_capture(
    target_sample_rate: u32,
    buffer: Arc<Mutex<Vec<f32>>>,
    state: Arc<Mutex<super::CaptureState>>,
) -> Result<AudioCaptureHandle> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .context("未找到默认音频输入设备（麦克风）")?;

    tracing::info!("麦克风设备: {}", device.name().unwrap_or_default());

    let supported_config = device
        .default_input_config()
        .context("无法获取麦克风配置")?;

    let mic_sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels() as usize;
    let sample_format = supported_config.sample_format();

    tracing::info!(
        "麦克风: {}Hz, {} 壁道, {:?}",
        mic_sample_rate,
        channels,
        sample_format
    );

    let config: cpal::StreamConfig = supported_config.into();
    let buf_clone = buffer.clone();
    let state_clone = state.clone();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    process_audio(data, channels, mic_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("麦克风捕获错误: {}", err),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    process_audio(&f32_data, channels, mic_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("麦克风捕获错误: {}", err),
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                    process_audio(&f32_data, channels, mic_sample_rate, target_sample_rate, &buf_clone, &state_clone);
                },
                |err| tracing::error!("麦克风捕获错误: {}", err),
                None,
            )
        }
        _ => {
            return Err(anyhow::anyhow!("不支持的音频格式: {:?}", sample_format));
        }
    }
    .context("构建麦克风音频流失败")?;

    stream.play().context("启动麦克风音频流失败")?;

    let wrapper = StreamWrapper(stream);
    std::thread::spawn(move || {
        let _w = wrapper;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });

    *state.lock().unwrap() = super::CaptureState::Running;
    tracing::info!("麦克风捕获已启动");

    Ok(AudioCaptureHandle {
        buffer,
        state,
        start_time: Instant::now(),
    })
}

/// 处理音频回调：多声道转单声道 + 重采样
fn process_audio(
    data: &[f32],
    channels: usize,
    sample_rate: u32,
    target_rate: u32,
    buffer: &Arc<Mutex<Vec<f32>>>,
    state: &Arc<Mutex<super::CaptureState>>,
) {
    let current_state = state.lock().unwrap();
    if *current_state != super::CaptureState::Running {
        return;
    }
    drop(current_state);

    // 多声道转单声道
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();

    // 重采样
    let resampled = if sample_rate != target_rate {
        resample_linear(&mono, sample_rate, target_rate)
    } else {
        mono
    };

    let mut buf = buffer.lock().unwrap();
    buf.extend_from_slice(&resampled);
}

/// 线性插值重采样
fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.is_empty() || from_rate == 0 || to_rate == 0 {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let sample = if idx + 1 < samples.len() {
            samples[idx] as f64 * (1.0 - frac) + samples[idx + 1] as f64 * frac
        } else if idx < samples.len() {
            samples[idx] as f64
        } else {
            0.0
        };

        output.push(sample as f32);
    }

    output
}
