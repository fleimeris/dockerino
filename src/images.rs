use std::borrow::Borrow;
use std::error::Error;
use hyper::Method;
use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};

use crate::docker::Docker;

#[derive(Serialize, Deserialize, Debug)]
pub struct Image
{
    ParentId: String,
    Created: i128,
    Size: i128,
    SharedSize: i128,
    Containers: i128,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ImageDetails
{
    id: String,
    repo_tags: Vec<String>,
    repo_digests: Vec<String>,
    parent: String,
    comment: String,
    container: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    container_config: Option<ContainerConfig>,
    architecture: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    variant: Option<String>,
    os: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    os_version: Option<String>,
    size: i128,
    virtual_size: i128,
    //TODO: add GraphDriver
    //TODO: add RootFS
    //TODO: add Metadata
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerConfig
{
    hostname: String,
    domainname: String,
    user: String,
    attach_stdin: bool,
    attach_stdout: bool,
    attach_stderr: bool,
    //TODO: add ExposedPorts
    tty: bool,
    open_stdin: bool,
    stdin_once: bool,
    env: Vec<String>,
    cmd: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    health_check: Option<HealthCheck>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    args_escaped: Option<bool>,
    working_dir: String,
    entrypoint: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    network_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    mac_address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    on_build: Option<Vec<String>>,

    //TODO: add Labels

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    stop_signal: Option<String>,

}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct HealthCheck
{
    test: Vec<String>,
    interval: i32,
    timeout: i32,
    retries: i32,
    start_period: i32
}

pub struct Images<'a>
{
    docker: &'a Docker
}

impl Images<'_>
{
    pub fn new(docker: &'_ Docker) -> Images
    {
        Images
        {
            docker
        }
    }

    pub async fn get_images_all(&self) -> Result<Vec<Image>, Box<dyn Error>>
    {
        let response = self
            .docker
            .borrow()
            .request(Method::GET, "/images/json").await?;

        let images: Vec<Image> = serde_json::from_str(response.as_ref())?;

        Ok(images)
    }

    pub async fn get_image(&self, image_name: &str) -> Result<ImageDetails, Box<dyn Error>>
    {
        let endpoint = format!("/images/{}/json", image_name);
        let response = self
            .docker
            .borrow()
            .request(Method::GET, endpoint.as_str()).await?;

        let image: ImageDetails = serde_json::from_str(response.as_str())?;

        Ok(image)
    }

    //pub async fn delete_image(&self, image_name: &str)
}