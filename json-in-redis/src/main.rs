//! Stores JSON data into Redis

// The experimental native async/await support
#![feature(await_macro, async_await, futures_api)]

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use redis_async::client::{self, PairedConnection};
use redis_async::resp_array;
use serde_json::{self, json, Value};

// async/await
use tokio::await;

mod error;

use crate::error::Result;

async fn async_put_json<'a>(conn: Arc<PairedConnection>, key: &'a str, value: &'a Value)
    -> Result<()>
{
    let value_str = value.to_string();
    let _ret = await!(conn.send::<String>(resp_array!["SET", key, value_str]))?;
    println!("async_put_json");
    Ok(())
}

async fn async_fetch_json(conn: Arc<PairedConnection>, key: &str) -> Result<Value> {
    let ret: String = await!(conn.send(resp_array!["GET", key]))?;
    println!("async_fetch_json");
    Ok(serde_json::from_str(&ret)?)
}

async fn run_client(addr: SocketAddr) -> Result<()> {
    let conn = Arc::new(await!(client::paired_connect(&addr))?);
    let conn2 = Arc::clone(&conn);

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

    await!(async_put_json(conn, "foo", &input_value))?;

    let output_value = await!(async_fetch_json(conn2, "foo"))?;
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
