use std::{fs, io, net::SocketAddr, sync::Arc};

use http_body_util::Full;
use hyper::{
  body::{self, Bytes},
  service::service_fn,
  Request, Response,
};
use hyper_util::{
  rt::{TokioExecutor, TokioIo},
  server::conn::auto::Builder,
};
use rustls::{
  pki_types::{CertificateDer, PrivateKeyDer},
  ServerConfig,
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

fn error(err: String) -> io::Error {
  io::Error::new(io::ErrorKind::Other, err)
}

async fn test(_: Request<body::Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
  Ok(Response::new(Full::new("Hello".into())))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

  let addr = SocketAddr::new([10, 0, 0, 181].into(), 3000);

  let certs = load_certs("/etc/letsencrypt/live/cknittel.com/cert.pem")?;
  let key = load_private_key("/etc/letsencrypt/live/cknittel.com/privkey.pem")?;

  println!("Starting server on https://{addr}");

  let listener = TcpListener::bind(addr).await?;

  // Build TLS configuration.
  let mut server_config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|e| error(e.to_string()))?;
  server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
  let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

  let service = service_fn(test);

  loop {
    let (stream, _remote_addr) = listener.accept().await?;

    let tls_acceptor = tls_acceptor.clone();

    tokio::task::spawn(async move {
      let tls_stream = match tls_acceptor.accept(stream).await {
        Ok(tls_stream) => tls_stream,
        Err(err) => {
          eprintln!("Failed to perform TLS handshake: {err:#}");
          return;
        }
      };
      let io = TokioIo::new(tls_stream);

      if let Err(err) = Builder::new(TokioExecutor::new())
        .serve_connection(io, service)
        .await
      {
        eprintln!("Error serving connection: {err:#}");
      }
    });
  }
}

fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
  let certfile =
    fs::File::open(filename).map_err(|e| error(format!("failed to open {filename}: {e}")))?;
  let mut reader = io::BufReader::new(certfile);

  rustls_pemfile::certs(&mut reader).collect()
}

fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
  let keyfile =
    fs::File::open(filename).map_err(|e| error(format!("failed to open {filename}: {e}")))?;
  let mut reader = io::BufReader::new(keyfile);

  rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
