use serde::{Deserialize, Serialize};
use router_env::logger;
use crate::{
    // connector::utils::AccessTokenRequestInfo,
    // consts,
    core::errors,
    pii::PeekInterface,
    types::{self, api, storage::enums},
    // utils::OptionExt,
};

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct BamboraCard {
    name: String,
    number: String,
    expiry_month: String,
    expiry_year: String,
    cvd: String
    // complete: bool
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct BamboraPaymentsRequest {
    amount: i64,
    payment_method: String,
    card: BamboraCard,
}


impl TryFrom<&types::PaymentsAuthorizeRouterData> for BamboraPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_item: &types::PaymentsAuthorizeRouterData) -> Result<Self, Self::Error> {
        match _item.request.payment_method_data {
            api::PaymentMethod::Card (ref req_card) => {
                let bambora_card =  BamboraCard {
                    name: req_card.card_holder_name.peek().clone(),
                    number: req_card.card_number.peek().clone(),
                    expiry_month: req_card.card_exp_month.peek().clone(),
                    expiry_year: req_card.card_exp_year.peek().clone(),
                    cvd: req_card.card_cvc.peek().clone()
                    // complete: false
                };
                logger::debug!(log_log=?_item);
                Ok(Self {
                    amount: _item.request.amount,
                    payment_method: "card".to_string(),
                    card: bambora_card
                })
            }
            _ => Err(errors::ConnectorError::NotImplemented("Payment methods".to_string()).into()),
        }
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct BamboraAuthType {
    pub(super) api_key: String,
}

impl TryFrom<&types::ConnectorAuthType> for BamboraAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        if let types::ConnectorAuthType::HeaderKey { api_key } = _auth_type {
            Ok(Self {
                api_key: api_key.to_string(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType)?
        }
    }
}
// PaymentsResponse
// //TODO: Append the remaining status flags
// #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
// #[serde(rename_all = "lowercase")]
// pub enum BamboraPaymentStatus {
//     Succeeded,
//     Failed,
//     #[default]
//     Processing,
// }

// impl From<BamboraPaymentStatus> for enums::AttemptStatus {
//     fn from(item: BamboraPaymentStatus) -> Self {
//         match item {
//             BamboraPaymentStatus::Succeeded => Self::Charged,
//             BamboraPaymentStatus::Failed => Self::Failure,
//             BamboraPaymentStatus::Processing => Self::Authorizing,
//         }
//     }
// }

// //TODO: Fill the struct with respective fields
// #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct BamboraPaymentsResponse {
//     status: BamboraPaymentStatus,
//     id: String,
// }

impl<F, T>
    TryFrom<types::ResponseRouterData<F, BamboraPaymentsResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        item: types::ResponseRouterData<F, BamboraPaymentsResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        logger::debug!(payment_sync_response=?item.response);
        let pg_response = item.response;
        Ok(Self {
            status: match pg_response.approved.as_str() {
                "0" => enums::AttemptStatus::Failure,
                "1" => enums::AttemptStatus::Pending,
                &_ => todo!(),
                // _ => todo!()
            },
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(pg_response.id),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum BamboraPaymentsResponse {
//     SuccessPaymentResponse(BamboraPaymentsSuccessResponse),
//     ErrorRespType(BamboraPaymentsErrorResponse)
// }
//TODO How to handle error response
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BamboraPaymentsErrorResponse {
    code: i32,
    category: i32,
    message: String,
    reference: String,
    details: Vec<ErrorDetail>,
    validation: Option<CardValidation>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorDetail {
    field: String,
    message: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardValidation {
    id: String,
    approved: i32,
    message_id: i32,
    message: String,
    auth_code: String,
    trans_date: String,
    order_number: String,
    type_: String,
    amount: f64,
    cvd_id: i32,
}


#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BamboraPaymentsResponse {
    id : String, 
    authorizing_merchant_id : i32, 
    approved : String, 
    message_id : String, 
    message : String, 
    auth_code : String,
    created : String,
    amount : f32,
    order_number : String,
    #[serde(rename = "type")]
    payment_type : String,
    comments : Option<String>,
    batch_number : Option<String>,
    total_refunds : Option<f32>,
    total_completions : Option<f32>,
    payment_method : String,
    card : CardData,
    billing : Option<AddressData>,
    shipping : Option<AddressData>,
    custom : CustomData,
    adjusted_by : Option<Vec<AdjustedBy>>,
    links : Vec<Links>,
    risk_score : String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardData {
    name : Option<String>,
    expiry_month : Option<String>,
    expiry_year : Option<String>,
    card_type : String,
    last_four : String,
    card_bin : String,
    avs_result : String,
    cvd_result : String,
    cavv_result: Option<String>,
    address_match: i32,
    postal_result: i32,
    avs: AvsObject 
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvsObject {
    id : String,
    message : String,
    processed : bool
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddressData {
    name : String,
    address_line1 : String,
    address_line2 : String,
    city : String,
    province : String,
    country : String,
    postal_code : String,
    phone_number : String,
    email_address : String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomData {
    ref1 : String, 
    ref2 : String, 
    ref3 : String, 
    ref4 : String, 
    ref5 : String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdjustedBy {
    id : i32, 
    #[serde(rename = "type")]
    adjusted_by_type : String, 
    approval : i32, 
    message : String, 
    amount : f32, 
    created : String, 
    url : String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Links {
    rel : String, 
    href : String, 
    method: String 
}







//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct BamboraRefundRequest {}

impl<F> TryFrom<&types::RefundsRouterData<F>> for BamboraRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self, Self::Error> {
        todo!()
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        
        todo!()
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>>
    for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::RSync, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct BamboraErrorResponse {}