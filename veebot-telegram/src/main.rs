use futures::{prelude::*, TryFutureExt};
use std::panic::AssertUnwindSafe;
use std::process::ExitCode;
use tracing::{error, info, warn};
use veebot_telegram::util::tracing_err;

#[tokio::main]
async fn main() -> ExitCode {
    if dotenv::dotenv().is_err() {
        eprintln!("Dotenv config was not found, ignoring this...")
    }

    let logging_task = veebot_telegram::LoggingConfig::load_or_panic().init_logging();

    let main_fut = AssertUnwindSafe(async {
        let result = try_main().await;

        result.map(|()| ExitCode::SUCCESS).unwrap_or_else(|err| {
            error!(err = tracing_err(&err), "Exitting with an error...");
            ExitCode::FAILURE
        })
    })
    .catch_unwind()
    .unwrap_or_else(|_| {
        error!("Exitting due to a panic...");
        ExitCode::FAILURE
    });

    let exit_code = if !cfg!(debug_assertions) {
        main_fut.await
    } else {
        // Don't wait for teloxide's shutdown logic when cancelling in debug mode.
        // That takes a lot of time for some reason:
        // https://github.com/teloxide/teloxide/issues/711
        tokio::select! {
            exit_code = main_fut => {
                info!("Main task has finished, exiting...");
                exit_code
            }
            result = tokio::signal::ctrl_c() => {
                if let Err(err) = result {
                    warn!(err = tracing_err(&err), "Failed to wait for Ctrl+C, exiting...");
                } else {
                    info!("Ctrl+C received, exiting forcefully...");
                }
                ExitCode::SUCCESS
            }
        }
    };

    // Let's await for three seconds heuristically to let the logging task
    // flush some data to the logging backend.
    //
    // Unfortunately, we can't guarantee the flush happens because no such
    // API exists in `tracing_loki`: https://github.com/hrxi/tracing-loki/issues/9
    if !cfg!(debug_assertions) {
        info!("Waiting for the logging task to finish nicely...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    } else {
        info!(
            "Forcefully shutting down logging task \
            (some logs may not be pushed to the backend)..."
        );
    }

    logging_task.abort();

    eprintln!("Stopped logging task: {:?}", logging_task.await);

    exit_code
}

async fn try_main() -> veebot_telegram::Result {
    let config = veebot_telegram::Config::load_or_panic();

    veebot_telegram::run(config).await?;

    Ok(())
}
