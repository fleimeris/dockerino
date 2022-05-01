use std::{borrow::Borrow, str};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use hyper::{client::HttpConnector, body, Client, Method, header, Body, Request, StatusCode, Response};
use hyperlocal::{UnixConnector, Uri as DomainUri};
use serde_derive::{Serialize, Deserialize};

use crate::images;
use crate::images::Images;
use crate::utils::DockerError;

#[derive(Serialize, Debug)]
pub struct AuthHeader
{
    pub username: String,
    pub password: String,
    pub email: String,
    pub serveraddress: String
}

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

    pub fn build_request(&self, method: Method, endpoint: &str, body: Option<Body>,
                     header: Option<HashMap<&str, &str>>)
        -> Result<Request<Body>, Box<dyn Error>>
    {
        let url = DomainUri::new(&self.sock_path, endpoint);

        let request = Request::builder()
            .method(method)
            .uri(url);

        let mut request = request.header(header::HOST, "");

        if let Some(header) = header
        {
            for (key, value) in header
            {
                request = request.header(key, value);
            }
        }

        if let Some(body) = body
        {
            return Ok(request.body(body)?)
        }

        Ok(request.body(Body::empty())?)
    }

    pub async fn parse_response_body(&self, response: Response<Body>) -> Result<String, Box<dyn Error>>
    {
        let bytes = hyper::body::to_bytes(response).await?;

        let json = str::from_utf8(bytes.as_ref())?;

        Ok(json.parse()?)
    }

    pub async fn request(&self, method : Method, endpoint: &str, body: Option<Body>,
                         header: Option<HashMap<&str, &str>>)
        -> Result<Response<Body>, Box<dyn Error>>
    {
        let request = self.build_request(method, endpoint, body, header)?;

        let response = self.borrow().client.request(request).await?;

        match response.status()
        {
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::CONFLICT | StatusCode::INTERNAL_SERVER_ERROR
                =>
            {
                let status_code = response.status();

                let docker_error = self.parse_error(response).await?;

                //TODO: custom error with response code

                return Err(format!("Response was {:?}. Message:\n{:?}", status_code, docker_error.message))?
            },
            _ => ()
        }

        Ok(response)
    }

    pub async fn parse_error(&self, response: Response<Body>) -> Result<DockerError, Box<dyn Error>>
    {
        let body = self.parse_response_body(response).await?;

        let dockerError: DockerError = serde_json::from_str(body.as_str())?;

        Ok(dockerError)
    }

    pub fn images(&self) -> Images
    {
        images::Images::new(self)
    }
}