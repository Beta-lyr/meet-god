/// 简易 VAD（Voice Activity Detection）实现
/// 基于音频能量阈值判断是否有语音活动

/// VAD 检测器
pub struct VadDetector {
    /// 能量阈值
    threshold: f32,
    /// 采样率（预留）
    #[allow(dead_code)]
    sample_rate: u32,
    /// 每帧大小 (samples)
    frame_size: usize,
}

/// VAD 检测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadResult {
    /// 有语音
    Speech,
    /// 静音
    Silence,
}

impl VadDetector {
    pub fn new(threshold: f32, sample_rate: u32) -> Self {
        // 使用 30ms 帧大小
        let frame_size = (sample_rate as f32 * 0.03) as usize;
        Self {
            threshold,
            sample_rate,
            frame_size,
        }
    }

    /// 检测一段音频是否包含语音
    /// 返回 RMS 能量值和检测结果
    pub fn detect(&self, samples: &[f32]) -> (f32, VadResult) {
        if samples.is_empty() {
            return (0.0, VadResult::Silence);
        }

        // 计算 RMS 能量
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum / samples.len() as f32).sqrt();

        let result = if rms > self.threshold {
            VadResult::Speech
        } else {
            VadResult::Silence
        };

        (rms, result)
    }

    /// 按帧检测，返回每帧的结果
    pub fn detect_frames(&self, samples: &[f32]) -> Vec<(f32, VadResult)> {
        samples
            .chunks(self.frame_size)
            .map(|chunk| self.detect(chunk))
            .collect()
    }

    /// 判断整段音频中语音占比是否超过阈值
    pub fn has_speech(&self, samples: &[f32], speech_ratio_threshold: f32) -> bool {
        let frames = self.detect_frames(samples);
        if frames.is_empty() {
            return false;
        }
        let speech_count = frames.iter().filter(|(_, r)| *r == VadResult::Speech).count();
        (speech_count as f32 / frames.len() as f32) > speech_ratio_threshold
    }

    #[allow(dead_code)]
    pub fn frame_size(&self) -> usize {
        self.frame_size
    }
}
