#![feature(drain_filter)]
extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate bytes;

use futures::*;
use futures::sync::mpsc;
use tokio::net::{UdpSocket, UdpFramed};
use tokio_io::codec::BytesCodec;
use tokio::executor::current_thread::{ CurrentThread, RunError };
use bytes::Bytes;

use std::net::SocketAddr;

fn main() {
    let target_addr_str = "127.0.0.1";
    let addr_vec: Vec<SocketAddr> = vec!(
        format!("{}:{}", &target_addr_str, 10000).parse().unwrap(),
        format!("{}:{}", &target_addr_str, 10001).parse().unwrap()
    );

    let (_tx_array, rx_array) = (0..2).fold((vec!(), vec!()), |mut sum, _| {
        let x = mpsc::channel::<(Bytes, SocketAddr)>(5000);
        sum.0.push(x.0);
        sum.1.push(x.1);
        sum
    });

    let _x = sendrecv(addr_vec, rx_array);
}

fn sendrecv(addrs: Vec<SocketAddr>, mut streams: Vec<mpsc::Receiver<(Bytes, SocketAddr)>>) -> Result<(), RunError> {
    let mut current_thread = CurrentThread::new();

    for addr in addrs.iter() {
        let sock = UdpSocket::bind(&addr).unwrap();
        let (a_sink, a_stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        let task = a_stream.map_err(|_| ()).for_each(|x| {
            println!("recv {:?}", x);
            Ok(())
        });

        let sender = a_sink.sink_map_err(|e| {
            eprintln!("err {:?}", e);
        }).send_all(streams.remove(0)).then(|_| Ok(()));

        current_thread.spawn({
            sender.join(task)
                .map(|_| ())
                .map_err(|e| println!("error = {:?}", e))
        });
    }

    current_thread.run()
}
