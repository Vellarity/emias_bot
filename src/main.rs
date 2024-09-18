use dotenv::dotenv;
use em_commands::callback::{back_to_main, get_doctors, get_referrals};
use std::{env, error::Error};
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, Me}, utils::command::BotCommands};
use sea_orm::{ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};

pub mod entities;
use entities::{info, prelude::*};

pub mod parsable;

pub mod helper;
use helper::get_user_referrals;

pub mod em_commands;

pub static DB:tokio::sync::OnceCell<DatabaseConnection> = tokio::sync::OnceCell::const_new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting emias_repr bot...");

    dotenv().ok();
    let token = env::var("TOKEN").expect("Doesn't provide `.env` file or `TOKEN` value");
    let db_connection = env::var("DATABASE_URL").expect("Doesn't provide `.env` file or `DATABASE_URL` value");

    DB.get_or_init(|| async {
        let opt = ConnectOptions::new(db_connection);
        Database::connect(opt).await.unwrap()
    }).await;

    let bot = Bot::new(token);
    let loop_bot = bot.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60*30));

        loop {
            interval.tick().await;

            let users_to_send = Info::find()
                .filter(info::Column::OmsCard.is_not_null())
                .filter(info::Column::DateBirth.is_not_null())
                .all(DB.get().unwrap())
                .await.expect("Не могу прочитать БАЗУ.");

            for user in users_to_send {
                let message = get_user_referrals(&user).await;

                match message {
                    Ok(message) => {
                        let go_to_ref_button = InlineKeyboardButton::new(
                            "Записаться", 
                            teloxide::types::InlineKeyboardButtonKind::CallbackData("get_referrals".to_string())
                        );
                        let markup = InlineKeyboardMarkup::new([[go_to_ref_button]]);

                        let _ = loop_bot.send_message(
                            ChatId(user.chat_id), 
                            message
                        ).reply_markup(markup).await;
                    },
                    Err(err) => {
                        let _ = loop_bot.send_message(
                            ChatId(user.chat_id), 
                            format!(
                                "Не удалось получить список направлений: \n-Ошибка в запросе по ссылке: `{}`; \n-Код ответа: `{}`.", 
                                err.url().unwrap(), 
                                err.status().unwrap()
                            )
                        ).await;
                    }
                }
            }
        }
    });

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot.clone(), handler).enable_ctrlc_handler().build().dispatch().await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase", description="Доступны данные команды:")]
enum EmCommand {
    #[command(description = "показать этот текст.")]
    Help,
    #[command(description = "инициализировать вашу запись в боте.")]
    Start,    
    #[command(description = "изменить номер ПОЛИСа.")]
    OmsCard(String),
    #[command(description = "изменить дату рождения (в формате DD.MM.YYYY).")]
    DateBirth(String),
    #[command(description = "показать актуальную инфомрацию обо мне в системе.")]
    Info
}

async fn callback_handler(bot: Bot, callback: CallbackQuery ) -> Result<(), Box<dyn Error + Send + Sync>> {

    let chat_id = callback.chat_id().unwrap();
    let message_id = callback.message.unwrap().id();
    //let chat_id = callback.chat_id().unwrap();
    if let Some(command) = callback.data {
        let user = Info::find().filter(info::Column::ChatId.eq(callback.from.id.0)).one(DB.get().unwrap()).await.unwrap().unwrap();

        let command_parts = command.split("/").collect::<Vec<&str>>();

        match command_parts[0] {
            "get_referrals" => {
                get_referrals(bot, user, chat_id, message_id).await;
            },
            "back_to_main" => {
                back_to_main(bot, chat_id, message_id).await;
            },
            "get_doctors" => {
                let referral_id = command_parts[1].parse().unwrap();
                get_doctors(bot, user, chat_id, message_id, &referral_id).await;
            }
            _ => {

            }
        }
    }
    

    Ok(())
}

async fn message_handler(bot: Bot, msg: Message, me: Me) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        let cmd = BotCommands::parse(text, me.username()).unwrap();

        match cmd {
            EmCommand::Help => {
                em_commands::message::help(bot, msg).await;
            },
            EmCommand::Start => {
                em_commands::message::start(bot, msg).await;
            },
            EmCommand::OmsCard(oms) => {
                em_commands::message::oms_card(bot, msg, oms).await;
            },
            EmCommand::DateBirth(date) => {
                em_commands::message::date_birth(bot, msg, date).await;
            },
            EmCommand::Info => {
                em_commands::message::info(bot, msg).await;
            }
        };
    }
    

    Ok(())
}