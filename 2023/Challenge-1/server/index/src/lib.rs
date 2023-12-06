use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

/// A simple Spin HTTP component.
#[http_component]
fn handle_index(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    Ok(http::Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("<html><iframe src=\"https://giphy.com/embed/26DNbOssnkCtfv0Oc\" width=480 height=480 frameBorder=0
        class=\"giphy-embed\" allowFullScreen></iframe><p><a href=\"https://giphy.com/gifs/westminsterkennelclub-dogs-wkcdogshow-26DNbOssnkCtfv0Oc\">via GIPHY</a>
        </p><h1>Hello, Fermyon!</h1></html>")?)
}
