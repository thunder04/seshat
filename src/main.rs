#[macro_use]
extern crate tracing;

pub mod entities;
mod router;

use std::time::Duration;

use actix_web::{App, HttpServer, middleware as mw};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    install_helpers()?;

    HttpServer::new(|| {
        let app = App::new()
            // TODO: What should I send?
            .wrap(mw::DefaultHeaders::new())
            .wrap(mw::NormalizePath::new(mw::TrailingSlash::Always));

        #[cfg(debug_assertions)]
        let app = app.wrap(mw::Logger::default());

        app.configure(router::config)
    })
    .keep_alive(Duration::from_secs(30))
    .bind(("127.0.0.1", 10100))?
    .run()
    .await?;

    Ok(())
}

fn install_helpers() -> eyre::Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    let stderr_logs = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
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
