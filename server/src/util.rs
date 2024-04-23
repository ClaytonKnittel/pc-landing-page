use std::io;

pub fn mk_error(err: String) -> io::Error {
  io::Error::new(io::ErrorKind::Other, err)
}
