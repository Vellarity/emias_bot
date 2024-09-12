use std::collections::HashMap;

use serde::{Deserialize, Serialize, };

#[derive(Debug, Serialize)]
pub struct ReferralsInfoRequest<T> {
    id: Option<String>,
    jsonrpc: String,
    method: String,
    params: T
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct ReferralsInfoParamsRequest {
    omsNumber: String,
    birthDate: String
}

impl ReferralsInfoRequest<ReferralsInfoParamsRequest> {
    pub fn new(
        id:Option<String>, 
        oms_number: String,
        birth_date:String
    ) -> Self {
        Self{
            id: id,
            jsonrpc: "2.0".to_string(),
            method: "getReferralsInfo".to_string(),
            params: ReferralsInfoParamsRequest {
                omsNumber:oms_number,
                birthDate:birth_date
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BasicResponse<T> {
    headers: HashMap<String, String>,
    data: T
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ReferralsInfoResponse {
    pub result: Vec<ReferralInfo>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ReferralInfo {
    pub id: u64,
    pub start_time: String,
    pub end_time: String,
    pub lpu_id:u64,
    pub lpu_name:String,
    pub to_ldp:Option<ToLdp>,
    pub to_doctor:Option<ToDoctor>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ToLdp {
    pub ldp_type_id:u64,
    pub ldp_type_name:String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ToDoctor {
    pub speciality_id: u32,
    pub speciality_name: String,
    pub reception_type_id:u32
}