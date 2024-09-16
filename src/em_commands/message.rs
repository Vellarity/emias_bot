use crate::{EmCommand, DB};
use sea_orm::{prelude::*, ActiveValue};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::entities::{info, prelude::*};

use chrono::NaiveDate;


pub async fn help(bot: Bot, msg: Message) {
    bot.send_message(msg.chat.id, EmCommand::descriptions().to_string()).await.unwrap();
}

pub async fn start(bot: Bot, msg: Message) {
    let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
    
    match q {
        Some(_) => { bot.send_message(msg.chat.id, "Пользователь с вашими данными найден. Обновление базы не требуется.").await.unwrap(); },
        None => {
            println!("user:{} \nchat:{}", &msg.chat.id.0, &msg.from.clone().unwrap().id.0);

            let res = Info::insert(info::ActiveModel{
                chat_id: ActiveValue::Set(msg.chat.id.0),
                ..Default::default()
            }).exec(DB.get().unwrap()).await;

            match res {
                Ok(_) => { bot.send_message(msg.chat.id, "Пользователь с вашими данными не найден. Инициализирована новая запись. Используйте команду `/help` для получения справки.").await.unwrap(); },
                Err(_) =>{ bot.send_message(msg.chat.id, "Не удалось инициализировать запись. Попробуйте позже или обратитесь к автору этого ужаса за помощью.").await.unwrap(); }
            }
        }
    }
}

pub async fn oms_card(bot: Bot, msg: Message, oms:String) {
    if oms.len() != 16 || oms.parse::<i64>().is_err() {
        bot.send_message(msg.chat.id, format!("Полис должен быть указан в формате 16 чисел без дополнительных символов и пробелов.")).await.unwrap();
    }
    let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
    match q {
        Some(v) => {
            let mut nv: info::ActiveModel = v.into();
            nv.oms_card = ActiveValue::Set(Some(oms.parse::<i64>().unwrap()));
            let updated = nv.update(DB.get().unwrap()).await;
            match updated {
                Ok(_) => { 
                    bot.send_message(msg.chat.id, format!("Ваш новый полис ОМС {oms}.")).await.unwrap(); 
                },
                Err(_) => { 
                    bot.send_message(msg.chat.id, format!("Не удалось обновить ваш полис. Попробуйте позже или обратитесь к автору этого безобразия.")).await.unwrap(); 
                }
            }
        }
        None => { 
            bot.send_message(
                msg.chat.id, 
                format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await.unwrap(); 
            }
    }
}

pub async fn date_birth(bot: Bot, msg: Message, date:String) {
    let date_parsed = NaiveDate::parse_from_str(&date, "%d.%m.%Y");
    if date_parsed.is_err() {
        bot.send_message(msg.chat.id, format!("Дата рождения должена быть указана в формате ДД.ММ.ГГГГ без дополнительных символов и пробелов.")).await.unwrap();
    }
    let q = Info::find().filter(info::Column::ChatId.eq(msg.chat.id.0)).one(DB.get().unwrap()).await.unwrap();
    match q {
        Some(v) => {
            let mut nv: info::ActiveModel = v.into();
            nv.date_birth = ActiveValue::Set(Some(date_parsed.unwrap()));
            let updated = nv.update(DB.get().unwrap()).await;
            match updated {
                Ok(_) => {
                    bot.send_message(msg.chat.id, format!("Вашa новая дата рождения {date}.")).await.unwrap();
                },
                Err(_) => {
                    bot.send_message(msg.chat.id, format!("Не удалось обновить вашу дату рождения. Попробуйте позже или обратитесь к автору этого безобразия.")).await.unwrap();
                }
            }
        }
        None => {
            bot.send_message(msg.chat.id, format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await.unwrap();
        }
    }
} 

pub async fn info(bot: Bot, msg: Message) {
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
            ).await.unwrap();
        },
        None => { 
            bot.send_message(msg.chat.id, format!("Не найдена запись с вашим id в системе бота. Попробуйте заново использовать команду `/start` или обратитесь к автору этого ужаса, если это не помогло.")).await.unwrap();
        }
    }
}