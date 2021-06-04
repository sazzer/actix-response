mod links;
mod response;

pub use links::*;
pub use response::*;

use super::response::Response;

/// Type representing a HAL Response for the given payload type.
pub type HalResponse<T> = Response<HalRespondable<T>>;
