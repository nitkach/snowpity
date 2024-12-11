use buildstructor::buildstructor;

mod gif_to_mp4;
mod webm_to_mp4;

pub(crate) use gif_to_mp4::*;
pub(crate) use webm_to_mp4::*;

use crate::{util::IntoIteratorExt, Result};

fn common_args() -> Vec<String> {
    // This is inspired a bit by this code:
    // https://github.com/philomena-dev/philomena/blob/master/lib/philomena/processors/gif.ex#L96

    // Rustfmt is doing a bad job of condensing this code, so let's disable it
    #[rustfmt::skip]
    let args = [
        // Overwrite output file without interactive confirmation
        "-y",

        // Preserve the original FPS
        "-fps_mode",
        "passthrough",

        // MP4 videos using H.264 need to have a dimensions that are divisible by 2.
        // This option ensures that's the case.
        "-vf",
        // TODO(Havoc) keep it while in debug
        &std::env::var("FFMPEG_SCALE").unwrap(),

        "-c:v",
        "libx264",

        // Experimentally determined it to be the most optimal one for our server class
        "-preset",
        "faster",

        // Some video players require this setting, but Telegram doesn't seem to need
        // this. So let's not enable it and see where this gets us
        "-pix_fmt",
        "yuv420p",

        // It's the default value, but it's better to be explicit
        "-crf",
        "23",

        // Fast start is needed to make the video playable before it's fully downloaded
        "-movflags",
        "+faststart",
    ];

    dbg!(args);

    args.map_collect(str::to_owned)
}

struct ArgsBuilder {
    args: Vec<String>,
}

#[buildstructor]
impl ArgsBuilder {
    #[builder]
    fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

async fn ffmpeg<'a>(args: &[impl AsRef<str>]) -> Result<Vec<u8>> {
    // let args = args.iter().map(|arg| arg.as_ref()).collect();

    crate::util::process::run("ffmpeg", args).await
}
