use std::sync::{Arc, Mutex};
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperState};

fn create_whisper_state() -> Arc<Mutex<WhisperState>> {
    let mut context_param = WhisperContextParameters::default();

    context_param.dtw_parameters.mode = whisper_rs::DtwMode::ModelPreset {
        model_preset: whisper_rs::DtwModelPreset::LargeV3,
    };

    // Enable DTW token level timestamp for unknown model by providing custom aheads
    // see details https://github.com/ggerganov/whisper.cpp/pull/1485#discussion_r1519681143
    // values corresponds to ggml-base.en.bin, result will be the same as with DtwModelPreset::BaseEn
    let custom_aheads = [
        (3, 1),
        (4, 2),
        (4, 3),
        (4, 7),
        (5, 1),
        (5, 2),
        (5, 4),
        (5, 6),
    ]
    .map(|(n_text_layer, n_head)| whisper_rs::DtwAhead {
        n_text_layer,
        n_head,
    });
    context_param.dtw_parameters.mode = whisper_rs::DtwMode::Custom {
        aheads: &custom_aheads,
    };

    let ctx = WhisperContext::new_with_params("./models/ggml-large.bin", context_param)
        .expect("failed to load model");
    // Create a state
    let state = ctx.create_state().expect("failed to create key");
    let safe_state = Arc::new(Mutex::new(state));

    return safe_state;
}

pub struct WhisperPool {

    // only one state for now
    states: [Arc<Mutex<WhisperState>>; 1],
}



impl WhisperPool {
    pub fn new_pool() -> Self {
        Self {
            states: [create_whisper_state()],
        }
    }

    pub fn get_state(&self) -> Arc<Mutex<WhisperState>> {
        
        return self.states[0].clone();

    }
}
