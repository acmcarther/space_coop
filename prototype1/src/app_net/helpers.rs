pub use self::helpers::{get_own_ip, get_ip};

mod helpers {
  use std::net::{IpAddr, lookup_host, TcpStream, Shutdown};
  use std::str::FromStr;
  use std::io::Error;

  // TODO: Use a non-hacky solution
  pub fn get_own_ip() -> IpAddr {
    let external_ip = host_lookup("google.com").unwrap();

    let stream = TcpStream::connect((external_ip, 80)).unwrap();
    let local_addr = stream.local_addr().unwrap();
    let _ = stream.shutdown(Shutdown::Both);
    local_addr.ip()
  }

  pub fn get_ip(ip: &str) -> IpAddr {
    IpAddr::from_str(ip).or_else(|_| host_lookup(ip)).unwrap()
  }

  fn host_lookup(host: &str) -> Result<IpAddr, Error> {
    if host == "localhost" {
      return Ok(get_own_ip());
    }
    lookup_host(host)
      .unwrap()
      .next()
      .unwrap()
      .map(|res| res.ip())
  }
}
