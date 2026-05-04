use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};

use crate::api;

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) fn handle_connection(mut stream: TcpStream, request: &str) -> io::Result<()> {
    let Some(key) = websocket_key(request) else {
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 21\r\nConnection: close\r\n\r\nmissing websocket key")?;
        return Ok(());
    };

    let accept = websocket_accept(&key);
    write!(
        stream,
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\
         \r\n"
    )?;

    send_text(&mut stream, r#"{"type":"ready","ok":true}"#)?;
    if let Some(command) = read_text(&mut stream)? {
        let mut send_error = None;
        api::run(&command, |event| {
            if send_error.is_none() {
                send_error = send_text(&mut stream, &api::event_json(&event)).err();
            }
        });
        if let Some(error) = send_error {
            return Err(error);
        }
    }
    send_close(&mut stream)
}

fn websocket_key(request: &str) -> Option<String> {
    request.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("Sec-WebSocket-Key")
            .then(|| value.trim().to_string())
    })
}

fn websocket_accept(key: &str) -> String {
    let mut data = Vec::with_capacity(key.len() + WS_GUID.len());
    data.extend_from_slice(key.as_bytes());
    data.extend_from_slice(WS_GUID.as_bytes());
    STANDARD.encode(Sha1::digest(&data))
}

fn read_text(stream: &mut TcpStream) -> io::Result<Option<String>> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;
    let opcode = header[0] & 0x0f;
    if opcode == 0x8 {
        return Ok(None);
    }

    let masked = (header[1] & 0x80) != 0;
    let mut len = u64::from(header[1] & 0x7f);
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext)?;
        len = u64::from(u16::from_be_bytes(ext));
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext)?;
        len = u64::from_be_bytes(ext);
    }

    let mut mask = [0u8; 4];
    if masked {
        stream.read_exact(&mut mask)?;
    }

    let mut payload = vec![0; len as usize];
    stream.read_exact(&mut payload)?;
    if masked {
        for (idx, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[idx % 4];
        }
    }

    String::from_utf8(payload)
        .map(Some)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "websocket text is not utf-8"))
}

fn send_text(stream: &mut TcpStream, text: &str) -> io::Result<()> {
    send_frame(stream, 0x1, text.as_bytes())
}

fn send_close(stream: &mut TcpStream) -> io::Result<()> {
    send_frame(stream, 0x8, &[])
}

fn send_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) -> io::Result<()> {
    let mut header = Vec::with_capacity(10);
    header.push(0x80 | opcode);
    if payload.len() < 126 {
        header.push(payload.len() as u8);
    } else if payload.len() <= u16::MAX as usize {
        header.push(126);
        header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        header.push(127);
        header.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    stream.write_all(&header)?;
    stream.write_all(payload)?;
    stream.flush()
}
