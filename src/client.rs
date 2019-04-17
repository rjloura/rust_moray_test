/*
 * Copyright 2019 Joyent, Inc.
 */

/* TODO: rust-cueball */
use rust_fast::client as mod_client;
use serde_json::{self, Value};
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::str::FromStr;

use super::buckets;
use super::objects;

#[derive(Debug)]
pub struct MorayClient {
    stream: TcpStream,
}

///
/// MorayClient
///
impl MorayClient {
    pub fn new<S: Into<SocketAddr>>(address: S) -> Result<MorayClient, Error> {
        match TcpStream::connect(address.into()) {
            Ok(st) => Ok(MorayClient { stream: st }),
            Err(e) => Err(e),
        }
    }

    pub fn from_parts<I: Into<IpAddr>>(
        ip: I,
        port: u16,
    ) -> Result<MorayClient, Error> {
        match TcpStream::connect(SocketAddr::new(ip.into(), port)) {
            Ok(st) => Ok(MorayClient { stream: st }),
            Err(e) => Err(e),
        }
    }

    pub fn list_buckets<F>(&mut self, bucket_handler: F) -> Result<(), Error>
    where
        F: FnMut(&buckets::Bucket) -> Result<(), Error>,
    {
        buckets::list_buckets(&mut self.stream, bucket_handler)?;
        Ok(())
    }

    pub fn find_objects<F>(
        &mut self,
        bucket: &str,
        filter: &str,
        opts: &str,
        object_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&objects::MorayObject) -> Result<(), Error>,
    {
        objects::find_objects(
            &mut self.stream,
            bucket,
            filter,
            opts,
            object_handler,
        )?;
        Ok(())
    }

    /*
     * TODO:
     *      * 'opts' should be a defined struct
     *      * Put this in a 'meta' module
     */
    pub fn sql<F>(
        &mut self,
        stmt: &str,
        vals: Vec<&str>,
        opts: &str,
        mut query_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&Value) -> Result<(), Error>,
    {
        let options: Value = serde_json::from_str(opts).unwrap();
        let values: Value = json!(vals);
        let args: Value = json!([stmt, values, options]);

        match mod_client::send(String::from("sql"), args, &mut self.stream)
            .and_then(|_| {
                mod_client::receive(&mut self.stream, |resp| {
                    dbg!(&resp.data.d);
                    query_handler(&resp.data.d)
                })
            }) {
            Ok(_s) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl FromStr for MorayClient {
    type Err = Error;
    fn from_str(s: &str) -> Result<MorayClient, Error> {
        let addr = SocketAddr::from_str(s).expect("Error parsing address");
        match TcpStream::connect(addr) {
            Ok(st) => Ok(MorayClient { stream: st }),
            Err(e) => Err(Error::new(ErrorKind::NotConnected, e)),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn placeholder() {
        assert_eq!(1, 1);
    }
}
