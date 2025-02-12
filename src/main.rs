#[macro_use]
extern crate tracing;

pub mod entities;
mod router;
pub mod utils;

use std::time::Duration;

use actix_web::{App, HttpServer, middleware as mw};
use clap::{arg, command};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let matches = command!()
        .arg(arg!(-v --verbose "Enable verbose logging"))
        .arg(arg!(--host <VALUE> "Set the server's listening host. Defaults to \"localhost\""))
        .arg(arg!(--port <VALUE> "Set the server's listening port. Defaults to 10100"))
        .after_help("Created by thunder04 <https://github.com/thunder04>")
        .get_matches();

    install_helpers(matches.get_flag("verbose"))?;

    let host = matches.try_get_one::<&str>("host")?.unwrap_or(&"localhost");
    let port = matches.try_get_one::<u16>("port")?.unwrap_or(&10100);

    HttpServer::new(|| {
        let app = App::new().wrap(mw::NormalizePath::new(mw::TrailingSlash::Always));

        #[cfg(debug_assertions)]
        let app = app.wrap(mw::Logger::default());

        app.configure(router::config)
    })
    .keep_alive(Duration::from_secs(30))
    .bind((host.to_string(), *port))?
    .run()
    .await?;

    Ok(())
}

fn install_helpers(verbose_enabled: bool) -> eyre::Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    let default_level_filter = if verbose_enabled {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    let stderr_logs = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(default_level_filter.into())
                .from_env_lossy(),
        );

    tracing_subscriber::registry().with(stderr_logs).init();

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("{}", panic_hook.panic_report(info));
        default_panic(info);
    }));

    Ok(())
}
