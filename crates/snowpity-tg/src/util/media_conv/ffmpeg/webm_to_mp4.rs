use super::ArgsBuilder;
use super::{common_args, ffmpeg};
use crate::prelude::*;
use crate::Result;
use std::path::Path;
use tempfile::TempPath;

#[instrument(skip_all, fields(input = %input.as_ref().display()))]
pub(crate) async fn webm_to_mp4(input: impl AsRef<Path>) -> Result<TempPath> {
    let input = input.as_ref();

    let output = std::env::temp_dir().join(format!("{}.mp4", nanoid::nanoid!()));
    let log_message = format!("Converting Webm to mp4 with output at {output:?}");

    let output = tempfile::TempPath::from_path(output);

    let input_arg = input.to_string_lossy();
    let output_arg = output.to_string_lossy();

    // Rustfmt is doing a bad job of condensing this code, so let's disable it
    #[rustfmt::skip]
    let args = [
        // Force Webm format of the input
        "-f",
        "webm",

        // Set input path
        "-i",
        &input_arg,
    ]
    .map_collect(str::to_owned);

    let args = ArgsBuilder::builder()
        .args(args)
        .args(common_args())
        .arg(output_arg)
        .build()
        .args;

    ffmpeg(&args).with_duration_log(&log_message).await?;

    Ok(output)
}
