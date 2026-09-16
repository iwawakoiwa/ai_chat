use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serenity::model::id::ChannelId;

struct Handler {
    histories: Arc<Mutex<HashMap<ChannelId, Vec<serde_json::Value>>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let question: String = if msg.content.starts_with("!ai ") {
            msg.content.strip_prefix("!ai ").unwrap().to_string()
        } else if msg.mentions.iter().any(|u| u.id == ctx.cache.current_user().id) {
            let bot_id = ctx.cache.current_user().id;
            let mention1 = format!("<@{}>", bot_id);
            let mention2 = format!("<@!{}>", bot_id);
            msg.content
                .replace(&mention1, "")
                .replace(&mention2, "")
                .trim()
                .to_string()
        } else {
            return;
        };

        let api_key = std::env::var("API_KEY").expect("NO API KEY");

        println!("質問開始!:{}",question);
        let mut histories = self.histories.lock().await;
        let histroty = histories.entry(msg.channel_id).or_insert_with(Vec::new);
        histroty.push(serde_json::json!({"role": "user", "content": question}));
        let body = serde_json::json!({
            "model": "openai/gpt-5.6-luna",
            "messages":*histroty
        });
        
        let client = reqwest::Client::new();
        let res = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)  // ← serde_jsonのValueやstructをそのまま渡せる
            .send()
            .await;
        match res {
            Ok(v) => {
                match v.json::<serde_json::Value>().await {
                    Ok(reqwest_json) => {
                        let content = reqwest_json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("(パーズ失敗)");
                        histroty.push(serde_json::json!({"role": "assistant", "content": content}));
                        let _ = msg.channel_id.say(&ctx.http, content).await;
                    }
                    Err(e) => {
                        println!("JSONパース失敗: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("むりでした:{}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("NO DISCORD TOKEN");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let handler = Handler {
    histories: Arc::new(Mutex::new(HashMap::new())),
    };
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await?;

    client.start().await?;
    Ok(())
}