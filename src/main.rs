#[macro_use] extern crate clap;
#[macro_use] extern crate log;

mod slib;

use chrono;
use clap::Parser;
use iron::prelude::*;
use iron::status;
use slib::router::{Router, RunAfter};
use std::io::Read;

struct GlobalLogger;

static LOGGER: GlobalLogger = GlobalLogger;
static mut RESPONSE: Option<String> = None;

#[derive(Parser, Debug)]
#[command(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!()
)]
struct Args {
    /// Specify the IP to bind to for the server
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    /// The port to bind the server to
    #[arg(short, long, default_value_t = 8000)]
    port: u32,

    /// Specify a custom response instead of the default
    #[arg(short, long, default_value = "ok")]
    response: String,

    /// Turn on debug output
    #[arg(short='D', long, default_value_t = false)]
    debug: bool,
}


/// This implements the logging to stderr from the `log` crate
impl log::Log for GlobalLogger {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        return meta.level() <= log::max_level();
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let d = chrono::Local::now();
            eprintln!(
                "{} - {} - {}:{} {} - {}",
                d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                record.target(),
                record.args(),
            );
        }
    }

    fn flush(&self) {}
}

/// Create a set of CLI args via the `clap` crate and return the matches
fn get_args() -> Args {
    return Args::parse();
}

/// Set the global logger from the `log` crate
fn setup_logging(args: &Args) {
    let l = if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(l);
}

fn index(req: &mut Request) -> IronResult<Response> {
    debug!(
        "Got a {} request from {}:{}",
        req.method,
        req.remote_addr.ip(),
        req.remote_addr.port()
    );

    println!("{} {} {}", req.version, req.method, req.url.path().join("/"));
    for item in req.headers.iter() {
        print!("{:?}", item);
    }

    let mut body: String = String::new();
    req.body.read_to_string(&mut body).unwrap();

    if body.len() > 0 {
        println!();
        println!("{}", body);
    }
    println!();
    println!("--");
    println!();

    let response = Response::with((status::Ok, "ok"));

    return Ok(response);
}

fn get_router() -> Router {
    let mut router = Router::new();

    router.get("*", index);
    router.post("*", index);
    router.put("*", index);
    router.delete("*", index);

    return router;
}

fn main() {
    let args = get_args();
    setup_logging(&args);

    unsafe {
        // Set the response global
        RESPONSE = Some(args.response.clone());
    }

    let router = get_router();
    let bind = format!(
        "{}:{}",
        args.bind,
        args.port,
    );

    let r = RunAfter::new(args.response.clone());
    let mut chain = Chain::new(router);
    chain.link_after(r);

    info!("Starting server on: {}", bind);
    Iron::new(chain).http(bind).unwrap();
}