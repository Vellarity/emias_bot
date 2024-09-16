use crate::parsable::basic::BasicRequest;
use crate::parsable::doctors::{self, DoctorsInfoParamsRequest, DoctorsInfoParamsResponse, HasComplexResource};
use crate::parsable::referrals::{ReferralsInfoParamsRequest, ReferralsInfoResponse};

use crate::entities::info::Model;

use chrono::NaiveDate;

pub async fn get_user_referrals(user: &Model) -> Result<String, reqwest::Error> {

    let ref_res = get_referrals_obj(user).await;

    match ref_res {
        Ok(referrals) => {
            let mut message_string = "Ваши направления: \n".to_string();

            for referral in referrals.result {
                message_string += &format!(
                    "[{start} - {end}] {name}\n", 
                    start = NaiveDate::parse_from_str(&referral.start_time, "%Y-%m-%d").unwrap().format("%d.%m.%Y").to_string(),
                    end = NaiveDate::parse_from_str(&referral.end_time, "%Y-%m-%d").unwrap().format("%d.%m.%Y").to_string(),
                    name = if referral.to_doctor.is_some() { referral.to_doctor.unwrap().speciality_name  } else { referral.to_ldp.unwrap().ldp_type_name },
                );

                let doctors_string = get_doctors_with_shedule(user, &referral.id).await;

                match doctors_string {
                    Ok(doc_str) => {
                        message_string += &doc_str;
                    },
                    Err(err) => return Err(err),
                }
            }
            return Ok(message_string)
        },
        Err(e) => return Err(e)
    }
}

pub async fn get_doctors_with_shedule(user:&Model, referral_id:&u64) -> Result<String, reqwest::Error> {

    let doc_res = get_doctors_obj(user, referral_id).await;

    match doc_res {
        Ok(doctors) => {
            let mut doctors_string = String::new();

            if let doctors::ResultType::LdpArray(result) = doctors.result {
                for ldp in result {
                    doctors_string.push_str(&format!("- {}: \n", ldp.name));
                    let free_rooms = collect_free_rooms_data(ldp);
                    doctors_string.push_str(&free_rooms);
                }
            } else if let doctors::ResultType::DocArray(result) = doctors.result {
                for doctor in result {
                    doctors_string.push_str(&format!("- {} {} {}: \n", doctor.main_doctor.first_name, doctor.main_doctor.second_name, doctor.main_doctor.last_name));
                    let free_rooms = collect_free_rooms_data(doctor);
                    doctors_string.push_str(&free_rooms);
                }
            }
            if doctors_string == "" {
                doctors_string += "- Нет врачей по данному направлению.\n";
            }
            doctors_string += "\n";

            return Ok(doctors_string)
        },
        Err(err) => return Err(err)
            //let _ = loop_bot.send_message(ChatId(user.chat_id), format!("Не удалось получить список направлений по причине: `{}`", err.status().unwrap())).await;
    }
}

pub async fn get_referrals_obj(user: &Model) -> Result<ReferralsInfoResponse, reqwest::Error> {
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
        Ok(v) => {
            Ok(v.json::<ReferralsInfoResponse>().await?)
        },
        Err(err) => Err(err)
    }
}

pub async fn get_doctors_obj(user:&Model, referral_id:&u64) -> Result<DoctorsInfoParamsResponse,reqwest::Error> {
    let doc_data = BasicRequest::<DoctorsInfoParamsRequest>::new(
        Some("123".to_owned()),
        user.oms_card.unwrap().to_string(),
        user.date_birth.unwrap().to_string(),
        *referral_id
    );

    let doc_res = reqwest::Client::new()
        .post("https://emias.info/api/emc/appointment-eip/v1/?getDoctorsInfo")
        .json(&doc_data)
        .send()
        .await;

    match doc_res {
        Ok(v) => Ok(v.json::<DoctorsInfoParamsResponse>().await?),
        Err(err) => Err(err)
    }

}

pub fn collect_free_rooms_data<T:HasComplexResource>(resource:T) -> String {
    let complex_res = resource.complex_resource();
    let mut rooms_string = String::new();

    for complex in complex_res {
        match &complex.room {
            Some(v) => {
                rooms_string.push_str(
                    &format!("[{}] \n", v.availability_date.format("%d.%m.%Y").to_string())
                )
            },
            None => {
                continue;
            }
        }
    }

    if rooms_string.len() == 0 {
        "Нет записей.\n".to_string()
    } else {
        rooms_string += "\n";
        rooms_string
    }
}