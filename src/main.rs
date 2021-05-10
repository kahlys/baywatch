#[macro_use]
extern crate prettytable;

use bollard::Docker;
use clap::{App, Arg};
use prettytable::Table;
use std::fs::File;

mod docker;

#[tokio::main]
async fn main() {
    let matches = App::new("Baywatch")
        .arg(
            Arg::with_name("docker-image")
                .short("i")
                .long("image")
                .takes_value(true)
                .help("Docker image")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Output CSV file"),
        )
        .get_matches();

    let image = matches.value_of("docker-image").unwrap();

    let docker = Docker::connect_with_local_defaults().unwrap();
    let d_info = docker.info().await.unwrap();
    let host_ncpu = d_info.ncpu.unwrap();
    let host_memory = d_info.mem_total.unwrap();

    println!("Docker infos");
    println!("host ncpu : {:?}", host_ncpu);
    println!("host memtotal : {:?}\n", host_memory);

    let mut table = Table::new();
    table.add_row(row!["CPU", "DURATION (ms)",]);
    for x in (1..(host_ncpu + 1)).rev() {
        let diff = docker::run_container(&docker, image, x).await.unwrap();
        table.add_row(row![x, diff,]);
    }
    table.printstd();

    if let Some(o) = matches.value_of("output") {
        let file = File::create(o).unwrap();
        table.to_csv(file).unwrap();
    }
}
