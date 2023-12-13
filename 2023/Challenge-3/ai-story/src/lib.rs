use anyhow::Result;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Json, Params, Request, Response, Router};
use spin_sdk::http_component;

use spin_sdk::llm::{infer_with_options, InferencingModel::Llama2Chat};

#[derive(Deserialize, Debug)]
pub struct HelvesStoryRequest {
    pub place: String,
    pub characters: Vec<String>,
    pub objects: Vec<String>,
}

#[derive(Serialize)]
pub struct HelvesStoryResponse {
    pub story: String,
}

const PROMPT: &str = r#"\
<<SYS>>
You are a helpful story teller. You immediately write a story, without introduction. You don't ask any questions.
You don't restart the discussion.
<</SYS>>
<INST>
You must include in your response the following:
Place: {PLACE}
Characters: {CHARACTERS}
Objects: {OBJECTS}
</INST>
"#;

// To test
// story: spin build ; spin cloud deploy ; curl https://ai-story-zepaisk0.fermyon.app -X POST -d '{"place": "North Pole","characters": ["Santa Claus", "The Grinch", "a pingvin"], "objects": ["A spoon", "Two presents", "Palm tree"]}'

/// A Spin HTTP component that internally routes requests.
#[http_component]
fn handle_route(req: Request) -> Response {
    let mut router = Router::new();
    router.any("/*", not_found);
    router.post("/", handle_ai_story);
    router.handle(req)
}

fn not_found(_: Request, _: Params) -> Result<impl IntoResponse> {
    Ok(Response::new(404, "Not found"))
}

/// A simple Spin HTTP component.
fn handle_ai_story(
    req: http::Request<Json<HelvesStoryRequest>>,
    _params: Params,
) -> anyhow::Result<impl IntoResponse> {
    let request = req.body();
    println!("request: {:?}", request);

    let inferencing_result = infer_with_options(
        Llama2Chat,
        &PROMPT
            .replace("{PLACE}", request.place.as_str())
            .replace("{CHARACTERS}", request.characters.join(",").as_str())
            .replace("{OBJECTS}", request.objects.join(",").as_str()),
        spin_sdk::llm::InferencingParams {
            max_tokens: 942,
            ..Default::default()
        },
    )?;

    let story_lines = inferencing_result.text.lines();

    println!("story lines: {:?}", story_lines);

    let story = clean_story(
        story_lines
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );

    println!("story: {:?}", story);

    let resp = HelvesStoryResponse { story };

    let resp_str = serde_json::to_string(&resp)?;

    Ok(Response::new(200, resp_str))
}

fn clean_story(story: &str) -> String {
    let parts = story.rsplit_once("</INST>");

    println!("parts: {:?}", parts);

    if parts.is_some() {
        return parts.unwrap().1.to_string();
    }

    return story.to_string();
}

#[cfg(test)]
mod tests {
    use crate::clean_story;

    #[test]
    fn cleanup() {
        let text = "<INST>\nPlease write a story that tree.\n</INST>\n<INST>\nPlease include reader.\n</INST>\n\nHere's the story:\n\nIt was Christmas Eve at the North ... vowing to never cause trouble again.";
        let result = clean_story(text);
        assert_eq!(result, "\n\nHere's the story:\n\nIt was Christmas Eve at the North ... vowing to never cause trouble again.");
    }
}
