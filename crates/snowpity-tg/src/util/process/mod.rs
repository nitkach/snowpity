use crate::prelude::*;
use crate::{fatal, Result};
use std::process::Stdio;

pub(crate) async fn run<'a>(program: &str, args: &[impl AsRef<str>]) -> Result<Vec<u8>> {
    let display_args = shlex::join(args.iter().map(AsRef::as_ref));
    let display_cmd = format!("{program} {display_args}");
    debug!(
        cmd = %display_cmd,
        "Running program"
    );

    let output = tokio::process::Command::new(program)
        .args(args.iter().map(AsRef::as_ref))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .fatal_ctx(|| format!("Invocation failed. Command:\n`{display_cmd}`"))?;

    let status = output.status;

    if !status.success() {
        let stderr = match String::from_utf8(output.stderr) {
            Ok(ok) => ok,
            Err(err) => format!(
                "stderr is not UTF-8 encoding\n{err}\nBytes len: {}",
                err.as_bytes().len()
            ),
        };

        let err = fatal!(
            "{program} invocation failed with status {status}. Command:\n{display_cmd}\nStderr:\n{}",
            stderr
        );

        return Err(err);
    }

    Ok(output.stdout)
}

async fn run_utf8(program: &str, args: &[impl AsRef<str>]) -> Result<String> {
    let bytes = run(program, args).await?;
    std::str::from_utf8(&bytes)
        .fatal_ctx(|| {
            let args = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
            format!(
                "Bad output (invalid UTF-8).\n\
                Program: {program}.\n\
                Args: {args:?}.\n\
                Output: {bytes:?}.\n"
            )
        })
        .map(ToOwned::to_owned)
}

pub async fn run_json<T: serde::de::DeserializeOwned>(
    program: &str,
    args: &[impl AsRef<str>],
) -> Result<T> {
    let output = run_utf8(program, args).await?;
    serde_json::from_str(&output).fatal_ctx(|| {
        let args = args.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        format!(
            "Bad output (invalid JSON).\n\
            Program: {program}.\n\
            Args: {args:?}.\n\
            Output: {output}.\n"
        )
    })
}
