use dwt;
use nih_plug::prelude::*;
use std::sync::Arc;

use dwt::{wavelet, Operation, Transform};

/// The maximum number of samples to iterate over at a time.
const MAX_BLOCK_SIZE: usize = 64;
const BUFFER_SIZE: usize = 16384;

fn highpass20hz(sample: f32) -> f32 {
    static STATE: atomic_float::AtomicF32 = atomic_float::AtomicF32::new(0.0f32);
    static CUTOFF_FREQUENCY_HZ: f32 = 20.0f32;
    static GAIN: f32 = CUTOFF_FREQUENCY_HZ / (2.0f32 * std::f32::consts::PI * 44100.0f32);
    let state = STATE.load(std::sync::atomic::Ordering::Acquire);
    let retval = sample - state;
    STATE.store(
        state + (GAIN * retval),
        std::sync::atomic::Ordering::Release,
    );
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

    #[id = "shavevapor0y"]
    pub shavevapor_y0: FloatParam,
    #[id = "shavevapor1y"]
    pub shavevapor_y1: FloatParam,
    #[id = "shavevapor2y"]
    pub shavevapor_y2: FloatParam,
    #[id = "shavevapor3y"]
    pub shavevapor_y3: FloatParam,
    #[id = "shavevapor4y"]
    pub shavevapor_y4: FloatParam,
    #[id = "shavevapor5y"]
    pub shavevapor_y5: FloatParam,
    #[id = "shavevapor6y"]
    pub shavevapor_y6: FloatParam,
    #[id = "shavevapor7y"]
    pub shavevapor_y7: FloatParam,
    #[id = "shavevapor8y"]
    pub shavevapor_y8: FloatParam,
    #[id = "shavevapor9y"]
    pub shavevapor_y9: FloatParam,

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
            mix_dry: FloatParam::new(
                "DryMix",
                0.5f32,
                FloatRange::Linear {
                    min: 0.0f32,
                    max: 1.0f32,
                },
            ),
            mix_slur: FloatParam::new(
                "SlurMix",
                0.5f32,
                FloatRange::Linear {
                    min: 0.0f32,
                    max: 1.0f32,
                },
            ),
            mix_hfj: FloatParam::new(
                "HonkForJesusMix",
                0.5f32,
                FloatRange::Linear {
                    min: 0.0f32,
                    max: 1.0f32,
                },
            ),
            buffer_length: IntParam::new(
                "BufferLength",
                512,
                IntRange::Linear {
                    min: 16,
                    max: BUFFER_SIZE as i32,
                },
            ),
            buffer_acceleration: IntParam::new(
                "HonkForJesusAcceleration",
                10,
                IntRange::Linear {
                    min: -1000,
                    max: 1000,
                },
            ),
            slur_multiplier: FloatParam::new(
                "SlurMultiplier",
                1.2f32,
                FloatRange::Linear {
                    min: 0.1f32,
                    max: 10.0f32,
                },
            ),

            shavevapor_y0: FloatParam::new(
                "ShaveVapor_0y",
                -1.0f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y1: FloatParam::new(
                "ShaveVapor_1y",
                -0.8f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y2: FloatParam::new(
                "ShaveVapor_2y",
                -0.6f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y3: FloatParam::new(
                "ShaveVapor_3y",
                -0.4f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y4: FloatParam::new(
                "ShaveVapor_4y",
                -0.2f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y5: FloatParam::new(
                "ShaveVapor_5y",
                0.2f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y6: FloatParam::new(
                "ShaveVapor_6y",
                0.4f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y7: FloatParam::new(
                "ShaveVapor_7y",
                0.6f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y8: FloatParam::new(
                "ShaveVapor_8y",
                0.8f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),
            shavevapor_y9: FloatParam::new(
                "ShaveVapor_9y",
                1.0f32,
                FloatRange::Linear {
                    min: -1.0f32,
                    max: 1.0f32,
                },
            ),

            gain: FloatParam::new(
                "Gain",
                1.0f32,
                FloatRange::Linear {
                    min: 0.1f32,
                    max: 24.0f32,
                },
            ),
        }
    }
}

pub struct EvenPog {
    params: Arc<EvenPogParams>,
    delay_buffer: [f32; BUFFER_SIZE],
    buffer_write_position: usize,
    buffer_read_position_slur: usize,
    buffer_read_position_slur_exact: f32,
    buffer_read_position_hfj: usize,
    buffer_rate_smps: usize,
}

impl Default for EvenPog {
    fn default() -> Self {
        Self {
            params: Arc::new(EvenPogParams::new()),
            delay_buffer: [0.0f32; BUFFER_SIZE],
            buffer_write_position: 0,
            buffer_read_position_slur: 0,
            buffer_read_position_slur_exact: 0.1f32,
            buffer_read_position_hfj: 0,
            buffer_rate_smps: 22050,
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
                    for sample in channel_samples.into_iter() {
                        self.delay_buffer[self.buffer_write_position
                            % self.params.buffer_length.value() as usize] = *sample;
                        self.buffer_write_position = self.buffer_write_position + 1;

                        let dry = *sample * self.params.mix_dry.value();
                        let s = self.delay_buffer[self.buffer_read_position_slur]
                            * self.params.mix_slur.value()
                            + self.delay_buffer[self.buffer_read_position_hfj]
                                * self.params.mix_hfj.value();
                        let s2 = s * s;
                        let s3 = s2 * s;

                        *sample = self.params.gain.value()
                            * self.waveshape(
                                dry + highpass20hz(s - (s2 / 8.0f32) - (s3 / 16.0f32) + 0.125f32),
                            );
                        self.buffer_read_position_slur_exact = self.buffer_read_position_slur
                            as f32
                            * self.params.slur_multiplier.value();
                        if self.buffer_read_position_slur_exact < 0.1f32 {
                            self.buffer_read_position_slur_exact = 1.0f32;
                        }
                        self.buffer_read_position_slur = self.buffer_read_position_slur_exact
                            as usize
                            % self.params.buffer_length.value() as usize;
                        self.buffer_read_position_hfj = (self.params.buffer_length.value()
                            as usize
                            + self.buffer_read_position_hfj
                            - self.buffer_rate_smps)
                            % self.params.buffer_length.value() as usize;

                        self.buffer_rate_smps = ((self.buffer_rate_smps as i32
                            + self.params.buffer_acceleration.value())
                        .abs()
                            % 22050) as usize;

                        let mut data = vec![42.0; 64];
                        let operation = Operation::Forward;
                        let wavelet = wavelet::Haar::new();
                        let level = (64 as f64).log2() as usize;
                        data.transform(operation, &wavelet, level)
                    }
                }
            }
        }
        ProcessStatus::Normal
    }
}

impl EvenPog {
    fn waveshape(&mut self, sample: f32) -> f32 {
        struct ShaperSet {
            multiplier0: f32,
            multiplier1: f32,
            sample_weighted: f32,
        }

        let s = match sample {
            d if d < -0.8f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y0.value(),
                multiplier1: self.params.shavevapor_y1.value(),
                sample_weighted: (d + 0.8f32 * 5.0f32),
            },
            d if d < -0.8f32 && d > -0.6f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y1.value(),
                multiplier1: self.params.shavevapor_y2.value(),
                sample_weighted: (d + 0.6f32 * 5.0f32),
            },
            d if d < -0.6f32 && d > -0.4f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y2.value(),
                multiplier1: self.params.shavevapor_y3.value(),
                sample_weighted: (d + 0.4f32 * 5.0f32),
            },
            d if d < -0.4f32 && d > -0.2f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y3.value(),
                multiplier1: self.params.shavevapor_y4.value(),
                sample_weighted: (d + 0.2f32 * 5.0f32),
            },
            d if d < -0.2f32 && d > -0.1f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y4.value(),
                multiplier1: 0.0f32,
                sample_weighted: (d * 5.0f32),
            },
            d if d > 0.1f32 && d < 0.2f32 => ShaperSet {
                multiplier0: 0.0f32,
                multiplier1: self.params.shavevapor_y5.value(),
                sample_weighted: (d * 5.0f32),
            },
            d if d > 0.2f32 && d < 0.4f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y5.value(),
                multiplier1: self.params.shavevapor_y6.value(),
                sample_weighted: (d - 0.2f32 * 5.0f32),
            },
            d if d > 0.4f32 && d < 0.6f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y6.value(),
                multiplier1: self.params.shavevapor_y7.value(),
                sample_weighted: (d - 0.4f32 * 5.0f32),
            },
            d if d > 0.6f32 && d < 0.8f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y7.value(),
                multiplier1: self.params.shavevapor_y8.value(),
                sample_weighted: (d - 0.6f32 * 5.0f32),
            },
            d if d > 0.8f32 => ShaperSet {
                multiplier0: self.params.shavevapor_y8.value(),
                multiplier1: self.params.shavevapor_y9.value(),
                sample_weighted: (d - 0.8f32 * 5.0f32),
            },
            _ => ShaperSet {
                multiplier0: 1.0f32,
                multiplier1: 1.0f32,
                sample_weighted: sample,
            },
        };
        s.multiplier0 * s.sample_weighted + ((1.0f32 - s.sample_weighted) * s.multiplier1)
    }
}

impl ClapPlugin for EvenPog {
    const CLAP_ID: &'static str = "volki9000.EvenPog";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(
        "An EHX POG2 and the OG Eventide Pitch Shifter had a love child and it's been circuit-bent",
    );
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
    const VST3_CATEGORIES: &'static str = "Fx|Distortion";
}

nih_export_clap!(EvenPog);
nih_export_vst3!(EvenPog);
