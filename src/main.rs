mod url;

use std::{
    env,
    io::{self, Read, Write},
    net::TcpStream,
    process,
};

use url::Url;

fn main() {
    let mut verbose = false;
    let mut method = "GET".to_string();
    let mut method_was_set = false;
    let mut headers = Vec::new();
    let mut data = None;
    let mut raw_url = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "-v" {
            verbose = true;
        } else if arg == "-X" {
            let Some(value) = args.next() else {
                print_usage_and_exit();
            };
            method = value;
            method_was_set = true;
        } else if arg == "-H" {
            let Some(value) = args.next() else {
                print_usage_and_exit();
            };
            headers.push(value);
        } else if arg == "-d" {
            let Some(value) = args.next() else {
                print_usage_and_exit();
            };
            data = Some(value);
        } else if raw_url.is_none() {
            raw_url = Some(arg);
        } else {
            print_usage_and_exit();
        }
    }

    let Some(raw_url) = raw_url else {
        print_usage_and_exit();
    };

    let url = match Url::parse(&raw_url) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    };

    let mut stream = match TcpStream::connect((url.host.as_str(), url.port)) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!(
                "error: failed to connect to {}:{}: {error}",
                url.host, url.port
            );
            process::exit(1);
        }
    };

    if data.is_some() && !method_was_set {
        method = "POST".to_string();
    }

    let request = url.get_request(&method, &headers, data.as_deref());

    if verbose {
        print_verbose_message(">", &request);
    }

    if let Err(error) = stream.write_all(request.as_bytes()) {
        eprintln!("error: failed to send request: {error}");
        process::exit(1);
    }

    let mut response = Vec::new();
    if let Err(error) = stream.read_to_end(&mut response) {
        eprintln!("error: failed to read response: {error}");
        process::exit(1);
    }

    let (headers, body) = split_response(&response);

    if verbose {
        print_verbose_message("<", headers);
    }

    if let Err(error) = io::stdout().write_all(body) {
        eprintln!("error: failed to write response: {error}");
        process::exit(1);
    }
}

fn split_response(response: &[u8]) -> (&str, &[u8]) {
    let Some(header_end) = response.windows(4).position(|window| window == b"\r\n\r\n") else {
        return ("", response);
    };

    let headers = str::from_utf8(&response[..header_end]).unwrap_or("");
    let body = &response[header_end + 4..];

    (headers, body)
}

fn print_verbose_message(prefix: &str, message: &str) {
    for line in message.trim_end_matches("\r\n").split("\r\n") {
        println!("{prefix} {line}");
    }
    println!("{prefix}");
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage: gurl [-v] [-X <method>] [-H <header>] [-d <data>] <url>");
    process::exit(1);
}
