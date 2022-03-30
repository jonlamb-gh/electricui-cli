//! An unofficial and incomplete CLI for devices implementing the ElectricUI Binary Protocol.

// TODO
//#![deny(warnings, clippy::all)]

use crate::opts::{Opts, Subcommand};
use structopt::StructOpt;
use tracing::{debug, error};

mod check;
mod codec;
mod device;
mod error;
mod opts;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match do_main().await {
        Ok(()) => Ok(()),
        Err(e) => {
            // TODO - print the sources/chain
            error!("{}", e);
            Err(e)
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();
    try_init_tracing_subscriber()?;

    let intr = interruptor::Interruptor::new();
    ctrlc::set_handler(move || {
        if intr.is_set() {
            let exit_code = if cfg!(target_family = "unix") {
                // 128 (fatal error signal "n") + 2 (control-c is fatal error signal 2)
                130
            } else {
                // Windows code 3221225786
                // -1073741510 == C000013A
                -1073741510
            };
            std::process::exit(exit_code);
        } else {
            intr.set();
        }
    })?;

    let mut cmd_handle = tokio::spawn(async move {
        match opts.subcommand {
            Subcommand::Check(c) => check::check(c),
        }
        .await
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            debug!("User signaled shutdown");
        }
        res = &mut cmd_handle => {
            debug!("Command returned");
            let join_res = res?;
            match join_res {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
        }
    };

    Ok(())
}

fn try_init_tracing_subscriber() -> Result<(), Box<dyn std::error::Error>> {
    let builder = tracing_subscriber::fmt::Subscriber::builder();
    let env_filter = std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV)
        .map(tracing_subscriber::EnvFilter::new)
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(format!(
                "{}={}",
                env!("CARGO_PKG_NAME").replace('-', "_"),
                tracing::Level::WARN
            ))
        });
    let builder = builder.with_env_filter(env_filter);
    let subscriber = builder.finish();
    use tracing_subscriber::util::SubscriberInitExt;
    subscriber.try_init()?;
    Ok(())
}

mod interruptor {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    #[repr(transparent)]
    pub struct Interruptor(Arc<AtomicBool>);

    impl Interruptor {
        pub fn new() -> Self {
            Interruptor(Arc::new(AtomicBool::new(false)))
        }

        pub fn set(&self) {
            self.0.store(true, SeqCst);
        }

        pub fn is_set(&self) -> bool {
            self.0.load(SeqCst)
        }
    }

    impl Default for Interruptor {
        fn default() -> Self {
            Self::new()
        }
    }
}
