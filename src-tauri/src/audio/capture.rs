use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 音频捕获管理器
pub struct AudioCapture {
    /// 捕获的音频数据缓冲区
    buffer: Arc<Mutex<Vec<f32>>>,
    /// 当前状态
    state: Arc<Mutex<super::CaptureState>>,
    /// 音频流（持有以保持活跃）
    _stream: Option<cpal::Stream>,
    /// 启动时间
    start_time: Instant,
    /// 系统采样率（捕获时的原始采样率）
    system_sample_rate: u32,
    /// 目标采样率（16kHz for Whisper）
    target_sample_rate: u32,
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

/// 获取系统默认音频输出设备名称
pub fn get_default_output_device_name() -> Result<String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("未找到默认音频输出设备")?;
    device.name().context("无法获取设备名称")
}

impl AudioCapture {
    /// 创建新的音频捕获实例
    pub fn new(target_sample_rate: u32) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            state: Arc::new(Mutex::new(super::CaptureState::Idle)),
            _stream: None,
            start_time: Instant::now(),
            system_sample_rate: 0,
            target_sample_rate,
        }
    }

    /// 开始捕获系统音频 (WASAPI Loopback)
    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .context("未找到默认音频输出设备")?;

        tracing::info!("音频设备: {}", device.name().unwrap_or_default());

        let supported_config = device
            .default_output_config()
            .context("无法获取音频输出配置")?;

        self.system_sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels() as usize;
        let sample_format = supported_config.sample_format();

        tracing::info!(
            "系统音频: {}Hz, {} 壁道, {:?}",
            self.system_sample_rate,
            channels,
            sample_format
        );

        let sample_rate = self.system_sample_rate;
        let buffer = self.buffer.clone();
        let state = self.state.clone();
        let target_rate = self.target_sample_rate;
        let config: cpal::StreamConfig = supported_config.into();

        // 根据采样格式构建对应的输入流
        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        process_audio(data, channels, sample_rate, target_rate, &buffer, &state);
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
                        process_audio(&f32_data, channels, sample_rate, target_rate, &buffer, &state);
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
                        process_audio(&f32_data, channels, sample_rate, target_rate, &buffer, &state);
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

        *self.state.lock().unwrap() = super::CaptureState::Running;
        self._stream = Some(stream);
        self.start_time = Instant::now();

        tracing::info!("音频捕获已启动");
        Ok(())
    }

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
