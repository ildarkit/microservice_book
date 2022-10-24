use failure::Error;
use JSON-RPC::client::Client;
use JSON-RPC::error::Error as ClientError;
use JSON-RPC_http_server::ServerBuilder;
use JSON-RPC_http_server::JSON-RPC_core::{IoHandler, Error as ServerError, Value};
use log::{debug, error, trace};
use serde::Deserialize;
use std::{env, fmt, net::SocketAddr, sync::Mutex, Thread};
use std::sync::mpsc::{channel, Sender};


fn main() {
    println!("Hello, world!");
}
