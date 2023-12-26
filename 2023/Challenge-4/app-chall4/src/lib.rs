use anyhow::Error;
use serde::{Deserialize, Serialize};
use spin_sdk::http;
use spin_sdk::http::{send, IntoResponse, Request, Response};
use spin_sdk::http_component;

#[derive(Debug, Deserialize)]
pub struct GuessResponse {
    pub cows: i32,
    pub bulls: i32,
    pub gameId: String,
    pub guesses: i32,
    pub solved: bool,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub gameId: String,
    pub solved: bool,
    pub guesses: i32,
    pub solution: String,
}

struct HistoryItem {
    pub guess: [u8; 3],
    pub cows: i32,
    pub bulls: i32,
}

/// A simple Spin HTTP component.
#[http_component]
async fn handle_app_chall4(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));

    let requested_url: Option<&http::HeaderValue> = req.header("spin-full-url");

    let r = match requested_url {
        None => return Ok(Response::new(404, "Not Found")),
        Some(v) => v.as_str().unwrap(),
    };

    if !r.ends_with("/") {
        return Ok(Response::new(404, "Not Found"));
    }

    let mut history: Vec<HistoryItem> = vec![];
    let mut game_id: String = "".to_string();

    let mut limit = 10;

    while limit > 0 {
        let guess: [u8; 3] = guess_next_value(&history);

        let guess_response: GuessResponse = try_guess(guess, game_id).await?;

        game_id = guess_response.gameId.clone();

        println!("response {:?}", guess_response);

        let solved = guess_response.solved;
        let gameId = guess_response.gameId;

        history.push(HistoryItem {
            guess,
            cows: guess_response.cows,
            bulls: guess_response.bulls,
        });

        if solved {
            let resp = Output {
                gameId,
                solved,
                solution: guess.map(|x| x.to_string()).join(""),
                guesses: guess_response.guesses,
            };

            let resp_str = serde_json::to_string(&resp)?;

            return Ok(Response::new(200, resp_str));
        }

        limit -= 1;
    }

    Ok(Response::new(500, "failed to solve"))
}

async fn try_guess(guess: [u8; 3], gameId: String) -> Result<GuessResponse, Error> {
    let guess_str: String = guess.map(|x| x.to_string()).join("");

    println!("guessing {}", guess_str);

    let mut uri = format!("https://bulls-n-cows.fermyon.app/api?guess={}", guess_str);

    if gameId.len() > 0 {
        uri = format!("{}&id={}", uri, gameId);
    }

    let outbound_req = Request::get(uri);

    // Send the outbound request, capturing the response as raw bytes
    let response: http::Response = send(outbound_req).await?;

    let guess_response: GuessResponse = serde_json::from_slice(response.body())?;

    Ok(guess_response)
}

fn guess_next_value(history: &Vec<HistoryItem>) -> [u8; 3] {
    let available_digits: Vec<u8> = (0..=4).collect();

    let mut all_guesses: Vec<[u8; 3]> = vec![];
    for x in &available_digits {
        for y in &available_digits {
            for z in &available_digits {
                let are_different = x != y && x != z && y != z;
                if are_different {
                    all_guesses.push([*x, *y, *z]);
                }
            }
        }
    }

    for guess in history {
        all_guesses.retain(|g| {
            !(g[0] == guess.guess[0] && g[1] == guess.guess[1] && g[2] == guess.guess[2])
        });

        if guess.bulls == 0 && guess.cows == 1 {
            // Special case, if the cows is 1, we know the 2 other digits are in the solution
            let have_to_use: Vec<u8> = available_digits
                .clone()
                .into_iter()
                .filter(|&x| x != guess.guess[0] && x != guess.guess[1] && x != guess.guess[2])
                .collect();

            println!("have to use  {:?}", have_to_use);

            all_guesses.retain(|g: &[u8; 3]| {
                for item in &have_to_use {
                    if !g.contains(&item) {
                        return false;
                    }
                }
                return true;
            })
        }

        if guess.bulls == 0 {
            // Special case, we know we have not any of the 3 digits in these positions
            all_guesses.retain(|g: &[u8; 3]| {
                if g[0] == guess.guess[0] || g[1] == guess.guess[1] || g[2] == guess.guess[2] {
                    return false;
                }

                return true;
            })
        }

        if guess.cows == 3 {
            // Special case, we know we have all 3 digits in wrong positions
            all_guesses.retain(|g: &[u8; 3]| {
                for item in &guess.guess {
                    if !g.contains(&item) {
                        return false;
                    }
                }
                return true;
            })
        }

        if guess.cows > 0 {
            // Special case, we know we have at least some of the numbers
            all_guesses.retain(|g: &[u8; 3]| {
                let mut common_numbers = 0;

                for item in &guess.guess {
                    if g.contains(&item) {
                        common_numbers += 1;
                    }
                }
                return common_numbers == guess.cows + guess.bulls;
            })
        }

        if guess.bulls > 0 {
            // Special case, we know we have at least some of the numbers
            all_guesses.retain(|g: &[u8; 3]| {
                let mut common_numbers_at_same_position = 0;

                if g[0] == guess.guess[0] {
                    common_numbers_at_same_position += 1;
                }
                if g[1] == guess.guess[1] {
                    common_numbers_at_same_position += 1;
                }
                if g[2] == guess.guess[2] {
                    common_numbers_at_same_position += 1;
                }

                return common_numbers_at_same_position <= guess.bulls;
            })
        }
    }

    println!("all guesses {:?}", all_guesses);

    all_guesses[0]
}

#[cfg(test)]
mod tests {
    use crate::{guess_next_value, HistoryItem};

    #[test]
    fn test_guess_next_value() {
        let history: Vec<HistoryItem> = vec![HistoryItem {
            guess: [0, 1, 2],
            cows: 1,
            bulls: 0,
        }];

        let result = guess_next_value(&history);
        assert_ne!(result, [0, 1, 2]);
        assert_eq!(result, [1, 3, 4]);
    }

    #[test]
    fn test_guess_all_cows() {
        let history: Vec<HistoryItem> = vec![HistoryItem {
            guess: [0, 1, 2],
            cows: 3,
            bulls: 0,
        }];

        let result = guess_next_value(&history);
        assert_ne!(result, [0, 1, 2]);
        assert_eq!(result, [1, 2, 0]);
    }
}
