use bollard::container::{
    Config, InspectContainerOptions, RemoveContainerOptions, UpdateContainerOptions,
    WaitContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use chrono::DateTime;
use futures_util::TryStreamExt;

const IMAGE: &str = "ubuntu:latest";

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let d_info = docker.info().await.unwrap();

    let host_ncpu = d_info.ncpu.unwrap();
    let host_memory = d_info.mem_total.unwrap();
    println!("Host NCPU     : {:?}", host_ncpu);
    println!("Host MemTotal : {:?}", host_memory);

    // testing bollard to run container and wait for it to finish

    println!("\nPulling image");
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

    for x in (1..(host_ncpu + 1)).rev() {
        println!("\nCPU count : {}", x);

        // starting container
        let config = Config {
            image: Some(IMAGE),
            cmd: Some(vec!["bash", "-c", "sleep $(nproc)"]),
            ..Default::default()
        };

        let id = docker
            .create_container::<&str, &str>(None, config)
            .await
            .unwrap()
            .id;

        docker
            .update_container(
                &id,
                UpdateContainerOptions::<String> {
                    cpuset_cpus: Some(cpu_shares(x)),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

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

        // get stats
        let container_info = docker
            .inspect_container(&id, Some(InspectContainerOptions { size: false }))
            .await
            .unwrap();
        let container_info = container_info.state.unwrap();
        let start = container_info.started_at.unwrap();
        let start = DateTime::parse_from_rfc3339(start.as_str()).unwrap();
        let end = container_info.finished_at.unwrap();
        let end = DateTime::parse_from_rfc3339(end.as_str()).unwrap();

        let diff = end.signed_duration_since(start);
        println!("diff : {}", diff);

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
}

fn cpu_shares(count: i64) -> String {
    match count {
        c if c < 1 => panic!("cpu count must be a positive number"),
        c if c == 1 => "0".to_string(),
        _ => format!("0-{}", count - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_shares() {
        assert_eq!("0".to_string(), cpu_shares(1));
        assert_eq!("0-4".to_string(), cpu_shares(5));
    }

    #[test]
    #[should_panic]
    fn test_cpu_shares_panic() {
        cpu_shares(-1);
    }
}
