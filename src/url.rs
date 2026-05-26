#[derive(Debug)]
pub struct Url {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl Url {
    pub fn parse(input: &str) -> Result<Self, String> {
        let (protocol, rest) = input
            .split_once("://")
            .ok_or_else(|| "no protocol is insane".to_string())?;

        if protocol != "http" {
            return Err(format!("unsupported protocol: {protocol}"));
        }

        let (authority, path) = match rest.split_once('/') {
            Some((authority, path)) => (authority, format!("/{path}")),
            None => (rest, "/".to_string()),
        };

        if authority.is_empty() {
            return Err("no host, really?".to_string());
        }

        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => {
                if host.is_empty() {
                    return Err("URL must include a host".to_string());
                }

                let port = port
                    .parse::<u16>()
                    .map_err(|_| format!("invalid port: {port}"))?;

                (host.to_string(), port)
            }
            None => (authority.to_string(), 80),
        };

        Ok(Self {
            protocol: protocol.to_string(),
            host,
            port,
            path,
        })
    }

    pub fn get_request(&self) -> String {
        debug_assert_eq!(self.protocol, "http");

        let host = if self.port == 80 {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        };

        format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: */*\r\nConnection: close\r\n\r\n",
            self.path, host
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Url;

    #[test]
    fn parses_http_url_with_default_port() {
        let url = Url::parse("http://rom.com/docs").unwrap();

        assert_eq!(url.protocol, "http");
        assert_eq!(url.host, "rom.com");
        assert_eq!(url.port, 80);
        assert_eq!(url.path, "/docs");
    }

    #[test]
    fn parses_http_url_with_explicit_port() {
        let url = Url::parse("http://rom.com:8080/docs").unwrap();

        assert_eq!(url.host, "rom.com");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/docs");
    }

    #[test]
    fn defaults_to_root_path() {
        let url = Url::parse("http://rom.com").unwrap();

        assert_eq!(url.path, "/");
    }

    #[test]
    fn rejects_non_http_protocols() {
        let error = Url::parse("https://rom.com").unwrap_err();

        assert_eq!(error, "unsupported protocol: https");
    }

    #[test]
    fn builds_get_request_with_close_header() {
        let url = Url::parse("http://rom.com/docs").unwrap();

        assert_eq!(
            url.get_request(),
            "GET /docs HTTP/1.1\r\nHost: rom.com\r\nAccept: */*\r\nConnection: close\r\n\r\n"
        );
    }
}
