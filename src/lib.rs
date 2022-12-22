use std::sync::Arc;

use nih_plug::prelude::*;

/// The maximum number of samples to iterate over at a time.
const MAX_BLOCK_SIZE: usize = 64;

#[derive(Params)]
pub struct EvenPogParams {
    /// This plugin really doesn't need its own bypass parameter, but it's still useful to have a
    /// dedicated one so it can be shown in the GUI. This is linked to the host's bypass if the host
    /// supports it.
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl EvenPogParams {
    pub fn new() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", false)
                .with_value_to_string(formatters::v2s_bool_bypass())
                .with_string_to_value(formatters::s2v_bool_bypass())
                .make_bypass()
        }
    }
}

pub struct EvenPog {
    params : Arc<EvenPogParams>,
}

impl Default for EvenPog {
    fn default() -> Self {
        Self{
        params: Arc::new(EvenPogParams::new(
        ))
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

    // fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    //     editor::create(
    //         editor::Data {
    //             params: self.params.clone(),

    //             sample_rate: self.sample_rate.clone(),
    //             spectrum: self.spectrum_output.clone(),
    //             safe_mode_clamper: SafeModeClamper::new(self.params.clone()),
    //         },
    //         self.params.editor_state.clone(),
    //     )
    // }

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
                        let s = *sample;
                        let s2 = s * s;
                        let s3 = s2 * s;
                        *sample = s - (s2/8.0f32) - (s3/16.0f32) + 0.125f32;
                    }
                }
            }

            // for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            //     for _ in 0..MAX_BLOCK_SIZE {
            //         let stereo_slice = [
            //             block.get_mut(0).into_iter().next(),
            //             block.get_mut(1).into_iter().next(),
            //         ];
            //         let l = stereo_slice[0].unwrap()[0];
            //         let r = stereo_slice[1].unwrap()[0];
            //         let l2 = stereo_slice[0].unwrap()[0] * stereo_slice[0].unwrap()[0];
            //         let r2 = stereo_slice[1].unwrap()[0] * stereo_slice[1].unwrap()[0];
            //         let l3 = l2 * stereo_slice[0].unwrap()[0];
            //         let r3 = r2 * stereo_slice[1].unwrap()[0];
            //         *(stereo_slice[0].unwrap()) = l - (l2/8.0f32) - (l3/16.0f32) + 0.125f32;
            //         *(stereo_slice[1].unwrap()) = r - (r2/8.0f32) - (r3/16.0f32) + 0.125f32;    
            //     }
            // }
            // We'll iterate in blocks to make the blending relatively cheap without having to
            // duplicate code or add a bunch of per-sample conditionals
            // for (_, mut block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            //     // We'll blend this with the dry signal as needed
            //     let mut dry = [<(f32, f32)>::default(); MAX_BLOCK_SIZE];
            //     let mut wet = [<(f32, f32)>::default(); MAX_BLOCK_SIZE];
            //     for (input_samples, (dry_samples, wet_samples)) in block
            //         .iter_samples()
            //         .zip(std::iter::zip(dry.iter_mut(), wet.iter_mut()))
            //     {
            //         *dry_samples = *input_samples;
            //         *wet_samples = *dry_samples;

            //         wet_samples.0 = wet_samples.0 - (wet_samples.0*wet_samples.0/8.0f32) - (wet_samples.0*wet_samples.0*wet_samples.0/16.0f32) + 0.125f32;
            //         wet_samples.1 = wet_samples.1 - (wet_samples.1*wet_samples.1/8.0f32) - (wet_samples.1*wet_samples.1*wet_samples.1/16.0f32) + 0.125f32;
            //     }

            //     for (mut channel_samples, (dry_samples, wet_samples)) in block
            //         .iter_samples()
            //         .zip(std::iter::zip(dry.iter_mut(), wet.iter_mut()))
            //     {
            //         // We'll do an equal-power fade
            //         let dry_t = dry_t_squared.sqrt();
            //         let wet_t = (1.0f32 - dry_t_squared).sqrt();

            //         let dry_weightedL = dry_samples.0/dry_t;
            //         let dry_weightedR = dry_samples.1/dry_t;
            //         let wet_weightedL = wet_samples.0/wet_t;
            //         let wet_weightedR = wet_samples.1/wet_t;

            //         channel_samples = dry_weightedL + wet_weightedL;
            //         //TODO What about R?
            //     }
            // }
        }
        ProcessStatus::Normal
    }
}

impl EvenPog {
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
