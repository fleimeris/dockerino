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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ImageDeletionInfo
{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    untagged: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    deleted: Option<String>
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

    //TODO: add return method, which returns about deleted image
    pub async fn delete_image(&self, image_name: &str, forced: bool, no_prune: bool)
        -> Result<Vec<ImageDeletionInfo>, Box<dyn Error>>
    {
        let endpoint = format!("/images/{}?force={}&noprune={}", image_name, forced, no_prune);

        let response = self.docker.borrow()
            .request(Method::DELETE, endpoint.as_str()).await?;

        let result: Vec<ImageDeletionInfo> = serde_json::from_str(response.as_str())?;

        Ok(result)
    }

    pub async fn tag_image(&self, image_name: &str, repo_name: Option<&str>, tag: Option<&str>)
        -> Result<(), Box<dyn Error>>
    {
        let endpoint = format!("/images/{}/tag?repo=!repo_name!&tag=!tag_name!", image_name);

        let mut endpoint = match repo_name
        {
            Some(repo_name) => endpoint.replace("!repo_name!", repo_name),
            None => endpoint.replace("!repo_name!", "")
        };

        let mut endpoint = match tag
        {
            Some(tag) => endpoint.replace("!tag_name!", tag),
            None => endpoint.replace("!tag_name!", "")
        };

        self.docker.borrow()
            .request(Method::POST, endpoint.as_str()).await?;

        Ok(())
    }
}