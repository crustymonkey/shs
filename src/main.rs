#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use chrono;
use clap::Parser;
use tiny_http::{Header, Request, Response, Server};

struct GlobalLogger;

static LOGGER: GlobalLogger = GlobalLogger;

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

    /// Specify any response headers to include. These are in the format of
    /// "header:value"
    #[arg(short = 'H', long)]
    headers: Vec<String>,

    /// Turn on debug output
    #[arg(short = 'D', long, default_value_t = false)]
    debug: bool,
}

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

fn get_args() -> Args {
    return Args::parse();
}

fn setup_logging(args: &Args) {
    let l = if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(l);
}

fn print_request(req: &mut Request) {
    debug!(
        "Got a {} req from {:?}",
        req.method(),
        req.remote_addr()
    );

    println!(
        "HTTP/{} {} {}",
        req.http_version(),
        req.method(),
        req.url()
    );

    for header in req.headers() {
        print!("{}: {}\n", header.field, header.value);
    }

    let mut body = String::new();
    if let Err(e) = req.as_reader().read_to_string(&mut body) {
        error!("Failed to read req body: {}", e);
    }

    if !body.is_empty() {
        println!();
        println!("{}", body);
    }
}

fn handle_request(
    mut req: Request,
    resp_body: &str,
    resp_headers: &[(String, String)],
) {
    print_request(&mut req);

    println!();
    println!("--");
    println!();

    let resp_body = format!("{}\n", resp_body);

    let mut resp = Response::from_string(&resp_body)
        .with_header(Header::from_bytes("Content-Type", "text/plain; charset=utf8").unwrap());

    for (key, value) in resp_headers {
        resp =
            resp.with_header(Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap());
    }

    for header in resp.headers() {
        print!("{}: {}\n", header.field, header.value);
    }

    println!("{}", &resp_body);
    println!("----");
    println!();

    if let Err(e) = req.respond(resp) {
        error!("Failed to send response: {}", e);
    }
}

fn parse_response_headers(headers: &[String]) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|header| {
            let mut split = header.split(':');
            let key = split.next().unwrap().trim();
            let value = split.next().unwrap().trim();
            (key.to_string(), value.to_string())
        })
        .collect()
}

fn main() {
    let args = get_args();
    setup_logging(&args);
    debug!("Headers: {:?}", args.headers);

    let response_headers = parse_response_headers(&args.headers);

    let bind = format!("{}:{}", args.bind, args.port);

    info!("Starting server on: {}", bind);

    let server = Server::http(&bind).expect("Failed to start server");

    for request in server.incoming_requests() {
        handle_request(request, &args.response, &response_headers);
    }
}
