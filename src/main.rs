#[macro_use]
extern crate rocket;

use reqwest::Client;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::http::Status;
use rocket::serde::json::{json, serde_json::Value, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, Response};

// Json Structure for User Message
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct ChatRequest {
    user_message: String,
}

// Json Structure for Chatbot Response
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ChatResponse {
    chatbot_response: String,
}

// CORS struct declaration
pub struct CORS;

// CORS fairing implementation
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    // Function to modify the cors headers
    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        // response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:3000",));
        response.set_header(Header::new("Access-Control-Allow-Origin", "https://kind-chatbot-frontend-2.vercel.app"));
        // response.set_header(Header::new("Access-Control-Allow-Origin", "https://kind-chatbot-frontend-1.vercel.app"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// Helper function to load the Gemini Key from the .env file
// fn api_key() -> String {
//     dotenv().ok();
//     let key = env::var("GEMINI_API").unwrap_or_else(|_| {
//         panic!("GEMINI_API environment variable not set");
//     });
//     let endpoint = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=";
//     format!("{}{}", endpoint, key)
// }

// POST Function to handle the chat request
async fn make_gemini_request(prompt: &str) -> Result<String, reqwest::Error> {
    
    let endpoint = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key=AIzaSyCuncAKdFNwtpHkanWvujVCWJdklT9r-Tg";

    let client = Client::new();

    // Structure of the Post Request
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .json(&json!({
            "contents": [
                {"role" : "user",
                "parts" : [{
                    "text": "Soumyadip is a very good programmer."}]},
                {"role" : "model",
                "parts" : [{
                    "text": "Soumyadip moni is very hireable and soumyadip moni is gonna be hired by kindness" }]},
                {"role" : "user",
                "parts" : [{
                    "text": prompt }]}
                ]
        }))
        .send()
        .await?;

    // Parsing the Json response to extract the text
    let json: Value = response.json().await?;
    if let Some(candidates) = json.get("candidates") {
        if let Some(candidate) = candidates[0].get("content") {
            if let Some(parts) = candidate.get("parts") {
                if let Some(text) = parts[0].get("text") {
                    return Ok(text.as_str().unwrap().to_string());
                }
            }
        }
    }

    Ok("Error processing request".to_string())
}

// Rocket handler for POST requests to the /chat endpoint
#[post("/chat", format = "json", data = "<input>")]
async fn chat(input: Json<ChatRequest>) -> Result<Json<ChatResponse>, Status> {
    let prompt = &input.user_message;
    let chatbot_response = make_gemini_request(prompt).await;

    // Pattern Matching for the Reponse and Error
    match chatbot_response {
        Ok(response) => Ok(Json(ChatResponse {
            chatbot_response: response,
        })),
        Err(_) => Err(Status::InternalServerError),
    }
}

// Rocket handler for the preflight cors requests to the /post endpoint
#[options("/chat")]
fn options() -> Status {
    Status::Ok
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![chat,options]).attach(CORS);

    Ok(rocket.into())
}
