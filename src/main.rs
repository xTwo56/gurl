mod url;

use std::{env, process};

use url::Url;

fn main() {
    let Some(raw_url) = env::args().nth(1) else {
        eprintln!("usage: gurl <url>");
        process::exit(1);
    };

    let url = match Url::parse(&raw_url) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    };

    print!("{}", url.get_request());
}
