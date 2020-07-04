//! Stores JSON data into Redis

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use redis_async::client::{self, PairedConnection};
use redis_async::resp_array;
use serde_json::{self, json, Value};

mod error;

use crate::error::{Error, Result};

async fn async_put_json<'a>(conn: &'a Arc<PairedConnection>, key: &'a str, value: &'a Value)
    -> Result<()>
{
    let value_str = value.to_string();
    let _ret = conn.send::<String>(resp_array!["SET", key, value_str]).await?;
    println!("async_put_json");
    Ok(())
}

async fn async_check_key<'a>(conn: &'a Arc<PairedConnection>, key: &'a str) -> Result<bool> {
    let key_exist = conn.send(resp_array!["EXISTS", key]).await?;
    println!("async_check_key");
    Ok(key_exist)
}

async fn async_fetch_json<'a>(conn: &'a Arc<PairedConnection>, key: &'a str) -> Result<Value> {
    let ret: String = conn.send(resp_array!["GET", key]).await?;
    println!("async_fetch_json");
    Ok(serde_json::from_str(&ret)?)
}

async fn run_client(addr: SocketAddr) -> Result<()> {
    let conn = Arc::new(client::paired_connect(&addr).await?);

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

    async_put_json(&conn, KEY, &input_value).await?;

    let key_exists = async_check_key(&conn, KEY).await?;
    if !key_exists {
        return Err(Error::InternalError("The key does NOT exist.".to_owned()));
    }

    let output_value = async_fetch_json(&conn, KEY).await?;
    println!("{:?}", output_value);
    assert_eq!(input_value, output_value, "The output JSON is not the same as the input one");

    Ok(())
}

#[tokio::main]
async fn main() -> error::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string())
        .parse()?;

    run_client(addr).await.expect("run_client error");

    Ok(())
}
