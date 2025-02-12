#[macro_use]
extern crate tracing;

pub mod entities;
pub mod library;
mod router;
pub mod utils;

use std::{path::PathBuf, time::Duration};

use actix_web::{App, HttpServer, middleware as mw, web::Data};
use clap::{ArgGroup, arg, command, value_parser};
use library::Libraries;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let matches = command!()
        .arg(arg!(--"lib:name" <NAME> ... "Add a library to the catalog. It must be followed by --lib:path").required(true))
        .arg(arg!(--"lib:path" <PATH> ... "Set the preceded's library path. It must point to the directory where \"metadata.db\" is located").value_parser(value_parser!(PathBuf)).required(true))
        .group(ArgGroup::new("lib").args(["lib:name", "lib:path"]).multiple(true))
        .arg(arg!(--host <VALUE> "Set the server's listening host").default_value("localhost"))
        .arg(arg!(--port <VALUE> "Set the server's listening port").value_parser(value_parser!(u16)).default_value("10100"))
        .arg(arg!(-v --verbose "Enable verbose logging"))
        .after_help("Created by Thunder04 <https://github.com/thunder04>.")
        .get_matches();

    install_helpers(matches.get_flag("verbose"))?;

    let libraries = Data::new(Libraries::from_arg_matches(&matches).await?);
    let host: &String = matches.try_get_one("host")?.expect("required");
    let port: &u16 = matches.try_get_one("port")?.expect("required");

    HttpServer::new(move || {
        let app = App::new()
            .wrap(mw::NormalizePath::new(mw::TrailingSlash::Always))
            .app_data(libraries.clone())
            .configure(router::config);

        #[cfg(debug_assertions)]
        let app = app.wrap(mw::Logger::default());

        app
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
