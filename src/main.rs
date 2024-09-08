use teloxide::{prelude::*, utils::command::BotCommands};
use sea_orm::{self, ConnectOptions, Database, DatabaseConnection};

use dotenv::dotenv;
use std::env;

static DB:tokio::sync::OnceCell<DatabaseConnection> = tokio::sync::OnceCell::const_new();

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting emias_repr bot...");
    println!("Starting emias_repr bot...");

    dotenv().ok();

    let token = env::var("TOKEN").expect("Doesn't provide `.env` file or `TOKEN` value");

    DB.get_or_init(|| async {
        let opt = ConnectOptions::new("sqlite://db.sqlite?mode=rwc");
        Database::connect(opt).await.unwrap()
    }).await;

    let bot = Bot::new(token);

    EmCommand::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase", description="These commands are supported:")]
enum EmCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a OMS card.")]
    OmsCard(String),
    #[command(description = "handle a date of birth in format DD.MM.YYYY.")]
    DateBirth(String),
}

async fn answer(bot: Bot, msg: Message, cmd: EmCommand) -> ResponseResult<()> {
    match cmd {
        EmCommand::Help => bot.send_message(msg.chat.id, EmCommand::descriptions().to_string()).await?,
        EmCommand::OmsCard(oms) => {
            bot.send_message(msg.chat.id, format!("Your oms is {oms}.")).await?
        }
        EmCommand::DateBirth(date) => {
            bot.send_message(msg.chat.id, format!("Your date of birth is {date}."))
                .await?
        }
    };

    Ok(())
}