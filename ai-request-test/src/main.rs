use std::{println, vec};

use tokio::{self};
use reqwest::{self};
use dotenvy;
use serde_json::{self, json};

#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("NO API KEY");
    let mut messages: Vec<serde_json::Value> = vec![];

    loop{
        println!("input:");
        let mut question = String::new();
        std::io::stdin()
            .read_line(&mut question)
            .expect("Read is Error");
        messages.push(json!({"role": "user", "content": question.trim()}));
        let body = serde_json::json!({
            "model": "openai/gpt-5.6-luna",
            "messages": messages
        });
    
        let client = reqwest::Client::new();
        let res = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)  // ← serde_jsonのValueやstructをそのまま渡せる
            .send()
            .await?;
    
        let reqwest_json: serde_json::Value = res.json().await?;
        let content = reqwest_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("(パーズ失敗)");
        messages.push(json!({"role": "assistant", "content": content}));
        println!("{}",content);
    }    
}
