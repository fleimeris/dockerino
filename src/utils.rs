use std::collections::HashMap;
use std::error::Error;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct DockerError
{
    pub(crate) message: String
}

pub trait ObjectConverter
{
    fn parse_params(&self, params: &HashMap<&'static str, String>) -> Result<String, Box<dyn Error>>
    {
        let mut result = String::new();

        let mut first = true;

        for (key, value) in params
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

    fn url_encoded(&self, params: &HashMap<&'static str, Vec<String>>) -> Result<String, Box<dyn Error>>
    {
        let json = serde_json::to_string(params)?;
        let url_encoded = urlencoding::encode(json.as_str());

        Ok(url_encoded.to_string())
    }
}