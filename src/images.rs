use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use hyper::{Body, Method};
use serde_derive::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use base64;
use urlencoding;
use crate::docker::{AuthHeader, Docker};
use flate2::{Compression, write::GzEncoder};
use tar::Builder;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Image
{
    parent_id: String,
    created: i128,
    size: i128,
    shared_size: i128,
    containers: i128,
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

pub struct ListImagesFilter
{
    params: HashMap<String, Vec<String>>
}

impl ListImagesFilter {

    pub fn new() -> Self
    {
        ListImagesFilter
        {
            params: HashMap::new()
        }
    }

    pub fn before(&mut self, before: String) -> &mut Self
    {
        self.params.insert("before".parse().unwrap(), vec![before]);
        self
    }

    pub fn dangling(&mut self, dangling: bool) -> &mut Self
    {
        self.params.insert("dangling".parse().unwrap(), vec![dangling.to_string()]);
        self
    }

    pub fn label(&mut self, label: String) -> &mut Self
    {
        self.params.insert("label".parse().unwrap(), vec![label]);
        self
    }

    pub fn reference(&mut self, reference: String) -> &mut Self
    {
        self.params.insert("reference".parse().unwrap(), vec![reference]);
        self
    }

    pub fn since(&mut self, since: String) -> &mut Self
    {
        self.params.insert("since".parse().unwrap(), vec![since]);
        self
    }

    pub fn build(&self) -> ListImagesFilter
    {
        ListImagesFilter
        {
            params: self.params.clone()
        }
    }

    pub fn url_encoded(&self) -> Result<String, Box<dyn Error>>
    {
        let json = serde_json::to_string(&self.params)?;
        let url_encoded = urlencoding::encode(json.as_str());

        Ok(url_encoded.to_string())
    }
}

pub struct DockerBuildParams
{
    params: HashMap<&'static str, String>
}

impl DockerBuildParams {
    pub fn url_encoded(self) -> Result<String, Box<dyn Error>>
    {
        let mut result = String::new();

        let mut first = true;

        for (key, value) in self.params
        {
            if first
            {
                result.push_str(format!("{}={}", key, value).as_str());
            }
            else
            {
                result.push_str(format!("&{}={}", key, value).as_str());
            }

            first = false;
        }

        Ok(result)
    }
}

pub struct DockerBuildParamsBuilder
{
    params: HashMap<&'static str, String>
}

impl DockerBuildParamsBuilder {

    pub fn new() -> Self
    {
        DockerBuildParamsBuilder
        {
            params: HashMap::new()
        }
    }

    pub fn dockerfile(&mut self, dockerfile: String) -> &mut Self
    {
        self.params.insert("dockerfile", dockerfile);
        self
    }

    pub fn tag(&mut self, tag: String) -> &mut Self
    {
        self.params.insert("t", tag);
        self
    }

    pub fn extrahosts(&mut self, extra_hosts: String) -> &mut Self
    {
        self.params.insert("extrahosts", extra_hosts);
        self
    }

    pub fn remote(&mut self, remote: String) -> &mut Self
    {
        self.params.insert("remote", remote);
        self
    }

    pub fn verbose(&mut self, verbose_enabled: bool) -> &mut Self
    {
        self.params.insert("q", verbose_enabled.to_string());
        self
    }

    pub fn no_cache(&mut self, no_cache_enabled: bool) -> &mut Self
    {
        self.params.insert("nocache", no_cache_enabled.to_string());
        self
    }

    pub fn cache_from(&mut self, image: String) -> &mut Self
    {
        self.params.insert("cachefrom", image);
        self
    }

    pub fn cache_from_multiple(&mut self, images: Vec<String>) -> &mut Self
    {
        let json_images = serde_json::to_string(&images).unwrap();
        self.params.insert("cachefrom", json_images);
        self
    }

    pub fn pull(&mut self, pull_enabled: bool) -> &mut Self
    {
        self.params.insert("pull", pull_enabled.to_string());
        self
    }

    pub fn remove_after_build(&mut self, remove_enabled: bool) -> &mut Self
    {
        self.params.insert("rm", remove_enabled.to_string());
        self
    }

    pub fn force_remove_after_build(&mut self, force_enabled: bool) -> &mut Self
    {
        self.params.insert("forcerm", force_enabled.to_string());
        self
    }

    pub fn memory_limit(&mut self, size_limit: i32) -> &mut Self
    {
        self.params.insert("memory", size_limit.to_string());
        self
    }

    pub fn swap_size(&mut self, swap_size: i32) -> &mut Self
    {
        self.params.insert("memswap", swap_size.to_string());
        self
    }

    pub fn cpu_shares(&mut self, weight: i32) -> &mut Self
    {
        self.params.insert("cpushares", weight.to_string());
        self
    }

    pub fn set_cpus(&mut self, cpus: String) -> &mut Self
    {
        self.params.insert("cpusetcpus", cpus);
        self
    }

    pub fn cpu_period(&mut self, period: i32) -> &mut Self
    {
        self.params.insert("cpuperiod", period.to_string());
        self
    }

    pub fn cpu_quota(&mut self, cpu_quota: i32) -> &mut Self
    {
        self.params.insert("cpuquota", cpu_quota.to_string());
        self
    }

    pub fn build_args(&mut self, args: HashMap<&str, String>) -> &mut Self
    {
        let json_args = serde_json::to_string(&args).unwrap();

        self.params.insert("buildargs", json_args);
        self
    }

    pub fn shm_size(&mut self, size: i32) -> &mut Self
    {
        self.params.insert("shmsize", size.to_string());
        self
    }

    pub fn squash(&mut self, squashing_enabled: bool) -> &mut Self
    {
        self.params.insert("squash", squashing_enabled.to_string());
        self
    }

    pub fn labels(&mut self, labels: Vec<String>) -> &mut Self
    {
        let json_labels = serde_json::to_string(&labels).unwrap();

        self.params.insert("labels", json_labels);
        self
    }

    pub fn network_mode(&mut self, mode: String) -> &mut Self
    {
        self.params.insert("networkmode", mode);
        self
    }

    pub fn platform(&mut self, platform: String) -> &mut Self
    {
        self.params.insert("platform", platform);
        self
    }

    pub fn target(&mut self, target: String) -> &mut Self
    {
        self.params.insert("target", target);
        self
    }

    pub fn outputs(&mut self, outputs: String) -> &mut Self
    {
        self.params.insert("outputs", outputs);
        self
    }

    pub fn build(&self) -> DockerBuildParams
    {
        DockerBuildParams
        {
            params: self.params.clone()
        }
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

        endpoint = match tag
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

        file.write_all(bytes.borrow())?;

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

    pub async fn build_image(&self, folder_path: &str, build_params: Option<DockerBuildParams>)
        -> Result<String, Box<dyn Error>>
    {
        let mut endpoint = String::from("/build");

        if let Some(build_params) = build_params
        {
            endpoint.push_str(format!("?{}", build_params.url_encoded()?).as_str());
        }

        let mut bytes = Vec::default();

        {
            let mut archive = Builder::new(GzEncoder::new(&mut bytes, Compression::best()));
            archive.append_dir_all("", folder_path)?;
        }

        let body = Body::from(bytes);

        let mut headers: HashMap<&str, &str> = HashMap::new();
        headers.insert("Content-type", "application/x-tar");

        let response = self.docker
            .request(Method::POST, endpoint.as_str(), Some(body),
            Some(headers)).await?;

        let result = self.docker.parse_response_body(response).await?;

        Ok(result)
    }
}