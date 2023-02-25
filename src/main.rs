extern crate futures;
extern crate tokio;
extern crate tokio_icmp_echo;
use std::{net::Ipv4Addr, time::Duration};

use crate::futures::{future, StreamExt};
use futures::FutureExt;
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};

const AMOUNT_OF_PINGS: usize = 1;

#[derive(Debug)]
struct PingResult {
    addr: Ipv4Addr,
    avg_ping: Option<f64>,
    packet_loss: f64,
}

async fn get_ip_avg_ping(pinger: &tokio_icmp_echo::Pinger, addr: Ipv4Addr) -> PingResult {
    let stream = pinger
        .chain(std::net::IpAddr::V4(addr))
        .timeout(Duration::from_millis(20))
        .stream();
    let mut sum = 0.0;
    let mut received_count = 0;
    stream
        .take(AMOUNT_OF_PINGS)
        .for_each(|mb_time| {
            match mb_time {
                Ok(Some(time)) => {
                    sum += time.as_secs_f64();
                    received_count += 1;
                }
                Ok(None) => {}
                Err(_) => {}
            }
            future::ready(())
        })
        .await;
    PingResult {
        addr,
        avg_ping: if received_count > 0 {
            Some(sum / received_count as f64)
        } else {
            None
        },
        packet_loss: 1.0 - (received_count as f64 / AMOUNT_OF_PINGS as f64),
    }
}

async fn ping_block(image: &mut RgbImage, pinger: &tokio_icmp_echo::Pinger, blocks: [u8; 2]) {
    let mut data_futures = vec![];
    for b3 in 0..255 {
        for b4 in 0..255 {
            // let addr = Ipv4Addr::from([blocks[0], blocks[1], b3, b4]);
            let addr = Ipv4Addr::from([b3, b4, 0, 0]); // faster, 256x256 image
            let result = get_ip_avg_ping(pinger, addr);
            data_futures.push(result);
        }
    }
    let data = futures::future::join_all(data_futures).await;
    for d in data {
        match d.avg_ping {
            Some(res) => {
                image.put_pixel(
                    d.addr.octets()[0].into(),
                    d.addr.octets()[1].into(),
                    image::Rgb([255, 255, 255]),
                );
            }
            None => {}
        }
    }
}

#[tokio::main]
async fn main() {
    // create a new image buffer, 255^2 x 255^2
    // let img: RgbImage = ImageBuffer::new(65025, 65025);
    let mut img: RgbImage = ImageBuffer::new(255, 255);

    let pinger = tokio_icmp_echo::Pinger::new().await.unwrap();
    // for a in 0..2 {
    //     for b in 0..255 {
    //         println!("{} {}", a, b);
    //         ping_block(&mut img, &pinger, [a, b]).await;
    //     }
    // }
    ping_block(&mut img, &pinger, [0, 0]).await;
    // let data = get_ip_avg_ping(pinger, addr).await;
    // println!("avg ping: {:?}", data);
    img.save("test.png").unwrap();
}
