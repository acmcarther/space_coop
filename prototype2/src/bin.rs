extern crate prototype2;
extern crate clap;

use std::str::FromStr;
use std::net::ToSocketAddrs;
use clap::{App, Arg, ArgMatches, SubCommand};
use clap::AppSettings::SubcommandRequired;

static EXAMPLE_SERVER_COMMAND: &'static str = "space_coop server 8888";
static EXAMPLE_CLIENT_COMMAND: &'static str = "space_coop client 9999 192.168.0.1:8888";

fn main() {
  let matches = App::new("space coop")
    .usage(format!("\t{}\n\t{}", EXAMPLE_SERVER_COMMAND, EXAMPLE_CLIENT_COMMAND).as_ref())
    .settings(&[SubcommandRequired])
    .subcommand(SubCommand::with_name("server")
      .usage(EXAMPLE_SERVER_COMMAND)
      .arg(Arg::with_name("port").required(true)))
    .subcommand(SubCommand::with_name("client")
      .usage(EXAMPLE_CLIENT_COMMAND)
      .arg(Arg::with_name("port").required(true))
      .arg(Arg::with_name("server address").required(true)))
    .get_matches();

  if let Some(server_matches) = matches.subcommand_matches("server") {
    prototype2::server::start(port_from(&server_matches))
  } else if let Some(client_matches) = matches.subcommand_matches("client") {
    prototype2::client::start(port_from(&client_matches), addr_from(&client_matches))
  }
}

fn port_from(matches: &ArgMatches) -> u16 {
  matches.value_of("port").and_then(|v| u16::from_str(&v).ok()).unwrap()
}

fn addr_from(matches: &ArgMatches) -> std::net::SocketAddr {
  matches.value_of("server address")
    .and_then(|v| v.to_socket_addrs().ok())
    .and_then(|mut socket_addr_iter| socket_addr_iter.next()).unwrap()
}
