extern crate regex;
extern crate stout_api as api;

mod stock;
mod common;
mod plot;

use tokio;

use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use serenity::client::{Client, Context, EventHandler};
use serenity::{
    utils,
    async_trait,
    model::channel::Message,
    //prelude::*,
    //utils::MessageBuilder,
    http::AttachmentType,
};

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

use std::env;

lazy_static! {
    static ref CLIENT: api::Client = api::Client::new();
    static ref SYMBOL_RE: Regex = Regex::new(r"\$([A-Z]{1,5})(\+)?(\W|$)").unwrap();
}

#[group]
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
                /*
                plot::build_chart(&stock).unwrap();
                let base64 = utils::read_image("./0.png")
                    .expect("Failed to read image");
                let img = base64;
                */
                //println!("{:?}", img);

                let company_website = stock.company.to_owned().map_or_else(|| None, |v| v.website);
                let company_website = company_website.map_or_else(|| "".to_string(), |w| format!(" | [web]({})", w));
                let msg = msg
                    .channel_id
                    .send_message(&context.http, |m| {
                        m.embed(|e| { 
                            e.title(format!("{} - 24hrs", stock.symbol));
                            e.fields(vec![
                                ("Price".to_string(), format!("${: <7.2}", stock.current_price), true),
                                ("Cap".to_string(), format!("{}", common::format_large_number(stock.market_cap.unwrap_or(0.0) as f64)), true),
                                ("Change".to_string(), format!("{:.2}%", stock.pct_change * 100.0), true),
                            ]);
                            e.fields(vec![
                                ("Low".to_string(), format!("${: <7.2}", stock.low), true),
                                ('\u{200B}'.to_string(), '\u{200B}'.to_string(), true),
                                ("High".to_string(), format!("${: <7.2}", stock.high), true),
                            ]);
                            e.description(format!("[twits](https://stocktwits.com/symbol/{}) | [yhoo](https://finance.yahoo.com/quote/{}/){}", stock.symbol, stock.symbol, company_website));

                            //e.image(AttachmentType::Image(img));
                            e
                        });

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

fn get_symbol_names(message: &str) -> Vec<&str> {
    return SYMBOL_RE
        .captures_iter(message)
        .map(|x| x.get(2).unwrap_or(None).as_str())
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

        let message = "this is a test $TEST+";
        assert_eq!(get_symbol_names(&message), ["TEST+"]);
    }
}
