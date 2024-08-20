use hound::WavReader;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use whisper_rs::{FullParams, SamplingStrategy, WhisperState};

pub fn to_outputfile_name(tmpfile_name: String) -> String {
    format!("{}_output.wav", tmpfile_name)
}

pub async fn process(
    tmpfile_name: String,
    whisper_state: Arc<Mutex<WhisperState>>,
) -> Result<String, impl std::error::Error> {
    let output_path = to_outputfile_name(tmpfile_name.clone());
    let input_path = tmpfile_name;

    let _output = match Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg(output_path.clone())
        .output()
        .await
    {
        Ok(ffmpeg_out) => ffmpeg_out,
        Err(e) => {
            return Result::Err(e);
        }
    };

    let transcribed_text = transcribe_audio(whisper_state, &output_path).await.unwrap();
    return Result::Ok(transcribed_text);
}

#[derive(Debug, Clone)]
struct TranscribeError<'a> {
    errormsg: &'a str,
}

impl<'a> std::error::Error for TranscribeError<'a> {}

impl<'a> std::fmt::Display for TranscribeError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.errormsg)
    }
}

async fn transcribe_audio(
    whisper_state: Arc<Mutex<WhisperState>>,
    filepath: &String,
) -> Result<String, TranscribeError> {
    // Create a params object for running the model.
    // The number of past samples to consider defaults to 0.

    let mut output_buffer = Vec::new();
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    // Edit params as needed.
    // Set the number of threads to use to 1.
    // Enable translation.
    params.set_translate(true);
    // Set the language to translate to to English.
    params.set_language(Some("en"));
    // Disable anything that prints to stdout.
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let reader = WavReader::open(filepath).unwrap();
    // Convert the audio to floating point samples.
    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .map(|x| x.expect("Invalid sample"))
        .collect();
    let mut audio = vec![0.0f32; samples.len().try_into().unwrap()];
    whisper_rs::convert_integer_to_float_audio(&samples, &mut audio).expect("Conversion error");

    // Open the audio file.
    let reader = hound::WavReader::open(filepath).expect("failed to open file");
    #[allow(unused_variables)]
    let hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        ..
    } = reader.spec();

    // Convert the audio to floating point samples.
    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .map(|x| x.expect("Invalid sample"))
        .collect();
    let mut audio = vec![0.0f32; samples.len().try_into().unwrap()];
    whisper_rs::convert_integer_to_float_audio(&samples, &mut audio).expect("Conversion error");

    // Convert audio to 16KHz mono f32 samples, as required by the model.
    // These utilities are provided for convenience, but can be replaced with custom conversion logic.
    // SIMD variants of these functions are also available on nightly Rust (see the docs).
    if channels == 2 {
        audio = whisper_rs::convert_stereo_to_mono_audio(&audio).expect("Conversion error");
    } else if channels != 1 {
        return Result::Err(TranscribeError {
            errormsg: "2 channels are not allowed",
        });
    }

    if sample_rate != 16000 {
        return Result::Err(TranscribeError {
            errormsg: "Sample rate should be 16000",
        });
    }

    // Run the model.

    let mut whisper_state_lock = whisper_state.lock().unwrap();
    whisper_state_lock
        .full(params, &audio[..])
        .expect("failed to run model");

    // Iterate through the segments of the transcript.
    let num_segments = whisper_state_lock
        .full_n_segments()
        .expect("failed to get number of segments");
    for i in 0..num_segments {
        // Get the transcribed text and timestamps for the current segment.
        let segment = whisper_state_lock
            .full_get_segment_text(i)
            .expect("failed to get segment");

        // Segments are not being used right now due to the sake of simplicity
        //        let start_timestamp = state
        //            .full_get_segment_t0(i)
        //            .expect("failed to get start timestamp");
        //        let end_timestamp = state
        //            .full_get_segment_t1(i)
        //            .expect("failed to get end timestamp");

        let _first_token_dtw_ts = if let Ok(token_count) = whisper_state_lock.full_n_tokens(i) {
            if token_count > 0 {
                if let Ok(token_data) = whisper_state_lock.full_get_token_data(i, 0) {
                    token_data.t_dtw
                } else {
                    -1i64
                }
            } else {
                -1i64
            }
        } else {
            -1i64
        };

        // Format the segment information as a string.
        let line = format!("{}\n", segment);
        // Write the segment information to the output buffer.
        output_buffer.extend_from_slice(line.as_bytes());
    }
    Ok(String::from_utf8(output_buffer).unwrap())
}
