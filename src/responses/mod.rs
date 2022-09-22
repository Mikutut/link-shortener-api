//TODO: Replace old response system with new one (in new submodule)

use serde::{Serialize};
use rocket::serde::json::Json;
use rocket::http::Status;

pub mod new;
pub mod errors;