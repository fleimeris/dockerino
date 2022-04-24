use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct DockerError
{
    pub(crate) message: String
}