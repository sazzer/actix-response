use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use actix_http::http::{
    header::{self, Header, IntoHeaderValue},
    HeaderMap, HeaderName, HeaderValue, StatusCode,
};
use serde::Serialize;

use super::{HalResponse, Link, Links};
use crate::response::Respondable;

/// Respondable to represent a HAL resource.
#[derive(Debug)]
pub struct HalRespondable<T>
where
    T: Serialize,
{
    payload:     T,
    status_code: StatusCode,
    headers:     Headers,
    links:       BTreeMap<String, Links>,
}

/// The actual JSON payload of a HAL resource.
#[derive(Debug, Serialize)]
pub struct HalPayload<T>
where
    T: Serialize,
{
    #[serde(rename = "_links")]
    pub links:   BTreeMap<String, Links>,
    #[serde(flatten)]
    pub payload: T,
}

impl<T> HalRespondable<T>
where
    T: Serialize,
{
    /// Create a new HAL Respondable for the provided payload.
    ///
    /// # Parameters
    /// - `payload` - The payload of the response
    pub fn new(payload: T) -> Self {
        let mut headers = Headers::default();
        headers.append(header::CONTENT_TYPE, HeaderValue::from_static("application/hal+json"));

        Self {
            payload,
            status_code: StatusCode::OK,
            headers,
            links: BTreeMap::new(),
        }
    }

    /// Specify the status code of the response.
    ///
    /// # Parameters
    /// - `status_code` - The status code
    pub fn with_status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;

        self
    }

    /// Add a header to the response.
    ///
    /// # Parameters
    /// - `name` - The name of the header
    /// - `value` - The value of the header
    pub fn with_header_value<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<HeaderName>,
        V: IntoHeaderValue,
    {
        self.headers.with_header_value(name, value);

        self
    }

    /// Add a header to the response
    ///
    /// # Parameters
    /// - `header` - The header to add
    pub fn with_header<H>(self, header: H) -> Self
    where
        H: Header + IntoHeaderValue,
    {
        self.with_header_value(H::name(), header)
    }

    /// Add a link to the response.
    ///
    /// # Parameters
    /// - `name` - The name of the link
    /// - `link` - The actual link
    pub fn with_link<S, L>(mut self, name: S, link: L) -> Self
    where
        S: Into<String>,
        L: Into<Link>,
    {
        let name = name.into();
        let links = match self.links.remove(&name) {
            None => Links::Single(link.into()),
            Some(links) => links.push(link.into()),
        };
        self.links.insert(name, links);

        self
    }
}

impl<T> Respondable for HalRespondable<T>
where
    T: Serialize,
{
    type Body = HalPayload<T>;

    fn body(self) -> Self::Body {
        HalPayload {
            payload: self.payload,
            links:   self.links,
        }
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn headers(&self) -> HeaderMap {
        self.headers.clone()
    }
}

/// Trait that model resources can implement to convert it into a HAL response.
///
/// # Types
/// - `T` - The HAL payload type.
pub trait IntoHal<T>: Sized
where
    T: Serialize,
{
    /// Convert this object into a `HalResponse` object.
    fn into_hal(self) -> HalResponse<T> {
        let mut headers = Headers::default();
        self.headers(&mut headers);

        let status_code = self.status_code();
        let links = self.links();
        let payload = self.payload();

        let mut respondable = HalRespondable::new(payload).with_status_code(status_code);

        for (name, value) in headers.iter() {
            respondable = respondable.with_header_value(name, value);
        }

        for (name, link) in links {
            respondable = respondable.with_link(name, link);
        }

        respondable.into()
    }

    /// Determine the status code to use for the response.
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }

    /// Determine any headers to use for the response.
    fn headers(&self, _headers: &mut Headers) {}

    /// Generate the links to include in the response.
    fn links(&self) -> Vec<(String, Link)> {
        vec![]
    }

    /// Generate the payload to respond with.
    fn payload(self) -> T;
}

/// Wrapper around the headers to make it easier to work with.
#[derive(Debug, Default)]
pub struct Headers(HeaderMap);

impl Headers {
    /// Add a header to the response.
    ///
    /// # Parameters
    /// - `name` - The name of the header
    /// - `value` - The value of the header
    pub fn with_header_value<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: Into<HeaderName>,
        V: IntoHeaderValue,
    {
        let value: HeaderValue = value.try_into_value().ok().unwrap();

        self.0.append(name.into(), value);

        self
    }

    /// Add a header to the response
    ///
    /// # Parameters
    /// - `header` - The header to add
    pub fn with_header<H>(&mut self, header: H) -> &mut Self
    where
        H: Header + IntoHeaderValue,
    {
        self.with_header_value(H::name(), header)
    }
}

impl Deref for Headers {
    type Target = HeaderMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
