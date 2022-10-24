use failure::Error;
use jsonrpc::client::Client;
use jsonrpc::error::Error as ClientError;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Error as ServerError, Value};
use log::{debug, error, trace};
use serde::Deserialize;
use std::{env, fmt, net::SocketAddr, sync::Mutex, thread};
use std::sync::mpsc::{channel, Sender};


fn main() {
    println!("Hello, world!");
}
