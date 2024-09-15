use chrono::NaiveDate;
use helper::collect_free_rooms_data;
use teloxide::{prelude::*, utils::command::BotCommands};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};

pub mod entities;
use entities::{info, prelude::*};

pub mod parsable;
use parsable::{basic::BasicRequest, doctors::{self, DoctorsInfoParamsRequest, DoctorsInfoParamsResponse}, referrals::{ReferralsInfoParamsRequest, ReferralsInfoResponse}};

pub mod helper;

use dotenv::dotenv;
use std::{env, sync::Arc};

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
                let ref_data = BasicRequest::<ReferralsInfoParamsRequest>::new(
                    Some("123".to_owned()), 
                    user.oms_card.unwrap().to_string(), 
                    user.date_birth.unwrap().to_string()
                );

                let ref_res = reqwest::Client::new()
                    .post("https://emias.info/api/emc/appointment-eip/v1/?getReferralsInfo")
                    .json(&ref_data)
                    .send()
                    .await;

                match ref_res {
                    Ok(res) => {
                        //let ref_data = res.json::<ReferralsInfoResponse>().await;
                        let parsed_ref_res = res.json::<ReferralsInfoResponse>().await.unwrap();
                        let mut message_string = "Ваши направления: \n".to_string();
                        for result in parsed_ref_res.result {
                            let doc_data = BasicRequest::<DoctorsInfoParamsRequest>::new(
                                Some("123".to_owned()),
                                user.oms_card.unwrap().to_string(),
                                user.date_birth.unwrap().to_string(),
                                result.id
                            );

                            message_string.push_str(
                                &format!(
                                    "[{start} - {end}] {name}\n", 
                                    start = NaiveDate::parse_from_str(&result.start_time, "%Y-%m-%d").unwrap().format("%d.%m.%Y").to_string(),
                                    end = NaiveDate::parse_from_str(&result.end_time, "%Y-%m-%d").unwrap().format("%d.%m.%Y").to_string(),
                                    name = if result.to_doctor.is_some() { result.to_doctor.unwrap().speciality_name  } else { result.to_ldp.unwrap().ldp_type_name },
                                )
                            );

                            let doc_res = reqwest::Client::new()
                                .post("https://emias.info/api/emc/appointment-eip/v1/?getDoctorsInfo")
                                .json(&doc_data)
                                .send()
                                .await;

                            match doc_res {
                                Ok(dr) => {
                                    //println!("{:#?}", dr.text().await);
                                    let parsed_doc_res = dr.json::<DoctorsInfoParamsResponse>().await.unwrap();
                                    let mut doctors_string = String::new();

                                    if let doctors::ResultType::LdpArray(result) = parsed_doc_res.result {
                                        for ldp in result {
                                            doctors_string.push_str(&format!("- {}: \n", ldp.name));
                                            let free_rooms = collect_free_rooms_data(ldp);
                                            doctors_string.push_str(&free_rooms);
                                        }
                                    } else if let doctors::ResultType::DocArray(result) = parsed_doc_res.result {
                                        for doctor in result {
                                            doctors_string.push_str(&format!("- {} {} {}: \n", doctor.main_doctor.first_name, doctor.main_doctor.second_name, doctor.main_doctor.last_name));
                                            let free_rooms = collect_free_rooms_data(doctor);
                                            doctors_string.push_str(&free_rooms);
                                        }
                                    }
                                    doctors_string += "\n";
                                    message_string.push_str(&doctors_string);
                                },
                                Err(err) => {
                                    let _ = loop_bot.send_message(ChatId(user.chat_id), format!("Не удалось получить список направлений по причине: `{}`", err.status().unwrap())).await;
                                }
                            }



                        }
                        let _ = loop_bot.send_message(
                            ChatId(user.chat_id), 
                            message_string
                        ).await;
                        
                    },
                    Err(err) => {
                        let _ = loop_bot.send_message(ChatId(user.chat_id), format!("Не удалось получить список направлений по причине: `{}`", err.status().unwrap())).await;
                    }
                }

                //let _ = loop_bot.send_message(ChatId(user.chat_id), "Test").await;
            }
        }
    });

    EmCommand::repl(main_bot, answer).await;
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

async fn answer(bot: Bot, msg: Message, cmd: EmCommand) -> ResponseResult<()> {
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

    Ok(())
}