use std::{fs, io};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::util::mk_error;

pub const CERTFILE: &str = "/etc/letsencrypt/live/cknittel.com/cert.pem";
pub const KEYFILE: &str = "/etc/letsencrypt/live/cknittel.com/privkey.pem";

pub fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
  let certfile =
    fs::File::open(filename).map_err(|e| mk_error(format!("failed to open {filename}: {e}")))?;
  let mut reader = io::BufReader::new(certfile);

  rustls_pemfile::certs(&mut reader).collect()
}

pub fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
  let keyfile =
    fs::File::open(filename).map_err(|e| mk_error(format!("failed to open {filename}: {e}")))?;
  let mut reader = io::BufReader::new(keyfile);

  rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
