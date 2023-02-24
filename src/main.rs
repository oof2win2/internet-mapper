extern crate futures;
extern crate tokio;
extern crate tokio_icmp_echo;
use crate::futures::{future, StreamExt};

const AMOUNT_OF_PINGS: usize = 3;

#[derive(Debug)]
struct PingResult {
    addr: std::net::IpAddr,
    avg_ping: Option<f64>,
    packet_loss: f64,
}

async fn get_ip_avg_ping(pinger: tokio_icmp_echo::Pinger, addr: std::net::IpAddr) -> PingResult {
    let stream = pinger.chain(addr).stream();
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

#[tokio::main]
async fn main() {
    let addr = std::env::args().nth(1).unwrap().parse().unwrap();

    let pinger = tokio_icmp_echo::Pinger::new().await.unwrap();
    let data = get_ip_avg_ping(pinger, addr).await;
    println!("avg ping: {:?}", data);
}
