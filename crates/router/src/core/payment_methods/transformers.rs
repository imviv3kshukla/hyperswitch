use error_stack::ResultExt;
use serde::{Deserialize, Serialize};

use crate::{
    configs::settings::Locker,
    core::errors::{self, CustomResult},
    headers,
    pii::{self, prelude::*, Secret},
    services::api as services,
    types::{api, storage},
    utils::{self, OptionExt},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCardRequest<'a> {
    pub card_number: Secret<String, pii::CardNumber>,
    pub customer_id: &'a str,
    pub card_exp_month: Secret<String>,
    pub card_exp_year: Secret<String>,
    pub merchant_id: &'a str,
    pub email_address: Option<Secret<String, pii::Email>>,
    pub name_on_card: Option<Secret<String>>,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCard<'a> {
    merchant_id: &'a str,
    card_id: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCardResponse {
    pub card_id: String,
    pub external_id: String,
    pub card_fingerprint: Secret<String>,
    pub card_global_fingerprint: Secret<String>,
    #[serde(rename = "merchant_id")]
    pub merchant_id: Option<String>,
    pub card_number: Option<Secret<String, pii::CardNumber>>,
    pub card_exp_year: Option<Secret<String>>,
    pub card_exp_month: Option<Secret<String>>,
    pub name_on_card: Option<Secret<String>>,
    pub nickname: Option<String>,
    pub customer_id: Option<String>,
    pub duplicate: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetCardResponse {
    pub card: AddCardResponse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCardResponse {
    pub card_id: String,
    pub external_id: String,
    pub card_isin: Option<Secret<String>>,
    pub status: String,
}

pub fn mk_add_card_request(
    locker: &Locker,
    card: &api::CardDetail,
    customer_id: &str,
    _req: &api::CreatePaymentMethod,
) -> CustomResult<services::Request, errors::CardVaultError> {
    let add_card_req = AddCardRequest {
        card_number: card.card_number.clone(),
        customer_id,
        card_exp_month: card.card_exp_month.clone(),
        card_exp_year: card.card_exp_year.clone(),
        merchant_id: "m0010", //FIXME: Need mapping for application mid to lockeId
        email_address: Some("dummy@gmail.com".to_string().into()), //FIXME: If these are mandatory need to have customer object
        name_on_card: Some("juspay".to_string().into()),           //FIXME
        nickname: Some("orca".to_string()),                        //FIXME
    };
    let body = utils::Encode::<AddCardRequest<'_>>::encode(&add_card_req)
        .change_context(errors::CardVaultError::RequestEncodingFailed)?;
    let mut url = locker.host.to_owned();
    url.push_str("/card/addCard");
    let mut request = services::Request::new(services::Method::Post, &url);
    request.add_header(headers::CONTENT_TYPE, "application/x-www-form-urlencoded");
    request.set_body(body);
    Ok(request)
}

pub fn mk_add_card_response(
    card: api::CardDetail,
    response: AddCardResponse,
    req: api::CreatePaymentMethod,
) -> api::PaymentMethodResponse {
    let card = api::CardDetailFromLocker {
        scheme: None,
        last4_digits: Some(card.card_number.peek().to_owned().split_off(12)),
        issuer_country: None, // TODO bin mapping
        card_number: None,
        expiry_month: Some(card.card_exp_month),
        expiry_year: Some(card.card_exp_year),
        card_token: Some(response.external_id.into()), // TODO ?
        card_fingerprint: Some(response.card_fingerprint),
        card_holder_name: None,
    };
    api::PaymentMethodResponse {
        payment_method_id: response.card_id,
        payment_method: req.payment_method,
        payment_method_type: req.payment_method_type,
        payment_method_issuer: req.payment_method_issuer,
        card: Some(card),
        metadata: req.metadata,
        created: Some(crate::utils::date_time::now()),
        payment_method_issuer_code: req.payment_method_issuer_code,
        recurring_enabled: false,                                      //TODO
        installment_payment_enabled: false,                            //TODO
        payment_experience: Some(vec!["redirect_to_url".to_string()]), //TODO,
    }
}

pub fn mk_get_card_request<'a>(
    locker: &Locker,
    _mid: &'a str,
    card_id: &'a str,
) -> CustomResult<services::Request, errors::CardVaultError> {
    let get_card_req = GetCard {
        merchant_id: "m0010", //FIXME: need to assign locker id to every merchant
        card_id,
    };

    let body = utils::Encode::<GetCard<'_>>::encode(&get_card_req)
        .change_context(errors::CardVaultError::RequestEncodingFailed)?;
    let mut url = locker.host.to_owned();
    url.push_str("/card/getCard");
    let mut request = services::Request::new(services::Method::Post, &url);
    request.add_header(headers::CONTENT_TYPE, "application/x-www-form-urlencoded");
    request.set_body(body);
    Ok(request)
}

pub fn mk_delete_card_request<'a>(
    locker: &Locker,
    merchant_id: &'a str,
    card_id: &'a str,
) -> CustomResult<services::Request, errors::CardVaultError> {
    let delete_card_req = GetCard {
        merchant_id,
        card_id,
    };
    let body = utils::Encode::<GetCard<'_>>::encode(&delete_card_req)
        .change_context(errors::CardVaultError::RequestEncodingFailed)?;
    let mut url = locker.host.to_owned();
    url.push_str("/card/deleteCard");
    let mut request = services::Request::new(services::Method::Post, &url);
    request.add_header(headers::X_ROUTER, "test");
    request.add_header(headers::CONTENT_TYPE, "application/x-www-form-urlencoded");
    //request.add_content_type(Content::FORMURLENCODED);
    request.set_body(body);
    Ok(request)
}

pub fn get_card_detail(
    pm: &storage::PaymentMethod,
    response: AddCardResponse,
) -> CustomResult<api::CardDetailFromLocker, errors::CardVaultError> {
    let card_number = response
        .card_number
        .get_required_value("card_number")
        .change_context(errors::CardVaultError::FetchCardFailed)?;
    let card_detail = api::CardDetailFromLocker {
        scheme: pm.scheme.clone(),
        issuer_country: pm.issuer_country.clone(),
        last4_digits: None, //.split_off(12)), //TODO: we need card number as well
        card_number: Some(card_number),
        expiry_month: response.card_exp_month,
        expiry_year: response.card_exp_year,
        card_token: Some(response.external_id.into()), //TODO ?
        card_fingerprint: Some(response.card_fingerprint),
        card_holder_name: None,
    };
    Ok(card_detail)
}