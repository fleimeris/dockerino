use dockerino::docker::Docker;
use tokio;

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::ops::Deref;
    use serde::de::Unexpected::Str;
    use dockerino::docker::Docker;
    use dockerino::images::{DeleteBuilderCacheFilterBuilder, DockerBuildParamsBuilder, ListImagesFilter, SearchImagesFilterBuilder};

    #[tokio::test]
    async fn get_all_images() {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let all_images = images.get_images_all(None).await;

        match all_images
        {
            Ok(images) => println!("{:?}", images),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn get_all_images_with_filter() {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();
        
        let filter = ListImagesFilter::new()
            .dangling(true)
            .build();

        let all_images = images.get_images_all(Some(filter)).await;

        match all_images
        {
            Ok(images) => println!("{:?}", images.len()),
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

    #[tokio::test]
    async fn tag_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let result = images
            .tag_image("35c73078fa43", Option::Some("someuser/someimage"),
            Option::Some("test")).await;

        match result
        {
            Ok(()) => println!("Image was renamed successfully "),
            Err(ref error) => panic!("{:?}", error)
        };
    }

    #[tokio::test]
    async fn get_image_history()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let result = images
            .get_image_history("35c73078fa43").await;

        match result
        {
            Ok(result) => println!("{:?}", result),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn export_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let result = images
            .export_image("b01b6452ecf7", "/home/benas/testas123.tar").await;

        match result
        {
            Ok(()) => println!("Image exported"),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn import_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let result = images
            .import_image("/home/benas/testas123.tar").await;

        match result
        {
            Ok(()) => println!("Image imported"),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn push_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        images.tag_image("7d49d79ed7b5", Some("localhost:5000/test_push"),
                         Some("test")).await.expect("Failed to tag image");

        let result = images
            .push_image("localhost:5000/test_push", "registry",
                        Some("test")).await;

        match result
        {
            Ok(result) => println!("Image pushed"),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn build_image()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let params = DockerBuildParamsBuilder::new().tag("t").build();

        let result = images.build_image("/home/benas/test", Some(params)).await;

        match result
        {
            Err(error) => panic!("{:?}", error),
            Ok(result) => println!("{:?}", result)
        };
    }

    #[tokio::test]
    async fn search_images()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let params = SearchImagesFilterBuilder::new()
            .is_official(true)
            .is_automated(true)
            .build();

        let result = images.search_images("mysql", Some(5), Some(params)).await;

        match result
        {
            Ok(images) => println!("Found these images: {:?}", images),
            Err(error) => panic!("{:?}", error)
        }
    }

    #[tokio::test]
    async fn delete_builder_cache()
    {
        let docker = Docker::new(String::from("/var/run/docker.sock"));

        let images = docker.images();

        let mut filter = DeleteBuilderCacheFilterBuilder::new()
            .in_use()
            .shared()
            .private()
            .until("1000h")
            .build();

        let result = images.delete_builder_cache(Some(0), Some(true), Some(filter)).await;

        match result
        {
            Ok(response) => println!("Cache delete response: {:?}", response),
            Err(error) => panic!("{:?}", error)
        }
    }
}