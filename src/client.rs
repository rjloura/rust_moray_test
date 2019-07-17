/*
 * Copyright 2019 Joyent, Inc.
 */

use cueball::backend::{Backend};
use cueball::connection_pool::types::ConnectionPoolOptions;
use cueball::connection_pool::ConnectionPool;
use cueball_static_resolver::StaticIpResolver;
use cueball_tcp_stream_connection::TcpStreamWrapper;

use std::ops::{DerefMut};
use slog::{o, Logger, Drain};
use std::sync::Mutex;

use std::str::FromStr;

use serde_json::{self, Value};
use std::io::Error;

use std::net::{IpAddr, SocketAddr};

use super::buckets;
use super::meta;
use super::objects;


pub struct MorayClient
{
    connection_pool:
        ConnectionPool<TcpStreamWrapper, StaticIpResolver, fn(&Backend)
            -> TcpStreamWrapper>
}

///
/// MorayClient
///
impl MorayClient {

    pub fn new(
            address: SocketAddr,
            log :Logger) -> Result<MorayClient, Error> {

        let primary_backend = (address.ip(), address.port());
        let resolver = StaticIpResolver::new(vec![primary_backend]);

        let pool_opts = ConnectionPoolOptions {
            maximum: 5,
            claim_timeout: Some(5000),
            log: log,
            rebalancer_action_delay: None,
        };

        let pool =
            ConnectionPool::<TcpStreamWrapper, StaticIpResolver, fn(&Backend)
                -> TcpStreamWrapper>::new(
                    pool_opts,
                    resolver,
                    TcpStreamWrapper::new,
        );


        Ok(MorayClient {
            connection_pool: pool,
        })
    }

    pub fn from_parts<I: Into<IpAddr>>(
        ip: I,
        port: u16,
        log: Logger,
    ) -> Result<MorayClient, Error> {
        Self::new(SocketAddr::new(ip.into(), port), log)
    }

    pub fn list_buckets<F>(
        &mut self,
        opts: buckets::MethodOptions,
        bucket_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&buckets::Bucket) -> Result<(), Error>,
    {
        let mut conn = self.connection_pool.claim().unwrap();

        buckets::get_list_buckets(
            &mut (*conn).deref_mut(),
            "",
            opts,
            buckets::Methods::List,
            bucket_handler,
        )
    }

    pub fn get_bucket<F>(
        &mut self,
        name: &str,
        opts: buckets::MethodOptions,
        bucket_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&buckets::Bucket) -> Result<(), Error>,
    {
        let mut conn = self.connection_pool.claim().unwrap();

        buckets::get_list_buckets(
            &mut (*conn).deref_mut(),
            name,
            opts,
            buckets::Methods::Get,
            bucket_handler,
        )
    }

    pub fn get_object<F>(
        &mut self,
        bucket: &str,
        key: &str,
        opts: &objects::MethodOptions,
        object_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&objects::MorayObject) -> Result<(), Error>,
    {
        let mut conn = self.connection_pool.claim().unwrap();

        objects::get_find_objects(
            &mut (*conn).deref_mut(),
            bucket,
            key,
            opts,
            objects::Methods::Get,
            object_handler,
        )
    }

    pub fn find_objects<F>(
        &mut self,
        bucket: &str,
        filter: &str,
        opts: &objects::MethodOptions,
        object_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&objects::MorayObject) -> Result<(), Error>,
    {
        let mut conn = self.connection_pool.claim().unwrap();
        objects::get_find_objects(
            &mut (*conn).deref_mut(),
            bucket,
            filter,
            opts,
            objects::Methods::Find,
            object_handler,
        )
    }

    pub fn put_object<F>(
        &mut self,
        bucket: &str,
        key: &str,
        value: Value,
        opts: &objects::MethodOptions,
        object_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&str) -> Result<(), Error>,
    {
        let mut conn = self.connection_pool.claim().unwrap();
        objects::put_object(
            &mut (*conn).deref_mut(),
            bucket,
            key,
            value,
            opts,
            object_handler,
        )
    }

    pub fn create_bucket(
        &mut self,
        name: &str,
        config: Value,
        opts: buckets::MethodOptions,
    ) -> Result<(), Error> {
        buckets::create_bucket(
            &mut self.connection_pool.claim().unwrap().deref_mut(),
            name,
            config,
            opts)
    }

    pub fn sql<F, V>(
        &mut self,
        stmt: &str,
        vals: Vec<&str>,
        opts: V,
        query_handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&Value) -> Result<(), Error>,
        V: Into<Value>,
    {
        meta::sql(&mut self.connection_pool.claim().unwrap().deref_mut(),
        stmt,
        vals,
        opts,
        query_handler)
    }
}

 impl FromStr for MorayClient {
     type Err = Error;
     fn from_str(
         s: &str,
     ) -> Result<MorayClient, Error> {
        let log = Logger::root(
        Mutex::new(slog_bunyan::default(std::io::stdout()),).fuse(),
        o!("build-id" => "0.1.0"),
    );

         let addr = SocketAddr::from_str(s).expect("Error parsing address");
         Self::new(addr, log)
     }
 }

#[cfg(test)]
mod tests {

    #[test]
    fn placeholder() {
        assert_eq!(1, 1);
    }
}
