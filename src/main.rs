use dotenv::dotenv;
use std::{env, fmt::format, sync::Arc};
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, Me}, utils::command::BotCommands};
use sea_orm::{ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};

pub mod entities;
use entities::{info, prelude::*};

pub mod parsable;

pub mod helper;
use helper::{get_referrals_obj, get_user_referrals};

pub mod em_commands;



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

    let bot = Arc::new(Bot::new(token));
    let loop_bot = bot.clone();
    let main_bot = bot.clone();

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
                            teloxide::types::InlineKeyboardButtonKind::CallbackData("get_doctors".to_string())
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
        //.branch(Update::filter_inline_query().endpoint(inline_handler));


    Dispatcher::builder(main_bot, handler).enable_ctrlc_handler().build().dispatch().await;

    //EmCommand::repl(main_bot, answer).await;
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

async fn callback_handler(bot:Bot, callback: CallbackQuery ) -> ResponseResult<()> {
    let chat_id = callback.chat_id().unwrap();
    if let Some(command) = callback.data {
        let user = Info::find().filter(info::Column::ChatId.eq(callback.from.id.0)).one(DB.get().unwrap()).await.unwrap().unwrap();

        let command_parts = command.split("/").collect::<Vec<&str>>();

        match command_parts[0] {
            "get_referrals" => {
                let refs_result = get_referrals_obj(&user).await;
                match refs_result {
                    Ok(referrals) => {
                        let away_key = InlineKeyboardButton::new("Назад", teloxide::types::InlineKeyboardButtonKind::CallbackData("back_to_main".to_string()));
                        let mut refs_keys = vec![];
                        
                        for referral in referrals.result {
                            let name = if referral.to_doctor.is_some() { referral.to_doctor.unwrap().speciality_name  } else { referral.to_ldp.unwrap().ldp_type_name };
                            refs_keys.push(
                                InlineKeyboardButton::new(name, teloxide::types::InlineKeyboardButtonKind::CallbackData(format!("get_doctors/{}", referral.id)))
                            );
                        }
                    },
                    Err(_) => {
                        bot.send_message(chat_id, "Не удалось получить список направлений").await.unwrap();
                    }
                }
            },
            _ => {}
        }
    }

    Ok(())
}

async fn message_handler(bot: Bot, msg: Message, me: Me) -> ResponseResult<()> {
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




/* 
    match cmd {
        EmCommand::Help => bot.send_message(msg.chat.id, EmCommand::descriptions().to_string()).await?,
        EmCommand::Start => {
            let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();

            match q {
                Some(_) => bot.send_message(msg.chat.id, "Пользователь с вашими данными найден. Обновление базы не требуется.").await?,
                None => {
                    println!("user:{} \nchat:{}", &msg.chat.id.0, &msg.from.clone().unwrap().id.0);
                    let res = Info::insert(info::ActiveModel{
                        chat_id: ActiveValue::Set(msg.chat.id.0),
                        ..Default::default()
                    }).exec(DB.get().unwrap()).await;
                    match res {
                        Ok(_) => bot.send_message(msg.chat.id, "Пользователь с вашими данными не найден. Инициализирована новая запись. Используйте команду `/help` для получения справки.").await?,
                        Err(_) => bot.send_message(msg.chat.id, "Не удалось инициализировать запись. Попробуйте позже или обратитесь к автору этого ужаса за помощью.").await?
                    }
                }
            }
        },
        EmCommand::OmsCard(oms) => {
            if oms.len() != 16 || oms.parse::<i64>().is_err() {
                return bot.send_message(msg.chat.id, format!("Полис должен быть указан в формате 16 чисел без дополнительных символов и пробелов.")).await.map(|_| ());
            }
            let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
            match q {
                Some(v) => {
                    let mut nv: info::ActiveModel = v.into();
                    nv.oms_card = ActiveValue::Set(Some(oms.parse::<i64>().unwrap()));
                    let updated = nv.update(DB.get().unwrap()).await;
                    match updated {
                        Ok(_) => bot.send_message(msg.chat.id, format!("Ваш новый полис ОМС {oms}.")).await?,
                        Err(_) => bot.send_message(msg.chat.id, format!("Не удалось обновить ваш полис. Попробуйте позже или обратитесь к автору этого безобразия.")).await?
                    }
                }
                None => bot.send_message(msg.chat.id, format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await?
            }
        },
        EmCommand::DateBirth(date) => {
            let date_parsed = NaiveDate::parse_from_str(&date, "%d.%m.%Y");
            if date_parsed.is_err() {
                return bot.send_message(msg.chat.id, format!("Дата рождения должена быть указана в формате ДД.ММ.ГГГГ без дополнительных символов и пробелов.")).await.map(|_| ());
            }
            let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
            match q {
                Some(v) => {
                    let mut nv: info::ActiveModel = v.into();
                    nv.date_birth = ActiveValue::Set(Some(date_parsed.unwrap()));
                    let updated = nv.update(DB.get().unwrap()).await;
                    match updated {
                        Ok(_) => bot.send_message(msg.chat.id, format!("Вашa новая дата рождения {date}.")).await?,
                        Err(_) => bot.send_message(msg.chat.id, format!("Не удалось обновить вашу дату рождения. Попробуйте позже или обратитесь к автору этого безобразия.")).await?
                    }
                }
                None => bot.send_message(msg.chat.id, format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await?
            }
        },
        EmCommand::Info => {
            let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
            match q {
                Some(v) => {
                    bot.send_message(
                        msg.chat.id, 
                        format!(
                            "Полис ОМС: {}; \nДата рождения: {}.", 
                            v.oms_card.map_or("не указан".to_string(), |s| s.to_string()), 
                            v.date_birth.map_or("не указан".to_string(), |d| d.format("%d.%m.%Y").to_string())
                        )
                    ).await?
                },
                None => bot.send_message(msg.chat.id, format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await?
            }
        }
    };


*/