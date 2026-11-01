//! HTTP listener whose bound port is shared with network discovery and access links.
use std::{io, time::Duration};
use tiny_http::{Request, Server};

pub(super) struct HttpServer {
    server: Server,
    pub port: u16,
}

impl HttpServer {
    pub fn bind(host: &str, port: u16) -> Result<Self, String> {
        let server = listen(host, port).map_err(|error| {
            crate::core::i18n::Language::default().format(
                "services-could-not-open-port",
                &[("port", &port.to_string()), ("error", &error.to_string())],
            )
        })?;
        let port = server.server_addr().to_ip().unwrap().port();
        Ok(Self { server, port })
    }

    pub fn recv_timeout(&self, timeout: Duration) -> io::Result<Option<Request>> {
        self.server.recv_timeout(timeout)
    }
}

fn listen(host: &str, port: u16) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
    // tiny_http's accept thread closes asynchronously after Server::drop.
    // Allow it to release the socket when settings restart the server.
    for attempt in 0..5 {
        match Server::http((host, port)) {
            Ok(server) => return Ok(server),
            Err(error)
                if attempt < 4
                    && error
                        .downcast_ref::<io::Error>()
                        .is_some_and(|error| error.kind() == io::ErrorKind::AddrInUse) =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
    };

    #[test]
    fn bound_port_serves_requests_and_is_released() {
        let server = HttpServer::bind("127.0.0.1", 0).unwrap();
        let port = server.port;
        let worker = std::thread::spawn(move || {
            let request = server
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                .unwrap();
            assert_eq!(request.url(), "/control");
            request
                .respond(tiny_http::Response::from_string("control panel"))
                .unwrap();
        });
        let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        socket
            .write_all(b"GET /control HTTP/1.0\r\nHost: omni-presenter.local\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        socket.read_to_string(&mut response).unwrap();
        assert!(response.ends_with("control panel"));
        worker.join().unwrap();
        // Verify shutdown separately from active connections. The listener's
        // accept thread can take a moment to release its socket.
        let server = HttpServer::bind("127.0.0.1", 0).unwrap();
        let port = server.port;
        drop(server);
        listen("127.0.0.1", port).unwrap();
    }

    #[test]
    fn occupied_configured_port_reports_the_conflict() {
        let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = occupied.local_addr().unwrap().port();
        let error = HttpServer::bind("127.0.0.1", port).err().unwrap();
        assert!(error.contains(&port.to_string()));
    }
}
