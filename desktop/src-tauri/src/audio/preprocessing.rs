use crate::error::{AppError, Result};
use std::path::Path;
use std::process::Command;

/// Compress audio for transcription to reduce file size and token usage
/// Converts to MP3 format with optimized settings for speech recognition
pub async fn compress_audio_for_transcription(input_path: &Path, output_path: &Path) -> Result<()> {
    // Use ffmpeg to compress audio
    // Settings optimized for speech:
    // - Format: MP3
    // - Sample rate: 16kHz (sufficient for speech)
    // - Channels: Mono (voice doesn't need stereo)
    // - Bitrate: 32kbps (optimal for voice, reduces file size)

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-ar")
        .arg("16000") // Sample rate: 16kHz
        .arg("-ac")
        .arg("1") // Channels: Mono
        .arg("-b:a")
        .arg("32k") // Bitrate: 32kbps
        .arg("-f")
        .arg("mp3")
        .arg("-y") // Overwrite output file
        .arg(output_path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::Internal(
                    "ffmpeg not found. Please install ffmpeg to use transcription features."
                        .to_string(),
                )
            } else {
                AppError::Io(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!(
            "ffmpeg compression failed: {}",
            stderr
        )));
    }

    Ok(())
}

/// Compress audio from bytes and return compressed bytes
pub async fn compress_audio_bytes(audio_data: &[u8], format: &str) -> Result<Vec<u8>> {
    use std::io::Write;

    // Create temporary input file
    let temp_dir = std::env::temp_dir();
    let input_id = uuid::Uuid::new_v4();
    let input_path = temp_dir.join(format!("input_{}.{}", input_id, format));
    let output_path = temp_dir.join(format!("output_{}.mp3", input_id));

    // Write input data to temp file
    {
        let mut file = std::fs::File::create(&input_path).map_err(|e| AppError::Io(e))?;
        file.write_all(audio_data).map_err(|e| AppError::Io(e))?;
    }

    // Compress audio
    compress_audio_for_transcription(&input_path, &output_path).await?;

    // Read compressed output
    let compressed_data = std::fs::read(&output_path).map_err(|e| AppError::Io(e))?;

    // Clean up temp files
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok(compressed_data)
}
