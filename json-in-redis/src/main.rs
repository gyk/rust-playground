//! Stores JSON data into Redis

// The experimental native async/await support
#![feature(await_macro, async_await)]

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use redis_async::client::{self, PairedConnection};
use redis_async::resp_array;
use serde_json::{self, json, Value};

// async/await
use tokio::await;

mod error;

use crate::error::{Error, Result};

async fn async_put_json<'a>(conn: &'a Arc<PairedConnection>, key: &'a str, value: &'a Value)
    -> Result<()>
{
    let value_str = value.to_string();
    let _ret = await!(conn.send::<String>(resp_array!["SET", key, value_str]))?;
    println!("async_put_json");
    Ok(())
}

async fn async_check_key<'a>(conn: &'a Arc<PairedConnection>, key: &'a str) -> Result<bool> {
    let key_exist = await!(conn.send(resp_array!["EXISTS", key]))?;
    println!("async_check_key");
    Ok(key_exist)
}

async fn async_fetch_json<'a>(conn: &'a Arc<PairedConnection>, key: &'a str) -> Result<Value> {
    let ret: String = await!(conn.send(resp_array!["GET", key]))?;
    println!("async_fetch_json");
    Ok(serde_json::from_str(&ret)?)
}

async fn run_client(addr: SocketAddr) -> Result<()> {
    let conn = Arc::new(await!(client::paired_connect(&addr))?);

    let input_value = json!({
        "code": 200,
        "success": true,
        "payload": {
            "features": [
                "serde",
                "json"
            ]
        }
    });

    const KEY: &str = "foo";

    await!(async_put_json(&conn, KEY, &input_value))?;

    let key_exists = await!(async_check_key(&conn, KEY))?;
    if !key_exists {
        return Err(Error::InternalError("The key does NOT exist.".to_owned()));
    }

    let output_value = await!(async_fetch_json(&conn, KEY))?;
    println!("{:?}", output_value);
    assert_eq!(input_value, output_value, "The output JSON is not the same as the input one");

    Ok(())
}

fn main() -> error::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string())
        .parse()?;

    tokio::run_async(async move {
        await!(run_client(addr)).expect("run_client error")
    });

    Ok(())
}
