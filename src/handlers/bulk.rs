use std::net::IpAddr;
use serde::{Serialize};
use rocket::serde::json::{Json, Value as JsonValue};
use rocket::http::Status;
use rocket::State;
use std::net::SocketAddr;
use crate::models;
use crate::{fairings::rate_limit::RateLimitState, fairings::database::Pool, responses::*};
use crate::config::Config;

pub mod utils {
  use super::*;

  pub fn check_rate_limit<S: Serialize, E: Serialize>(ip: IpAddr, rls: &RateLimitState, request_size: i64, response_builder: &mut ResponseBuilder<S, E>) -> Result<(), ()> {
    match rls.lock() {
      Ok(rls) => {
        match rls.get(&ip) {
          Some(entry) => {
            let allowed_requests = entry.0;

            if entry.0 <= (allowed_requests + request_size) {
              Ok(())
            } else {
              response_builder.error(
                Status::TooManyRequests,
                ResponseErrorType::BulkRequestExceedingSizeError,
                format!("Received {} requests, but your limit only allows for {} requests!",
                  request_size,
                  allowed_requests
                )
              );
              Err(())
            }
          },
          None => Ok(())
        }
      },
      Err(_) => {
        response_builder.error(
          Status::InternalServerError,
          ResponseErrorType::UndefinedError,
          String::from("Could not acquire rate limit state!")
        );
        Err(())
      }
    }
  }
}

pub fn add_link(links: Json<Vec<models::db_less::NewLink>>, db: &State<Pool>, config: &State<Config>, rl: &State<RateLimitState>, ip: SocketAddr) -> ResponseBuilder<Vec<models::db_less::NewLinkResult>, JsonValue> {
  let mut response_builder = ResponseBuilder::new();
  let ip = ip.ip();
  let links = links.0;

  match utils::check_rate_limit(ip, rl.inner(), links.len() as i64, &mut response_builder) {
    Ok(_) => {
      let mut i: u32 = 0;
      let mut results: Vec<models::db_less::NewLinkResult> = Vec::new();
      let mut last_error: bool = false;

      for link in links.iter() {
        i += 1;

        let res_builder = crate::handlers::add_link(link, db, config);

        if ResponseBuilder::is_success(&res_builder) {
          let data = res_builder
            .get_data()
            .unwrap();

          results.push(data.clone());
        } else {
          let bulk_request_error = errors::BulkRequestError {
            request_number: i,
            request_error_type: res_builder.get_error_type().unwrap(),
            request_error_message: res_builder.get_error_message().unwrap(),
            request_error_data: res_builder.get_error_data()
          };

          response_builder.error(
            res_builder.get_status(),
            ResponseErrorType::BulkRequestError,
            if results.len() == 0 { 
              String::from("An error happened inside of bulk request's internal request!") 
            } else {
              String::from("Some requests have been successfully fulfilled but an error happened inside one of them!")
            }
          )
            .data(results.clone())
            .error_data(bulk_request_error.to_json().unwrap());
          last_error = true;
          break;
        }
      }

      if !last_error {
        response_builder.success(
          Status::Ok
        )
          .data(results);
      }
    },
    Err(_) => {} 
  }

  response_builder
}