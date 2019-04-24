//! Stores JSON data into Redis

use std::env;
use std::sync::Arc;

use redis_async::client::{self, PairedConnection};
use redis_async::resp_array;
use serde_json::{self, json, Value};
use tokio::prelude::future::{Future, IntoFuture};
use tokio::runtime::Runtime;

mod error;

fn async_put_json(conn: &PairedConnection, key: &str, value: &Value)
    -> impl Future<Item = (), Error = error::Error>
{
    let value_str = value.to_string();
    conn
        .send::<String>(resp_array!["SET", key, value_str])
        .map(|_ret| {
            println!("async_put_json");
        })
        .map_err(From::from)
}

fn async_fetch_json(conn: &PairedConnection, key: &str)
    -> impl Future<Item = Value, Error = error::Error>
{
    conn
        .send(resp_array!["GET", key])
        .map_err(From::from)
        .and_then(|ret: String| {
            println!("async_fetch_json");
            serde_json::from_str(&ret)
                .map_err(From::from)
                .into_future()
        })
}

fn main() -> error::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string())
        .parse()?;

    let task = client::paired_connect(&addr)
        .map_err(From::from)
        .and_then(|conn| {
            let conn = Arc::new(conn);
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

            async_put_json(&conn, "foo", &input_value)
                .and_then(move |()| {
                    async_fetch_json(&conn2, "foo")
                })
                .map(move |output_value| {
                    println!("{:?}", output_value);
                    assert_eq!(input_value, output_value,
                        "The output JSON is not the same as the input one");
                })
        })
        .map_err(|e| {
            println!("Error occured: {:?}", e);
        });

    let mut rt = Runtime::new().unwrap();
    rt.block_on(task).unwrap();
    Ok(())
}
