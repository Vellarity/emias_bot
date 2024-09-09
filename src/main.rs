use teloxide::{prelude::*, utils::command::BotCommands};
use sea_orm::{self, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};

pub mod entities;
use entities::{info, prelude::*};

use dotenv::dotenv;
use std::env;

pub static DB:tokio::sync::OnceCell<DatabaseConnection> = tokio::sync::OnceCell::const_new();

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting emias_repr bot...");
    println!("Starting emias_repr bot...");

    dotenv().ok();
    let token = env::var("TOKEN").expect("Doesn't provide `.env` file or `TOKEN` value");
    let db_connection = env::var("DATABASE_URL").expect("Doesn't provide `.env` file or `DATABASE_URL` value");

    DB.get_or_init(|| async {
        let opt = ConnectOptions::new(db_connection);
        Database::connect(opt).await.unwrap()
    }).await;

    let bot = Bot::new(token);

    EmCommand::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase", description="Доступны данные команды:")]
enum EmCommand {
    #[command(description = "показать этот текст.")]
    Help,
    #[command(description = "инициализировать вашу запись в боте.")]
    Init,    
    #[command(description = "изменить номер ПОЛИСа.")]
    OmsCard(String),
    #[command(description = "изменить дату рождения (в формате DD.MM.YYYY).")]
    DateBirth(String),
}

async fn answer(bot: Bot, msg: Message, cmd: EmCommand) -> ResponseResult<()> {
    match cmd {
        EmCommand::Help => bot.send_message(msg.chat.id, EmCommand::descriptions().to_string()).await?,
        EmCommand::Init => {
            let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();

            match q {
                Some(_) => bot.send_message(msg.chat.id, "Пользователь с вашими данными найден. Обновление базы не требуется.").await?,
                None => bot.send_message(msg.chat.id, "Пользователь с вашими данными не найден. Инициализирована новая запись.").await?
            }
        },
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