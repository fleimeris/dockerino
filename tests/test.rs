use dockerino::docker::Docker;
use tokio;

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use dockerino::docker::Docker;

    #[tokio::test]
    async fn it_works() {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let all_images = images.get_images_all().await;

        match all_images
        {
            Ok(images) => println!("{:?}", images),
            Err(error) => panic!("{:?}", error)
        }

        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn get_single_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let image = images.get_image("ss").await;

        match image
        {
            Ok(ref image) => println!("{:?}", image),
            Err(ref error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn delete_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let image = images
            .delete_image("c859aafa677c", true, false).await;

        match image
        {
            Ok(response) => println!("{:?}", response),
            Err(ref error) => panic!("{:?}", error)
        }
    }
}