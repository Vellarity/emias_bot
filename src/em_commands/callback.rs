use crate::{entities::info::Model, helper::{get_doctors_obj, get_referrals_obj}, parsable::doctors};
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId}};

pub async fn get_referrals(bot: Bot, user: Model, chat_id:ChatId, message_id:MessageId) {
    let refs_result = get_referrals_obj(&user).await;
    match refs_result {
        Ok(referrals) => {
            let away_key = InlineKeyboardButton::new("Назад", teloxide::types::InlineKeyboardButtonKind::CallbackData("back_to_main".to_string()));
            let mut refs_keys = vec![];
            
            for referral in referrals.result {
                let name = if referral.to_doctor.is_some() { referral.to_doctor.unwrap().speciality_name  } else { referral.to_ldp.unwrap().ldp_type_name };
                refs_keys.push(
                    [InlineKeyboardButton::new(name, teloxide::types::InlineKeyboardButtonKind::CallbackData(format!("get_doctors/{}", referral.id)))]
                );
            }

            refs_keys.push([away_key]);

            let markup = InlineKeyboardMarkup::new(refs_keys);

            bot.edit_message_reply_markup(chat_id, message_id).reply_markup(markup).await.unwrap();
        },
        Err(_) => {
            bot.send_message(chat_id, "Не удалось получить список направлений").await.unwrap();
        }
    }
} 

pub async fn back_to_main(bot: Bot, chat_id:ChatId, message_id:MessageId) {
    let go_to_ref_button = InlineKeyboardButton::new(
        "Записаться", 
        teloxide::types::InlineKeyboardButtonKind::CallbackData("get_referrals".to_string())
    );
    let markup = InlineKeyboardMarkup::new([[go_to_ref_button]]);
    bot.edit_message_reply_markup(chat_id, message_id).reply_markup(markup).await.unwrap();
}

pub async fn get_doctors(bot: Bot, user: Model, chat_id:ChatId, message_id:MessageId, referral_id: &u64) {
    let docs_result = get_doctors_obj(&user, referral_id).await;

    match docs_result {
        Ok(doctors) => {
            let away_key=InlineKeyboardButton::new("Назад", teloxide::types::InlineKeyboardButtonKind::CallbackData("get_referrals".to_string()));

            match doctors.result {
                doctors::ResultType::DocArray(doctors) => {
                    let mut doc_vec = vec![];
                    for doctor in doctors {
                        let doc_button = InlineKeyboardButton::new(
                            &format!("{} {} {}", doctor.main_doctor.first_name, doctor.main_doctor.second_name, doctor.main_doctor.last_name), 
                            teloxide::types::InlineKeyboardButtonKind::CallbackData("get_shedule".to_string()));
                        
                        doc_vec.push([doc_button]);
                    }

                    doc_vec.push([away_key]);
                    let markup = InlineKeyboardMarkup::new(doc_vec);
                    bot.edit_message_reply_markup(chat_id, message_id).reply_markup(markup).await.unwrap();
                },
                doctors::ResultType::LdpArray(ldps) => {
                    let mut ldp_vec = vec![];
                    for ldp in ldps {
                        let ldp_button = InlineKeyboardButton::new(
                            &format!("{}", ldp.name), 
                            teloxide::types::InlineKeyboardButtonKind::CallbackData("get_shedule".to_string()));
                        
                        ldp_vec.push([ldp_button]);
                    }

                    ldp_vec.push([away_key]);
                    let markup = InlineKeyboardMarkup::new(ldp_vec);
                    bot.edit_message_reply_markup(chat_id, message_id).reply_markup(markup).await.unwrap();
                },
                doctors::ResultType::EmptyObject(_) => {
                    let no_doc = InlineKeyboardButton::new("Нет врачей по данному направлению.", teloxide::types::InlineKeyboardButtonKind::CallbackData("_".to_string()));
                    let markup = InlineKeyboardMarkup::new([[no_doc], [away_key]]);
                    bot.edit_message_reply_markup(chat_id, message_id).reply_markup(markup).await.unwrap();
                }
            }
        },
        Err(_) => {
            bot.send_message(chat_id, "Не удалось получить список врачей").await.unwrap();
        }
    }
}