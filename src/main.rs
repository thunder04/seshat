#[macro_use]
extern crate tracing;

pub mod errors;
pub mod library;
mod router;
pub mod utils;

use std::{path::PathBuf, time::Duration};

use actix_web::{App, HttpServer, middleware as mw, web::Data};
use clap::Parser;
use library::Libraries;

pub type Result<T, E = errors::AppError> = std::result::Result<T, E>;

#[derive(clap::Parser)]
#[group(id = "lib", multiple = true)]
#[clap(
    after_help = "Created by Thunder04 <https://github.com/thunder04>",
    about
)]
pub struct Cli {
    /// Set the server's listening host
    #[clap(long, default_value = "localhost")]
    pub host: String,
    /// Set the server's listening port
    #[clap(long, default_value = "10100")]
    pub port: u16,

    /// Enable verbose logging. For greater control, use the $RUST_LOG environment
    /// variable
    #[cfg_attr(debug_assertions, clap(default_value = "true"))]
    #[clap(short, long, global = true)]
    pub verbose: bool,

    /// Add a library to the catalog. It must be followed by --lib:path
    #[clap(long = "lib:name", group = "lib")]
    pub lib_name: Vec<String>,
    /// Set the preceded's library path. It must point to the directory where
    /// "metadata.db" is located.
    #[clap(long = "lib:path", group = "lib")]
    pub lib_path: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let mut cli = Cli::parse();
    install_helpers(cli.verbose)?;

    let libraries = Data::new(Libraries::from_cli(&mut cli).await?);

    HttpServer::new(move || {
        App::new()
            .wrap(mw::Condition::new(cli.verbose, mw::Logger::default()))
            .wrap(mw::NormalizePath::trim())
            .app_data(libraries.clone())
            .configure(router::config)
    })
    .keep_alive(Duration::from_secs(30))
    .bind((cli.host, cli.port))?
    .run()
    .await?;

    Ok(())
}

fn install_helpers(verbose: bool) -> eyre::Result<()> {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    let stderr_logs = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(if verbose {
                    LevelFilter::DEBUG.into()
                } else {
                    LevelFilter::INFO.into()
                })
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
