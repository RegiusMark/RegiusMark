use crate::Wallet;
use native_tls::TlsConnector;
use regiusmark::net::*;
use std::{
    io::Cursor,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};
use tungstenite::{client, handshake::client::Request as WsReq, protocol::Message, stream::Stream};

macro_rules! check_unlocked {
    ($self:expr) => {
        if $self.db.state() != DbState::Unlocked {
            return Err("wallet not unlocked".to_owned());
        }
    };
}

macro_rules! check_args {
    ($args:expr, $count:expr) => {
        if $args.len() != $count + 1 {
            let word = if $count == 1 { "argument" } else { "arguments" };
            return Err(format!("Expected {} {}", $count, word));
        }
    };
}

macro_rules! check_at_least_args {
    ($args:expr, $count:expr) => {
        if $args.len() < $count + 1 {
            let word = if $count == 1 { "argument" } else { "arguments" };
            return Err(format!("Expected at least {} {}", $count, word));
        }
    };
}

macro_rules! hex_to_bytes {
    ($string:expr) => {{
        let len = $string.len() / 2;
        let mut dst = vec![0; len];
        let res = faster_hex::hex_decode($string.as_bytes(), &mut dst);
        match res {
            Ok(_) => Ok(dst),
            Err(_) => Err("invalid hex string"),
        }
    }};
}

pub fn send_print_rpc_req(wallet: &mut Wallet, body: rpc::Request) {
    let res = send_rpc_req(wallet, body);
    match res {
        Ok(res) => println!("{:#?}", res),
        Err(e) => println!("{}", e),
    }
}

pub fn send_rpc_req(wallet: &mut Wallet, body: rpc::Request) -> Result<Msg, String> {
    let buf = {
        let req_id = {
            let id = wallet.req_id;
            wallet.req_id += 1;
            if wallet.req_id == u32::max_value() {
                wallet.req_id = 0;
            }
            id
        };

        let mut buf = Vec::with_capacity(8192);
        let req = Msg {
            id: req_id,
            body: Body::Request(body),
        };
        req.serialize(&mut buf);
        buf
    };

    let mut ws = {
        let mut addr = (wallet.url.host_str().unwrap(), wallet.url.port().unwrap())
            .to_socket_addrs()
            .unwrap();

        let addr = loop {
            match addr.next() {
                Some(addr) => match addr {
                    SocketAddr::V4(_) => break addr,
                    _ => continue,
                },
                None => return Err("No resolved IPv4 addresses found from host".to_owned()),
            }
        };

        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
            .map_err(|e| format!("Failed to connect to host: {:?}", e))?;
        let stream = match wallet.url.scheme() {
            "ws" => Stream::Plain(stream),
            "wss" => {
                let connector = TlsConnector::new().unwrap();
                Stream::Tls(
                    connector
                        .connect(wallet.url.host_str().unwrap(), stream)
                        .unwrap(),
                )
            }
            _ => panic!("Expected node url scheme to be ws or wss"),
        };

        let (ws, _) = client(
            WsReq {
                url: wallet.url.clone(),
                extra_headers: None,
            },
            stream,
        )
        .map_err(|e| format!("Failed to init ws socket: {:?}", e))?;
        ws
    };
    ws.write_message(Message::Binary(buf)).unwrap();
    ws.write_pending().unwrap();

    let msg = loop {
        let msg = ws.read_message().unwrap();
        match msg {
            Message::Binary(res) => {
                let mut cursor = Cursor::<&[u8]>::new(&res);
                let msg = Msg::deserialize(&mut cursor)
                    .map_err(|e| format!("Failed to deserialize response: {}", e))?;
                match msg.body {
                    Body::Error(_) | Body::Response(_) => break msg,
                    Body::Ping(nonce) => {
                        let msg = Msg {
                            id: msg.id,
                            body: Body::Pong(nonce),
                        };
                        let mut buf = Vec::with_capacity(16);
                        msg.serialize(&mut buf);
                        ws.write_message(Message::Binary(buf)).unwrap();
                        ws.write_pending().unwrap();
                    }
                    _ => continue,
                }
            }
            _ => continue,
        }
    };
    let _ = ws.close(None);

    Ok(msg)
}
