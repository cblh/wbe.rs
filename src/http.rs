use std::{
    collections::BTreeMap,
    io::BufRead,
    io::{BufReader, Read, Write},
    net::TcpStream,
    str,
    str::FromStr,
};

use rustls_connector::RustlsConnector;
use tracing::{debug, instrument};

use crate::*;

#[instrument]
pub fn request(url: &str) -> eyre::Result<(BTreeMap<String, String>, Vec<u8>)> {
    let mut url = url;
    let scheme = lparse_chomp(&mut url, "https?:").expect("failed to chomp url scheme");
    lparse_chomp(&mut url, "//").expect("failed to chomp url //");
    let (host, path) = lparse_split(url, "[^/]+").expect("failed to split url host/path");
    let (hostname, port) = rparse_split(host, r":\d+").unwrap_or((
        host,
        match scheme {
            "http:" => ":80",
            "https:" => ":443",
            _ => unreachable!(),
        },
    ));
    let port = u16::from_str(&port[1..])?;
    let path = match path {
        "" => "/",
        other => other,
    };

    let mut stream: Box<dyn ReadWriteStream> = match scheme {
        "http:" => Box::new(TcpStream::connect((hostname, port))?),
        "https:" => {
            let connector = RustlsConnector::new_with_native_certs()?;
            let stream = TcpStream::connect((hostname, port))?;
            Box::new(connector.connect(hostname, stream)?)
        }
        _ => unreachable!(),
    };
    write!(stream, "GET {} HTTP/1.0\r\n", path)?;
    write!(stream, "Host: {}\r\n\r\n", host)?;

    let mut stream = BufReader::new(stream);
    let mut received = vec![];
    assert_ne!(stream.read_until(b'\n', &mut received)?, 0);

    let line = received.strip_suffix(b"\r\n").unwrap();
    let [version, status, explanation] = line.splitn(3, |x| *x == b' ').collect::<Vec<_>>()[..]
        else { panic!("failed to parse response status line") };
    assert_eq!(
        status,
        b"200",
        "unexpected {:?} {:?} {:?}",
        dump(version),
        dump(status),
        dump(explanation)
    );
    received.clear();

    let mut headers = BTreeMap::default();
    while stream.read_until(b'\n', &mut received)? > 0 {
        // TODO: hard-coding utf-8 is not correct in practice
        let line = str::from_utf8(&received)?;
        let line = line.strip_suffix("\r\n").unwrap();
        if line.is_empty() {
            break;
        }
        let (field, value) = line
            .split_once(":")
            .expect("failed to parse response header");
        debug!(field = field, value = value);
        headers.insert(
            trim_ascii(field).to_ascii_lowercase(),
            trim_ascii(value).to_owned(),
        );
        received.clear();
    }

    assert!(!headers.contains_key("transfer-encoding"));
    assert!(!headers.contains_key("content-encoding"));

    let mut body = vec![];
    stream.read_to_end(&mut body)?;
    debug!(body = dump(&body));

    Ok((headers, body))
}
