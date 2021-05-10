#[macro_use]
extern crate prettytable;

use bollard::image::CreateImageOptions;
use bollard::Docker;
use clap::{App, Arg};
use futures_util::TryStreamExt;
use prettytable::Table;
use std::fs::File;

mod docker;

const IMAGE: &str = "ubuntu:latest";

#[tokio::main]
async fn main() {
    let matches = App::new("Baywatch")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Output CSV file"),
        )
        .get_matches();
    let docker = Docker::connect_with_local_defaults().unwrap();
    let d_info = docker.info().await.unwrap();
    let host_ncpu = d_info.ncpu.unwrap();
    let host_memory = d_info.mem_total.unwrap();

    println!("Docker infos");
    println!("host ncpu : {:?}", host_ncpu);
    println!("host memtotal : {:?}\n", host_memory);

    // pulling image
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

    let mut table = Table::new();
    table.add_row(row!["CPU", "DURATION (ms)",]);
    for x in (1..(host_ncpu + 1)).rev() {
        let diff = docker::run_container(&docker, IMAGE, x).await.unwrap();
        table.add_row(row![x, diff,]);
    }
    table.printstd();

    if let Some(o) = matches.value_of("output") {
        let file = File::create(o).unwrap();
        table.to_csv(file).unwrap();
    }
}
