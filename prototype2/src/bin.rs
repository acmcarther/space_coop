#![feature(try_from)]
extern crate prototype2;
extern crate clap;

use clap::{App, Arg, ArgMatches, SubCommand};
use clap::AppSettings::SubcommandRequired;
use std::convert::TryFrom;
use std::net::ToSocketAddrs;
use std::str::FromStr;

static EXAMPLE_SERVER_COMMAND: &'static str = "space_coop server -p 8888";
static EXAMPLE_CLIENT_COMMAND: &'static str = "space_coop client -p 9999 -s 192.168.0.1:8888";
static EXAMPLE_CLIENT_DEPS_COMMAND: &'static str = "space_coop client-deps -p 9999 -s \
                                                    192.168.0.1:8888";

fn main() {
  let matches = App::new("space coop")
    .usage(format!("\t{}\n\t{}", EXAMPLE_SERVER_COMMAND, EXAMPLE_CLIENT_COMMAND).as_ref())
    .settings(&[SubcommandRequired])
    .subcommand(SubCommand::with_name("server")
      .usage(EXAMPLE_SERVER_COMMAND)
      .arg(Arg::with_name("port")
        .short("p")
        .long("port")
        .help("Server's port")
        .takes_value(true)
        .default_value("7090")
        .value_name("PORT"))
      .arg(Arg::with_name("use upnp")
        .short("u")
        .long("use_upnp")
        .takes_value(false)
        .help("Use UPNP to automatically port forward")))
    .subcommand(SubCommand::with_name("client")
      .usage(EXAMPLE_CLIENT_COMMAND)
      .arg(Arg::with_name("port")
        .short("p")
        .long("port")
        .help("Client's port")
        .takes_value(true)
        .default_value("7190")
        .value_name("PORT"))
      .arg(Arg::with_name("server address")
        .short("s")
        .help("Server's address and port")
        .long("server_address")
        .value_name("ADDRESS:PORT")
        .takes_value(true)
        .default_value("127.0.0.1:7090")
        .required(true)))
    .subcommand(SubCommand::with_name("client-deps")
      .usage(EXAMPLE_CLIENT_DEPS_COMMAND)
      .arg(Arg::with_name("output file")
        .short("d")
        .help("Dependency data output file")
        .long("debug_output")
        .value_name("FILE")
        .takes_value(true)
        .default_value("dependencies.txt")
        .required(true))
      .arg(Arg::with_name("dependency mode")
        .short("m")
        .help("Dependency output mode")
        .long("output")
        .value_name("MODE")
        .takes_value(true)
        .possible_value("dag")
        .possible_value("list")
        .default_value("dag")
        .required(true)))
    .get_matches();

  if let Some(server_matches) = matches.subcommand_matches("server") {
    prototype2::server::start(port_from(&server_matches),
                              server_matches.occurrences_of("use upnp") > 0)
  } else if let Some(client_matches) = matches.subcommand_matches("client") {
    prototype2::client::start(port_from(&client_matches), addr_from(&client_matches))
  } else if let Some(client_deps_matches) = matches.subcommand_matches("client-deps") {
    prototype2::client::dependencies(output_file_from(&client_deps_matches),
                                     dependency_mode_from(&client_deps_matches))
  }
}

fn port_from(matches: &ArgMatches) -> u16 {
  matches.value_of("port").and_then(|v| u16::from_str(&v).ok()).unwrap()
}

fn addr_from(matches: &ArgMatches) -> std::net::SocketAddr {
  matches.value_of("server address")
    .and_then(|v| v.to_socket_addrs().ok())
    .and_then(|mut socket_addr_iter| socket_addr_iter.next())
    .unwrap()
}

fn output_file_from(matches: &ArgMatches) -> String {
  matches.value_of("output file").map(|v| v.to_owned()).unwrap()
}

fn dependency_mode_from(matches: &ArgMatches) -> prototype2::client::DependencyMode {
  matches.value_of("dependency mode")
    .and_then(|v| prototype2::client::DependencyMode::try_from(v.to_owned()).ok())
    .unwrap()
}
