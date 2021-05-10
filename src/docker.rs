use bollard::container::{
    Config, InspectContainerOptions, RemoveContainerOptions, UpdateContainerOptions,
    WaitContainerOptions,
};
use chrono::DateTime;
use futures_util::TryStreamExt;

pub async fn run_container(docker: &bollard::Docker, image: &str, cpu: i64) -> Result<i64, String> {
    // starting container
    let id = docker
        .create_container::<&str, &str>(
            None,
            Config {
                image: Some(image),
                cmd: Some(vec!["bash", "-c", "sleep $(nproc)"]),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    docker
        .update_container(
            &id,
            UpdateContainerOptions::<String> {
                cpuset_cpus: Some(cpu_shares(cpu)),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    docker.start_container::<String>(&id, None).await.unwrap();

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

    let container_info = docker
        .inspect_container(&id, Some(InspectContainerOptions { size: false }))
        .await
        .unwrap();
    let container_info = container_info.state.unwrap();
    let start = DateTime::parse_from_rfc3339(container_info.started_at.unwrap().as_str()).unwrap();
    let end = DateTime::parse_from_rfc3339(container_info.finished_at.unwrap().as_str()).unwrap();

    let diff = end.signed_duration_since(start);

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

    Ok(diff.num_milliseconds())
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
