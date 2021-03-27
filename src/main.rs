extern crate chrono;
extern crate clap;
extern crate log;

mod lib;

use clap::{
    ArgMatches,
    App,
    Arg,
    value_t,
    crate_name,
    crate_version,
    crate_description,
};
use iron::prelude::*;
use iron::status;
use log::{debug};
use lib::router::Router;
use std::io::Read;

struct GlobalLogger;

static LOGGER: GlobalLogger = GlobalLogger;
static mut RESPONSE: Option<String> = None;


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
fn get_args() -> ArgMatches<'static> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author("Jay Deiman")
        .about(crate_description!())
        .arg(Arg::with_name("bind")
            .short("-b")
            .long("--bind")
            .default_value("127.0.0.1")
            .takes_value(true)
            .value_name("IP")
            .help("Specify the IP to bind to for the server")
        )
        .arg(Arg::with_name("port")
            .short("-p")
            .long("--port")
            .default_value("8000")
            .takes_value(true)
            .value_name("INT")
            .help("The port to bind the server to")
        )
        .arg(Arg::with_name("response")
            .short("-r")
            .long("--response")
            .default_value("ok")
            .takes_value(true)
            .value_name("TEXT")
            .help("Specify a custom response instead of the default")
        )
        .arg_from_usage("-D, --debug 'Turn on debug output'")
        .get_matches();

    return matches;
}

/// Set the global logger from the `log` crate
fn setup_logging(args: &ArgMatches) {
    let l = if args.is_present("debug") {
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
    
    // Get the value of the response from the global variable
    let mut text = String::new();
    unsafe {
        text += &RESPONSE.clone().unwrap();
    }

    let response = Response::with((status::Ok, text));
    println!("{:?}", response);

    println!("----");
    println!();

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
        RESPONSE = Some(args.value_of("response").unwrap().to_string());
    }

    let router = get_router();
    let bind = format!(
        "{}:{}",
        args.value_of("bind").unwrap(),
        value_t!(args, "port", u32).expect("Invalid value for --port"),
    );

    Iron::new(router).http(bind).unwrap();   
}