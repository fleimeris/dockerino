use std::{borrow::Borrow, str};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use hyper::{client::HttpConnector, body, Client, Method, header, Body, Request, StatusCode, Response};
use hyperlocal::{UnixConnector, Uri as DomainUri};
use serde_derive::{Serialize, Deserialize};

use crate::images;
use crate::images::Images;
use crate::utils::DockerError;

pub struct Docker
{
    client: Client<UnixConnector>,
    sock_path: String
}

impl Docker
{
    pub fn new(socket_path: String) -> Docker
    {
        Docker
        {
            client: Client::builder()
                .pool_max_idle_per_host(0)
                .build(UnixConnector),
            sock_path: socket_path
        }
    }

    pub fn build_request(&self, method : Method, endpoint: &str) -> Result<Request<Body>, Box<dyn std::error::Error>>
    {
        let uri = DomainUri::new(&self.sock_path, endpoint);

        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::HOST, "")
            .body(Body::empty())?;

        Ok(request)
    }

    pub async fn parse_response_body(&self, response: Response<Body>) -> Result<String, Box<dyn Error>>
    {
        let bytes = hyper::body::to_bytes(response).await?;

        let json = str::from_utf8(bytes.as_ref())?;

        Ok(json.parse()?)
    }

    pub async fn request(&self, method : Method, endpoint: &str) -> Result<String, Box<dyn Error>>
    {
        let request = self.build_request(method, endpoint)?;

        let response = self.borrow().client.request(request).await?;

        match response.status()
        {
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::CONFLICT | StatusCode::INTERNAL_SERVER_ERROR
                =>
            {
                let status_code = response.status();
                let body = self.parse_response_body(response).await?;

                let dockerError: DockerError = serde_json::from_str(body.as_str())?;

                //TODO: custom error with response code

                return Err(format!("Response was {:?}. Message:\n{:?}", status_code, dockerError.message))?
            },
            _ => ()
        }

        let json_body = self.parse_response_body(response).await?;

        Ok(json_body.parse()?)
    }

    pub fn images(&self) -> Images
    {
        images::Images::new(self)
    }
}