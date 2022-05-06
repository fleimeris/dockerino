use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use hyper::{Body, Method};
use serde_derive::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use base64;
use crate::docker::{AuthHeader, Docker};
use flate2::{Compression, write::GzEncoder};
use tar::Builder;
use crate::utils::ObjectConverter;

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
            endpoint.push_str(format!("?filters={}", filter.url_encoded(&filter.params)?).as_str())
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
            endpoint.push_str(format!("?{}", build_params.parse_params(&build_params.params)?).as_str());
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

    pub async fn search_images(&self, term: &str, limit: Option<i32>, filters: Option<SearchImagesFilter>)
        -> Result<Vec<ImageSearchResult>, Box<dyn Error>>
    {
        let mut endpoint = format!("/images/search?term={}", term);

        if let Some(limit) = limit
        {
            endpoint.push_str(format!("&limit{}", limit).as_str());
        }

        if let Some(filters) = filters
        {
            let filter_json = filters.url_encoded(&filters.params)?;
            endpoint.push_str(format!("&filters={}", filter_json).as_str());
        }

        let response = self.docker
            .request(Method::GET, endpoint.as_str(), None, None).await?;

        let response_body = self.docker.parse_response_body(response).await?;

        let result: Vec<ImageSearchResult> = serde_json::from_str(response_body.as_str())?;

        Ok(result)
    }
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSearchResult
{
    description: String,
    is_official: bool,
    is_automated: bool,
    name: String,
    star_count: i32
}

pub struct ListImagesFilter
{
    params: HashMap<&'static str, Vec<String>>
}

impl ListImagesFilter {

    pub fn new() -> Self
    {
        ListImagesFilter
        {
            params: HashMap::new()
        }
    }

    pub fn before<T>(&mut self, before: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("before", vec![before.into()]);
        self
    }

    pub fn dangling(&mut self, dangling: bool) -> &mut Self
    {
        self.params.insert("dangling", vec![dangling.to_string()]);
        self
    }

    pub fn label<T>(&mut self, label: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("label", vec![label.into()]);
        self
    }

    pub fn reference<T>(&mut self, reference: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("reference", vec![reference.into()]);
        self
    }

    pub fn since<T>(&mut self, since: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("since", vec![since.into()]);
        self
    }

    pub fn build(&self) -> ListImagesFilter
    {
        ListImagesFilter
        {
            params: self.params.clone()
        }
    }
}

impl ObjectConverter for ListImagesFilter
{}

pub struct DockerBuildParams
{
    params: HashMap<&'static str, String>
}

impl ObjectConverter for DockerBuildParams
{
}

pub struct DockerBuildParamsBuilder
{
    params: HashMap<&'static str, String>
}

impl DockerBuildParamsBuilder
{

    pub fn new() -> Self
    {
        DockerBuildParamsBuilder
        {
            params: HashMap::new()
        }
    }

    pub fn dockerfile<T>(&mut self, dockerfile: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("dockerfile", dockerfile.into());
        self
    }

    pub fn tag<T>(&mut self, tag: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("t", tag.into());
        self
    }

    pub fn extrahosts<T>(&mut self, extra_hosts: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("extrahosts", extra_hosts.into());
        self
    }

    pub fn remote<T>(&mut self, remote: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("remote", remote.into());
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

    pub fn cache_from<T>(&mut self, image: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("cachefrom", image.into());
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

    pub fn set_cpus<T>(&mut self, cpus: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("cpusetcpus", cpus.into());
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

    pub fn network_mode<T>(&mut self, mode: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("networkmode", mode.into());
        self
    }

    pub fn platform<T>(&mut self, platform: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("platform", platform.into());
        self
    }

    pub fn target<T>(&mut self, target: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("target", target.into());
        self
    }

    pub fn outputs<T>(&mut self, outputs: T) -> &mut Self
        where
            T: Into<String>
    {
        self.params.insert("outputs", outputs.into());
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

pub struct SearchImagesFilter
{
    params: HashMap<&'static str, Vec<String>>
}

impl ObjectConverter for SearchImagesFilter {}

pub struct SearchImagesFilterBuilder
{
    params: HashMap<&'static str, Vec<String>>
}

impl SearchImagesFilterBuilder
{
    pub fn new() -> Self
    {
        SearchImagesFilterBuilder
        {
            params: HashMap::new()
        }
    }

    pub fn is_automated(&mut self, automated_enabled: bool) -> &mut Self
    {
        self.params.insert("is-automated", vec![automated_enabled.to_string()]);
        self
    }

    pub fn is_official(&mut self, official_enabled: bool) -> &mut Self
    {
        self.params.insert("is-official", vec![official_enabled.to_string()]);
        self
    }

    pub fn minimum_stars(&mut self, min_stars: i32) -> &mut Self
    {
        self.params.insert("stars", vec![min_stars.to_string()]);
        self
    }

    pub fn build(&self) -> SearchImagesFilter
    {
        SearchImagesFilter
        {
            params: self.params.clone()
        }
    }
}