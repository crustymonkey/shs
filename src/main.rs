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
            let split: Vec<&str> = header.splitn(2,':').map(|s| s.trim()).collect();
            if split.len() != 2 {
                panic!("Invalid header format: {}", header);
            }
            (split[0].to_string(), split[1].to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args(debug: bool) -> Args {
        Args {
            bind: "127.0.0.1".to_string(),
            port: 8000,
            response: "ok".to_string(),
            headers: vec![],
            debug,
        }
    }

    #[test]
    fn test_parse_single_header() {
        let headers = vec!["Content-Type: application/json".to_string()];
        let result = parse_response_headers(&headers);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Content-Type");
        assert_eq!(result[0].1, "application/json");
    }

    #[test]
    fn test_parse_multiple_headers() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "X-Custom-Header: custom-value".to_string(),
            "Authorization: Bearer token123".to_string(),
        ];
        let result = parse_response_headers(&headers);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("Content-Type".to_string(), "application/json".to_string()));
        assert_eq!(result[1], ("X-Custom-Header".to_string(), "custom-value".to_string()));
        assert_eq!(result[2], ("Authorization".to_string(), "Bearer token123".to_string()));
    }

    #[test]
    fn test_parse_headers_with_extra_whitespace() {
        let headers = vec!["  Content-Type  :   text/html  ".to_string()];
        let result = parse_response_headers(&headers);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Content-Type");
        assert_eq!(result[0].1, "text/html");
    }

    #[test]
    fn test_parse_empty_headers() {
        let headers: Vec<String> = vec![];
        let result = parse_response_headers(&headers);

        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_header_with_colon_in_value() {
        let headers = vec!["X-Timestamp: 2024:01:01:12:00:00".to_string()];
        let result = parse_response_headers(&headers);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "X-Timestamp");
        assert_eq!(result[0].1, "2024:01:01:12:00:00");
    }

    #[test]
    fn test_args_default_values() {
        let args = create_test_args(false);

        assert_eq!(args.bind, "127.0.0.1");
        assert_eq!(args.port, 8000);
        assert_eq!(args.response, "ok");
        assert!(args.headers.is_empty());
        assert!(!args.debug);
    }

    #[test]
    fn test_args_with_debug_enabled() {
        let args = create_test_args(true);

        assert!(args.debug);
    }

    #[test]
    fn test_args_with_custom_values() {
        let args = Args {
            bind: "0.0.0.0".to_string(),
            port: 9000,
            response: "custom response".to_string(),
            headers: vec!["X-Test: value".to_string()],
            debug: true,
        };

        assert_eq!(args.bind, "0.0.0.0");
        assert_eq!(args.port, 9000);
        assert_eq!(args.response, "custom response");
        assert_eq!(args.headers.len(), 1);
        assert!(args.debug);
    }

/*
    #[test]
    fn test_full_header_parsing_workflow() {
        let raw_headers = vec![
            "Content-Type: application/json".to_string(),
            "Cache-Control: no-cache".to_string(),
        ];

        let parsed = parse_response_headers(&raw_headers);

        assert_eq!(parsed.len(), 2);

        for (key, value) in &parsed {
            assert!(!key.is_empty());
            assert!(!value.is_empty());
        }
    }

    #[test]
    fn test_args_to_bind_address_format() {
        let args = Args {
            bind: "192.168.1.1".to_string(),
            port: 3000,
            response: "test".to_string(),
            headers: vec![],
            debug: false,
        };

        let bind = format!("{}:{}", args.bind, args.port);
        assert_eq!(bind, "192.168.1.1:3000");
    }
*/
}
