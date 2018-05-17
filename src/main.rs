#![feature(drain_filter)]
extern crate futures;
extern crate tokio_core;

use futures::*;
use futures::sync::mpsc;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;
use tokio_core::net::UdpCodec;

use std::io;
use std::net::SocketAddr;

fn main() {
    let target_addr_str = "127.0.0.1";
    let addr_vec: Vec<SocketAddr> = vec!(
        format!("{}:{}", &target_addr_str, 10000).parse().unwrap(),
        format!("{}:{}", &target_addr_str, 10001).parse().unwrap()
    );

    let (tx_array, rx_array) = (0..2).fold((vec!(), vec!()), |mut sum, _| {
        let x = mpsc::channel::<(SocketAddr, Vec<u8>)>(5000);
        sum.0.push(x.0);
        sum.1.push(x.1);
        sum
    });

    sendrecv(addr_vec, rx_array);
}

pub struct LineCodec;

impl UdpCodec for LineCodec {
    type In = (SocketAddr, Vec<u8>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        into.extend(buf);
        addr
    }
}

fn sendrecv(addrs: Vec<SocketAddr>, mut streams: Vec<mpsc::Receiver<(SocketAddr, Vec<u8>)>>) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    for addr in addrs.iter() {
        let sock = UdpSocket::bind(addr, &handle).unwrap();
        let (a_sink, a_stream) = sock.framed(LineCodec).split();
        let task = a_stream.map_err(|_| ()).for_each(|x| {
            println!("recv {:?}", x);
            Ok(())
        });

        let sender = a_sink.sink_map_err(|e| {
            eprintln!("err {:?}", e);
        }).send_all(streams.remove(0));

        handle.spawn(sender.then(|_| Ok(())));
        handle.spawn(task);
    }

    use futures::future;
    core.run(future::empty::<(), ()>()).unwrap();
}
