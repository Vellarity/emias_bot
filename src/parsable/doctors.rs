use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use super::basic::BasicRequest;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct DoctorsInfoParamsRequest {
    omsNumber: String,
    birthDate:String,
    referralId: u64
}

impl BasicRequest<DoctorsInfoParamsRequest> {
    pub fn new(
        id:Option<String>, 
        oms_number: String,
        birth_date:String,
        referral_id: u64,
    ) -> Self {
        Self {
            id,
            jsonrpc: "2.0".to_string(),
            method: "getDoctorsInfo".to_string(),
            params: DoctorsInfoParamsRequest {
                omsNumber:oms_number,
                birthDate:birth_date,
                referralId: referral_id
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DoctorsInfoParamsResponse {
    pub result: ResultType//ResultType
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultType {
    LdpArray(Vec<LdpInfo>),
    DocArray(Vec<DoctorInfo>),
    EmptyObject(HashMap<String, String>)
}

#[derive(Debug, Deserialize)]
pub struct Empty;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LdpInfo {
    pub id: u64,
    pub lpu_id: u64,
    pub name: String,
    pub ldp_type: Vec<LdpType>,
    pub complex_resource: Vec<ComplexResource>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LdpType {
    pub code: String,
    pub name: String
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DoctorInfo {
    pub id: u64,
    pub lpu_id: u64,
    pub name: String,
    pub ar_speciality_id: u32,
    pub ar_speciality_name: String,
    pub main_doctor: MainDoctor,
    pub complex_resource: Vec<ComplexResource>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MainDoctor {
    pub speciality_name:String,
    pub speciality_id: u32,
    pub first_name: String,
    pub last_name: String,
    pub second_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ComplexResource {
    pub id: u64,
    pub name: String
}