#![deny(warnings)]
#![warn(rust_2018_idioms)]

use futures::stream::Stream;
use futures::sync::mpsc;
use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};
use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use termion::input::TermRead;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not set SIGTERM handler: {}", source))]
    HandlerError { source: ctrlc::Error },

    #[snafu(display("Could not read secret from file {}: {}", path.display(), source))]
    ReadFromFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Stdin error: {}", source))]
    StdinError { source: std::io::Error },

    #[snafu(display("Stdout error: {}", source))]
    StdoutError { source: std::io::Error },

    #[snafu(display("Stdin aborted"))]
    StdinAborted { backtrace: Backtrace },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, StructOpt)]
#[structopt(name = "httpose", about = "Options to run httpose")]
struct Opt {
    /// Hostname and port to listen to
    #[structopt(short = "a", default_value = "127.0.0.1:2048")]
    addr: SocketAddr,

    /// Use content from specified file path instead of env var or secret stdin
    #[structopt(short = "f")]
    file: Option<PathBuf>,
}

const SECRET_NAME: &'static str = "HTTPOSE_SECRET";

fn main_impl() -> Result<()> {
    let opt = Opt::from_args();

    // The order of secret to use is as follow:
    // 1. File content (-f)
    // 2. Env var (HTTPOSE_SECRET)
    // 3. Stdin

    let secret = match opt.file {
        // Read from file
        Some(path) => {
            println!("Using file content as secret");
            fs::read_to_string(&path).context(ReadFromFile { path })?
        }
        _ => match env::var(SECRET_NAME) {
            // Read from env var if present
            Ok(secret) => {
                println!("Using env var secret");
                secret
            }

            // Get secret from stdin without echoing
            _ => {
                print!("Enter the secret: ");
                stdout().flush().context(StdoutError {})?;

                let stdin = stdin();
                let mut stdin = stdin.lock();
                let stdout = stdout();
                let mut stdout = stdout.lock();
                let secret = stdin
                    .read_passwd(&mut stdout)
                    .context(StdinError {})?
                    .context(StdinAborted {})?;

                println!();
                secret
            }
        },
    };

    let secret = Arc::new(secret);
    let (tx, rx) = mpsc::channel(1);
    let tx = Arc::new(Mutex::new(tx));

    // Gracefully handle SIGTERM
    ctrlc::set_handler(move || {
        let _ = tx
            .lock()
            .expect("Unable to lock mutex to get handler sender! Aborting...")
            .try_send(())
            .expect("Handler unable to send signal to receiver! Aborting...");
    })
    .context(HandlerError {})?;

    let server = Server::bind(&opt.addr)
        .serve(move || {
            let secret = secret.clone();
            service_fn_ok(move |_: Request<Body>| {
                Response::new(Body::from((*secret).clone()))
            })
        })
        .with_graceful_shutdown(rx.into_future().map(|_| ()))
        .map_err(|e| eprintln!("Server error: {}", e));

    println!("Listening on {}", opt.addr);
    rt::run(server);
    println!("\nReceived SIGTERM, terminating...");

    Ok(())
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
