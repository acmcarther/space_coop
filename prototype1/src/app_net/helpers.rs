pub use self::helpers::{get_own_ip, get_ip};

mod helpers {
  use std::net::{IpAddr, lookup_host, TcpStream, Shutdown};
  use std::str::FromStr;
  use std::io::{Error, ErrorKind};

  // TODO: Use a non-hacky solution
  pub fn get_own_ip() -> IpAddr {
    host_lookup("google.com")
      .and_then(|external_ip| TcpStream::connect((external_ip, 80)))
      .and_then(|stream| {
        let addr = stream.local_addr();
        stream.shutdown(Shutdown::Both);
        addr.map(|addr| addr.ip())
      }).unwrap_or(IpAddr::from_str("127.0.0.1").unwrap())
  }

  pub fn get_ip(ip: &str) -> IpAddr {
    IpAddr::from_str(ip).or_else(|_| host_lookup(ip)).unwrap()
  }

  fn host_lookup(host: &str) -> Result<IpAddr, Error> {
    if host == "localhost" {
      return Ok(get_own_ip());
    }
    lookup_host(host)
      .map(|mut results| results.next())
      .and_then(|possible_host_addr| {
        possible_host_addr
         .map(|addr| addr.map(|x| x.ip()))
         .ok_or(Error::new(ErrorKind::Other, "No addresses for host"))
         .and_then(|res| res)
      })
  }
}
