use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{IncomingRequest, Method, Response},
    http_component,
    key_value::Store,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Wish {
    pub value: String,
}

/// A simple Spin HTTP component.
#[http_component]
async fn handle_data(req: IncomingRequest) -> anyhow::Result<Response> {
    println!("Handling request to {:?}", req.headers());
    let path_with_query = req.path_with_query();
    let bindings = path_with_query.ok_or("No path").unwrap();
    let parts: Vec<&str> = bindings.split("?").collect();

    println!("Handling request {:?}", parts);

    if parts.len() != 2 {
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "text/plain")
            .body("Bad request")
            .build());
    }

    let store = Store::open_default()?;

    let key = parts[1];

    if req.method() == Method::Post {
        let body: Vec<u8> = req.into_body().await?;

        let wish: Wish = serde_json::from_slice(body.as_slice())?;

        println!("Got body {:?}", wish);

        store.set(key, wish.value.as_bytes())?;

        return Ok(Response::builder()
            .status(201)
            .header("content-type", "application/json")
            .body("")
            .build());
    }

    if req.method() == Method::Get {
        let value = store.get(key)?.unwrap();

        let result = Wish {
            value: String::from_utf8(value).unwrap(),
        };

        let j = serde_json::to_string(&result)?;

        return Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(format!("{}", j))
            .build());
    }

    return Ok(Response::builder()
        .status(405)
        .header("content-type", "text/plain")
        .body("Method not allowed")
        .build());
}
