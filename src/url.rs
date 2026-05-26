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

    pub fn get_request(&self, method: &str, headers: &[String], body: Option<&str>) -> String {
        debug_assert_eq!(self.protocol, "http");

        let host = if self.port == 80 {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        };

        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nAccept: */*\r\n",
            method, self.path, host
        );

        for header in headers {
            request.push_str(header);
            request.push_str("\r\n");
        }

        if let Some(body) = body {
            let has_content_length = headers.iter().any(|header| {
                header
                    .split_once(':')
                    .is_some_and(|(name, _)| name.eq_ignore_ascii_case("Content-Length"))
            });

            if !has_content_length {
                request.push_str(&format!("Content-Length: {}\r\n", body.len()));
            }
        }

        request.push_str("Connection: close\r\n\r\n");

        if let Some(body) = body {
            request.push_str(body);
        }

        request
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
            url.get_request("GET", &[], None),
            "GET /docs HTTP/1.1\r\nHost: rom.com\r\nAccept: */*\r\nConnection: close\r\n\r\n"
        );
    }

    #[test]
    fn builds_request_with_custom_method() {
        let url = Url::parse("http://rom.com/docs").unwrap();

        assert_eq!(
            url.get_request("DELETE", &[], None),
            "DELETE /docs HTTP/1.1\r\nHost: rom.com\r\nAccept: */*\r\nConnection: close\r\n\r\n"
        );
    }

    #[test]
    fn builds_post_request_with_headers_and_body() {
        let url = Url::parse("http://rom.com/docs").unwrap();
        let headers = vec!["Content-Type: application/json".to_string()];

        assert_eq!(
            url.get_request("POST", &headers, Some(r#"{"key": "value"}"#)),
            "POST /docs HTTP/1.1\r\nHost: rom.com\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 16\r\nConnection: close\r\n\r\n{\"key\": \"value\"}"
        );
    }

    #[test]
    fn does_not_duplicate_custom_content_length() {
        let url = Url::parse("http://rom.com/docs").unwrap();
        let headers = vec!["Content-Length: 3".to_string()];

        assert_eq!(
            url.get_request("POST", &headers, Some("abc")),
            "POST /docs HTTP/1.1\r\nHost: rom.com\r\nAccept: */*\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc"
        );
    }
}
