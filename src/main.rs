use std::time::{Duration, Instant};

use bollard::container::{Config, RemoveContainerOptions, WaitContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::TryStreamExt;

const IMAGE: &'static str = "ubuntu:latest";

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let d_info = docker.info().await.unwrap();

    let host_ncpu = d_info.ncpu.unwrap();
    let host_memory = d_info.mem_total.unwrap();
    println!("Host NCPU     : {:?}", host_ncpu);
    println!("Host MemTotal : {:?}", host_memory);

    // testing bollard to run container and wait for it to finish

    println!("Pulling image");
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    // starting container
    let start = Instant::now();

    println!("Starting container");
    let config = Config {
        image: Some(IMAGE),
        cmd: Some(vec!["sleep", "5"]),
        ..Default::default()
    };

    let id = docker
        .create_container::<&str, &str>(None, config)
        .await
        .unwrap()
        .id;

    docker.start_container::<String>(&id, None).await.unwrap();

    // wait to finish

    docker
        .wait_container(
            &id,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        )
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    let duration = start.elapsed();
    println!("Duration : {:?}", duration);

    // remove container

    docker
        .remove_container(
            &id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .unwrap();
}
