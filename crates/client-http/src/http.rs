use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::{api, wss};

pub(crate) fn run_server() -> io::Result<()> {
    let port = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u16>().ok())
        .or_else(|| std::env::var("PORT").ok().and_then(|arg| arg.parse().ok()))
        .unwrap_or(8080);

    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("Listening on http://0.0.0.0:{port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = handle_connection(stream);
            }
            Err(err) => eprintln!("accept failed: {err}"),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut buf = vec![0; 16 * 1024];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..n]).to_string();
    let request_line = request.lines().next().unwrap_or("");
    let fields = request_line.split_ascii_whitespace().collect::<Vec<_>>();
    if fields.len() < 2 {
        stream.write_all(&response("400 Bad Request", "text/plain", "bad request"))?;
        return Ok(());
    }

    match (fields[0], fields[1]) {
        ("GET", "/") => stream.write_all(&response(
            "200 OK",
            "text/html; charset=utf-8",
            api::index_html(),
        ))?,
        ("GET", "/report") => {
            stream.write_all(&response("200 OK", "application/json", &api::report_json()))?
        }
        ("GET", "/ws") => wss::handle_connection(stream, &request)?,
        _ => stream.write_all(&response("404 Not Found", "text/plain", "not found"))?,
    }

    Ok(())
}

fn response(status: &str, content_type: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}
