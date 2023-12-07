use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::http_component;

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub kids: Vec<i32>,
    pub weight: Vec<i32>,
    pub capacity: i32,
}

#[derive(Debug)]
pub struct Home {
    pub kids: i32,
    pub weight: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub kids: i32,
}

/// A simple Spin HTTP component.
#[http_component]
async fn handle_server(req: Request) -> anyhow::Result<Response> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    if *req.method() != Method::Post {
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "text/plain")
            .body("Method not allowed")
            .build());
    }

    let body: Vec<u8> = req.into_body();

    let input: Input = serde_json::from_slice(body.as_slice())?;

    println!("Input: {:?}", input);

    let mut homes = input
        .kids
        .iter()
        .enumerate()
        .map(|(i, x)| {
            return Home {
                kids: *x,
                weight: input.weight[i],
            };
        })
        .filter(|h| h.kids > 0)
        .collect::<Vec<_>>();

    homes.sort_by(|a, b| {
        let a_ratio = a.weight / a.kids;
        let b_ratio = b.weight / b.kids;

        if a_ratio < b_ratio {
            return std::cmp::Ordering::Less;
        } else if a_ratio > b_ratio {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Equal;
        }
    });

    let mut kid_numbers = 0;
    let mut capacity = input.capacity;

    homes.iter().for_each(|h| {
        if capacity >= h.weight {
            capacity -= h.weight;
            kid_numbers += h.kids;
        }
    });

    let result = Output { kids: kid_numbers };
    let r = serde_json::to_string(&result)?;

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(format!("{}", r))
        .build())
}
