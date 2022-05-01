use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use hyper::{Body, Method};
use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use base64::{encode, decode};
use urlencoding;
use crate::docker::{AuthHeader, Docker};

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ImageHistory
{
    id: String,
    created: i128,
    created_by: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    tags: Option<Vec<String>>,
    size: i128,
    comment: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListImagesFilter
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dangling: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>
}

impl ListImagesFilter {
    pub fn url_encoded(self) -> Result<String, Box<dyn Error>>
    {
        let mut params_list = HashMap::new();

        if let Some(before) = self.before
        {
            params_list.insert("before", vec![before]);
        }

        if let Some(dangling) = self.dangling
        {
            params_list.insert("dangling", vec![dangling.to_string()]);
        }

        if let Some(label) = self.label
        {
            params_list.insert("label", vec![label]);
        }

        if let Some(reference) = self.reference
        {
            params_list.insert("reference", vec![reference]);
        }

        if let Some(since) = self.since
        {
            params_list.insert("since", vec![since]);
        }

        let json = serde_json::to_string(&params_list)?;
        let url_encoded = urlencoding::encode(json.as_str());

        Ok(url_encoded.to_string())
    }
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

    pub async fn get_images_all(&self, filter: Option<ListImagesFilter>) -> Result<Vec<Image>, Box<dyn Error>>
    {
        let mut endpoint = format!("/images/json");

        if let Some(filter) = filter
        {
            endpoint.push_str(format!("?filters={}", filter.url_encoded()?).as_str())
        }

        let response = self
            .docker
            .borrow()
            .request(Method::GET, endpoint.as_str(), None, None).await?;

        let response_body = &self.docker.borrow()
            .parse_response_body(response).await?;

        let images: Vec<Image> = serde_json::from_str(response_body)?;

        Ok(images)
    }

    pub async fn get_image(&self, image_name: &str) -> Result<ImageDetails, Box<dyn Error>>
    {
        let endpoint = format!("/images/{}/json", image_name);

        let response = self
            .docker
            .borrow()
            .request(Method::GET, endpoint.as_str(), None, None).await?;

        let response_body = &self.docker.borrow()
            .parse_response_body(response).await?;

        let image: ImageDetails = serde_json::from_str(response_body)?;

        Ok(image)
    }

    pub async fn get_image_history(&self, image_name: &str) -> Result<Vec<ImageHistory>, Box<dyn Error>>
    {
        let endpoint = format!("/images/{}/history", image_name);

        let response = self
            .docker
            .borrow()
            .request(Method::GET, endpoint.as_str(), None, None).await?;

        let response_body = &self.docker.borrow()
            .parse_response_body(response).await?;

        let result: Vec<ImageHistory> = serde_json::from_str(response_body)?;

        Ok(result)
    }

    //TODO: add return method, which returns about deleted image
    pub async fn delete_image(&self, image_name: &str, forced: bool, no_prune: bool)
        -> Result<Vec<ImageDeletionInfo>, Box<dyn Error>>
    {
        let endpoint = format!("/images/{}?force={}&noprune={}", image_name, forced, no_prune);

        let response = self.docker.borrow()
            .request(Method::DELETE, endpoint.as_str(), None, None).await?;

        let response_body = &self.docker.borrow()
            .parse_response_body(response).await?;

        let result: Vec<ImageDeletionInfo> = serde_json::from_str(response_body)?;

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
            .request(Method::POST, endpoint.as_str(), None, None).await?;

        Ok(())
    }

    pub async fn export_image(&self, image_name: &str, file_path: &str) -> Result<(), Box<dyn Error>>
    {
        let endpoint = format!("/images/{}/get", image_name);

        let response = self.docker.borrow()
            .request(Method::GET, endpoint.as_str(), None, None).await?;

        let bytes = hyper::body::to_bytes(response).await?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;

        file.write_all(bytes.borrow());

        drop(file);

        Ok(())
    }

    pub async fn import_image(&self, image_path: &str) -> Result<(), Box<dyn Error>>
    {
        let endpoint = "/images/load";

        let mut file_handle = File::open(image_path)?;
        let mut buffer = Vec::default();

        file_handle.read_to_end(&mut buffer)?;

        let body = Body::from(buffer);

        self.docker.borrow().request(Method::POST, endpoint, Some(body), None).await?;

        drop(file_handle);

        Ok(())
    }

    pub async fn push_image(&self, image_name: &str, server_address: &str, tag: Option<&str>) -> Result<String, Box<dyn Error>>
    {
        let mut endpoint = format!("/images/{}/push", image_name);

        if let Some(some_tag) = tag
        {
            endpoint.push_str(format!("?tag={}", some_tag).as_str());
        }

        let auth_header = AuthHeader
        {
            username: "".to_string(),
            password: "".to_string(),
            email: "".to_string(),
            serveraddress: server_address.to_string()
        };

        let auth_header_json = serde_json::to_string(&auth_header)?;
        let auth_header_base64 = base64::encode(&auth_header_json);

        let mut headers: HashMap<&str, &str> = HashMap::new();
        headers.insert("X-Registry-Auth", auth_header_base64.as_str());

        let result = self.docker.borrow()
            .request(Method::POST, endpoint.as_str(), None, Some(headers)).await?;

        let log = self.docker.borrow()
            .parse_response_body(result).await?;

        Ok(log)
    }
}