use std::sync::Arc;
use nih_plug::prelude::*;

/// The maximum number of samples to iterate over at a time.
const MAX_BLOCK_SIZE: usize = 64;
const BUFFER_SIZE: usize = 16384;

fn highpass20hz(sample: f32) -> f32 {
    static STATE: atomic_float::AtomicF32 = atomic_float::AtomicF32::new(0.0f32);
    static CUTOFF_FREQUENCY_HZ: f32 = 20.0f32;
    static GAIN: f32 = CUTOFF_FREQUENCY_HZ / (2.0f32 * std::f32::consts::PI * 44100.0f32);
    let state = STATE.load(std::sync::atomic::Ordering::Acquire);
    let retval = sample - state;
    STATE.store(state + (GAIN * retval), std::sync::atomic::Ordering::Release);
    retval
}

#[derive(Params)]
pub struct EvenPogParams {
    /// This plugin really doesn't need its own bypass parameter, but it's still useful to have a
    /// dedicated one so it can be shown in the GUI. This is linked to the host's bypass if the host
    /// supports it.
    #[id = "bypass"]
    pub bypass: BoolParam,
    #[id = "drymix"]
    pub mix_dry: FloatParam,
    #[id = "slurmix"]
    pub mix_slur: FloatParam,
    #[id = "honkforjesusmix"]
    pub mix_hfj: FloatParam,
    #[id = "bufferlength"]
    pub buffer_length: IntParam,
    #[id = "honkforjesusrate"]
    pub buffer_acceleration: IntParam,
    #[id = "slurrate"]
    pub slur_multiplier: FloatParam,

    #[id = "shavevapor0x"]
    pub shavevapor_x0: FloatParam,
    #[id = "shavevapor1x"]
    pub shavevapor_x1: FloatParam,
    #[id = "shavevapor2x"]
    pub shavevapor_x2: FloatParam,
    #[id = "shavevapor3x"]
    pub shavevapor_x3: FloatParam,
    #[id = "shavevapor4x"]
    pub shavevapor_x4: FloatParam,
    #[id = "shavevapor5x"]
    pub shavevapor_x5: FloatParam,
    #[id = "shavevapor6x"]
    pub shavevapor_x6: FloatParam,
    #[id = "shavevapor7x"]
    pub shavevapor_x7: FloatParam,
    #[id = "shavevapor8x"]
    pub shavevapor_x8: FloatParam,
    #[id = "shavevapor9x"]
    pub shavevapor_x9: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,
}

impl EvenPogParams {
    pub fn new() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", false)
                .with_value_to_string(formatters::v2s_bool_bypass())
                .with_string_to_value(formatters::s2v_bool_bypass())
                .make_bypass(),
            mix_dry: FloatParam::new("DryMix", 0.5f32, FloatRange::Linear { min: 0.0f32, max: 1.0f32 }),
            mix_slur: FloatParam::new("SlurMix", 0.5f32, FloatRange::Linear { min: 0.0f32, max: 1.0f32 }),
            mix_hfj: FloatParam::new("HonkForJesusMix", 0.5f32, FloatRange::Linear { min: 0.0f32, max: 1.0f32 }),
            buffer_length: IntParam::new("BufferLength", 512, IntRange::Linear { min: 16, max: BUFFER_SIZE as i32 }),
            buffer_acceleration: IntParam::new("HonkForJesusAcceleration", 10, IntRange::Linear {
                min: -1000,
                max: 1000
            }),
            slur_multiplier: FloatParam::new("SlurMultiplier", 1.2f32, FloatRange::Linear {
                min: 0.1f32,
                max: 10.0f32
            }),

            shavevapor_x0: FloatParam::new("ShaveVapor_0x", -1.0f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x1: FloatParam::new("ShaveVapor_1x", -0.8f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x2: FloatParam::new("ShaveVapor_2x", -0.6f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x3: FloatParam::new("ShaveVapor_3x", -0.4f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x4: FloatParam::new("ShaveVapor_4x", -0.2f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x5: FloatParam::new("ShaveVapor_5x", 0.2f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x6: FloatParam::new("ShaveVapor_6x", 0.4f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x7: FloatParam::new("ShaveVapor_7x", 0.6f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x8: FloatParam::new("ShaveVapor_8x", 0.8f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),
            shavevapor_x9: FloatParam::new("ShaveVapor_9x", 1.0f32, FloatRange::Linear {
                min: -1.0f32,
                max: 1.0f32
            }),

            gain: FloatParam::new("Gain", 1.0f32, FloatRange::Linear {
                min: 0.1f32,
                max: 24.0f32
            })
        }
    }
}

pub struct EvenPog {
    params : Arc<EvenPogParams>,
    delay_buffer: [f32; BUFFER_SIZE],
    buffer_write_position: usize,
    buffer_read_position_slur: usize,
    buffer_read_position_slur_exact: f32,
    buffer_read_position_hfj: usize,
    buffer_rate_smps: usize
}

impl Default for EvenPog {
    fn default() -> Self {
        Self {
            params: Arc::new(EvenPogParams::new(
            )),
            delay_buffer: [0.0f32;BUFFER_SIZE],
            buffer_write_position: 0,
            buffer_read_position_slur: 0,
            buffer_read_position_slur_exact: 0.1f32,
            buffer_read_position_hfj: 0,
            buffer_rate_smps: 22050
        }
    }
}

impl Plugin for EvenPog {
    const NAME: &'static str = "EvenPog";
    const VENDOR: &'static str = "volki9000";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "https://github.com/volki9000";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        Arc::new(self.params.clone())
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // The bypass parameter controls a smoother so we can crossfade between the dry and the wet
        // signals as needed
        if !self.params.bypass.value() {
            for (_, mut block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
                for channel_samples in block.iter_samples() {
                    for sample in channel_samples.into_iter()
                    {
                        self.delay_buffer[self.buffer_write_position % self.params.buffer_length.value() as usize] = *sample;
                        self.buffer_write_position = self.buffer_write_position + 1;

                        let s = self.delay_buffer[self.buffer_read_position_slur] * self.params.mix_slur.value() + self.delay_buffer[self.buffer_read_position_hfj] * self.params.mix_hfj.value();
                        let s2 = s * s;
                        let s3 = s2 * s;

                        *sample = s * self.params.mix_dry.value() + self.params.gain.value() * self.waveshape(highpass20hz(s - (s2/8.0f32) - (s3/16.0f32) + 0.125f32));
                        self.buffer_read_position_slur_exact = self.buffer_read_position_slur as f32 * self.params.slur_multiplier.value();
                        if self.buffer_read_position_slur_exact < 0.1f32 {
                            self.buffer_read_position_slur_exact = 1.0f32;
                        }
                        self.buffer_read_position_slur = self.buffer_read_position_slur_exact as usize % self.params.buffer_length.value() as usize;
                        self.buffer_read_position_hfj = (self.params.buffer_length.value() as usize + self.buffer_read_position_hfj - self.buffer_rate_smps)
                            % self.params.buffer_length.value() as usize;

                        self.buffer_rate_smps = ((self.buffer_rate_smps as i32 + self.params.buffer_acceleration.value()).abs() % 22050) as usize; 
                    }
                }
            }
        }
        ProcessStatus::Normal
    }
}

impl EvenPog {
    fn waveshape(
        &mut self,
        sample: f32,
    ) -> f32 {
        if sample > 0.1f32 {
            if sample > 0.6f32 {
                if sample > 0.8f32 {
                    return sample * self.params.shavevapor_x9.value();
                }
                else {
                    return sample * self.params.shavevapor_x8.value();
                }
            }
            else if sample > 0.4f32 {
                return sample * self.params.shavevapor_x7.value();
            }
            else {
                if sample > -0.2f32 {
                    return sample * self.params.shavevapor_x5.value();
                }
                else {
                    return sample * self.params.shavevapor_x6.value();
                }
            }
        }
        else if sample < -0.1f32 {
            if sample < -0.6f32 {
                if sample < -0.8f32 {
                    return sample * self.params.shavevapor_x0.value();
                }
                else {
                    return sample * self.params.shavevapor_x1.value();
                }
            }
            else if sample < -0.4f32 {
                if sample < -0.2f32 {
                    return sample * self.params.shavevapor_x4.value();
                }
                else {
                    return sample * self.params.shavevapor_x3.value();
                }
            }
            else {
                return sample * self.params.shavevapor_x2.value();
            }
        }
        return sample;
    }
}

impl ClapPlugin for EvenPog {
    const CLAP_ID: &'static str = "volki9000.EvenPog";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("An EHX POG2 and the OG Eventide Pitch Shifter had a love child and it's been circuit-bent");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Filter,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for EvenPog {
    const VST3_CLASS_ID: [u8; 16] = *b"EvenPogPlugRvdH.";
    const VST3_CATEGORIES: &'static str = "Fx|Filter";
}

nih_export_clap!(EvenPog);
nih_export_vst3!(EvenPog);
