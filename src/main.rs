extern crate regex;
extern crate stout_api as api;

mod stock;
mod common;
mod plot;

use tokio;

use lazy_static::lazy_static;
use regex::Regex;
use serenity::client::{Client, Context, EventHandler};
use serenity::{
    async_trait,
    model::channel::Message,
    //prelude::*,
    //utils::MessageBuilder,
    //http::AttachmentType,
};

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

use std::env;

lazy_static! {
    static ref CLIENT: api::Client = api::Client::new();
    static ref SYMBOL_RE: Regex = Regex::new(r"\$([A-Z]{1,5})(\W|$)").unwrap();
}

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if (msg.author.name != "Stout") {
            let symbol_names = get_symbol_names(&msg.content);

            let stocks = symbol_names.into_iter().map(|x| stock::Stock::new(&x));

            for handle in stocks {
                let stock = handle.await;
                let msg = msg
                    .channel_id
                    .send_message(&context.http, |m| {
                        m.embed(|e| { 
                            e.title(format!("{} - 24hrs", stock.symbol));
                            e.fields(vec![
                                ("Price".to_string(), format!("${: <7.2}", stock.current_price), true),
                                ('\u{200B}'.to_string(), '\u{200B}'.to_string(), true),
                                ("Change".to_string(), format!("{:.2}%", stock.pct_change * 100.0), true),
                            ]);
                            e.fields(vec![
                                ("Low".to_string(), format!("${: <7.2}", stock.low), true),
                                ('\u{200B}'.to_string(), '\u{200B}'.to_string(), true),
                                ("High".to_string(), format!("${: <7.2}", stock.high), true),
                            ]);

                            e
                        });
                        //plot::build_chart(stock).unwrap();
                        //m.add_file(AttachmentType::Path(Path::new("./ferris_eyes.png")));
                        m
                    })
                    .await;
                if let Err(why) = msg {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

fn get_symbol_names(message: &str) -> Vec<&str> {
    return SYMBOL_RE
        .captures_iter(message)
        .map(|x| return x.get(1).unwrap().as_str())
        .collect::<Vec<&str>>();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_pulls_correct_symbol_names() {
        let message = "This is a $TEST $20.20 to $ASDFAA $asdf see if $TICKS names $N works.";
        assert_eq!(get_symbol_names(&message), ["TEST", "TICKS", "N"]);

        let message = "$TEST";
        assert_eq!(get_symbol_names(&message), ["TEST"]);
    }
}
