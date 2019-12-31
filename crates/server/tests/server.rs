use futures::prelude::*;
use regiusmark::{
    constants,
    prelude::{net::ErrorKind, *},
};
use regiusmark_server::WsState;
use std::{
    io::Cursor,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio_tungstenite::tungstenite::Message;

mod common;
pub use common::*;

#[test]
fn successful_broadcast() {
    let minter = TestMinter::new();

    let mut tx = TxVariant::V0(TxVariantV0::MintTx(MintTx {
        base: create_tx_header("0.00000 MARK"),
        to: (&minter.genesis_info().script).into(),
        amount: get_asset("10.00000 MARK"),
        attachment: vec![],
        attachment_name: "".to_owned(),
        script: minter.genesis_info().script.clone(),
    }));

    tx.append_sign(&minter.genesis_info().wallet_keys[1]);
    tx.append_sign(&minter.genesis_info().wallet_keys[0]);

    let res = minter.send_req(rpc::Request::Broadcast(tx)).unwrap();
    assert_eq!(res, Ok(rpc::Response::Broadcast));
}

#[test]
fn get_properties() {
    let minter = TestMinter::new();
    let res = minter.send_req(rpc::Request::GetProperties).unwrap();
    let chain_props = minter.chain().get_properties();
    assert_eq!(res, Ok(rpc::Response::GetProperties(chain_props)));
}

#[test]
fn get_block_unfiltered() {
    let minter = TestMinter::new();

    let res = minter.send_req(rpc::Request::GetBlock(0)).unwrap();
    let other = minter.chain().get_block(0).unwrap();
    assert_eq!(
        res,
        Ok(rpc::Response::GetBlock(FilteredBlock::Block(other)))
    );

    let res = minter.send_req(rpc::Request::GetBlock(2)).unwrap();
    assert_eq!(res, Err(ErrorKind::InvalidHeight));
}

#[test]
fn get_block_filtered_with_addresses() {
    let mut state = create_uninit_state().0;
    let minter = TestMinter::new();

    let mut filter = BlockFilter::new();
    filter.insert((&minter.genesis_info().script).into());
    let res = minter
        .send_msg(
            &mut state,
            Msg {
                id: 0,
                body: Body::Request(rpc::Request::SetBlockFilter(filter.clone())),
            },
        )
        .unwrap()
        .body;
    assert_eq!(res, Body::Response(rpc::Response::SetBlockFilter));
    assert_eq!(state.filter(), Some(&filter));

    {
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Block(block)))
        );
    }

    {
        // Produce block 2, should be filtered
        minter.produce_block().unwrap();

        let tx = {
            let mut tx = TxVariant::V0(TxVariantV0::TransferTx(TransferTx {
                base: create_tx_header("1.00000 MARK"),
                from: (&minter.genesis_info().script).into(),
                to: (&KeyPair::gen().0).into(),
                amount: get_asset("1.00000 MARK"),
                memo: vec![],
                script: minter.genesis_info().script.clone(),
            }));
            tx.append_sign(&minter.genesis_info().wallet_keys[3]);
            tx.append_sign(&minter.genesis_info().wallet_keys[0]);
            tx
        };
        let res = minter.send_req(rpc::Request::Broadcast(tx)).unwrap();
        assert_eq!(res, Ok(rpc::Response::Broadcast));
        // Produce block 3, should not be filtered
        minter.produce_block().unwrap();
    }

    {
        let block = minter.chain().get_block(2).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(2)),
                },
            )
            .unwrap()
            .body;

        let signer = block.signer().unwrap().clone();
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Header((
                block.header(),
                signer
            ))))
        );
    }

    {
        let block = minter.chain().get_block(3).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(3)),
                },
            )
            .unwrap()
            .body;
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Block(block)))
        );
    }
}

#[test]
fn get_block_filtered_all() {
    let mut state = create_uninit_state().0;
    let minter = TestMinter::new();

    {
        // Unfiltered
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Block(block)))
        );
    }

    // Empty filter means filter everything
    let filter = BlockFilter::new();
    let res = minter
        .send_msg(
            &mut state,
            Msg {
                id: 0,
                body: Body::Request(rpc::Request::SetBlockFilter(filter.clone())),
            },
        )
        .unwrap()
        .body;
    assert_eq!(res, Body::Response(rpc::Response::SetBlockFilter));

    assert_eq!(state.filter(), Some(&filter));

    {
        // Filtered
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        let signer = block.signer().unwrap().clone();
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Header((
                block.header(),
                signer
            ))))
        );
    }
}

#[test]
fn clear_block_filter() {
    let mut state = create_uninit_state().0;
    let minter = TestMinter::new();

    // Empty filter means filter everything
    let filter = BlockFilter::new();
    let res = minter
        .send_msg(
            &mut state,
            Msg {
                id: 0,
                body: Body::Request(rpc::Request::SetBlockFilter(filter.clone())),
            },
        )
        .unwrap()
        .body;
    assert_eq!(res, Body::Response(rpc::Response::SetBlockFilter));
    assert_eq!(state.filter(), Some(&filter));

    {
        // Filtered
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        let signer = block.signer().unwrap().clone();
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Header((
                block.header(),
                signer
            ))))
        );
    }

    let res = minter
        .send_msg(
            &mut state,
            Msg {
                id: 0,
                body: Body::Request(rpc::Request::ClearBlockFilter),
            },
        )
        .unwrap()
        .body;
    assert_eq!(res, Body::Response(rpc::Response::ClearBlockFilter));

    {
        // Unfiltered
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Block(block)))
        );
    }
}

#[test]
fn get_full_block() {
    let mut state = create_uninit_state().0;
    let minter = TestMinter::new();

    {
        // Empty filter means filter everything
        let filter = BlockFilter::new();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::SetBlockFilter(filter.clone())),
                },
            )
            .unwrap()
            .body;
        assert_eq!(res, Body::Response(rpc::Response::SetBlockFilter));

        assert_eq!(state.filter(), Some(&filter));
    }

    {
        // Filtered
        let block = minter.chain().get_block(1).unwrap();
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetBlock(1)),
                },
            )
            .unwrap()
            .body;
        let signer = block.signer().unwrap().clone();
        assert_eq!(
            res,
            Body::Response(rpc::Response::GetBlock(FilteredBlock::Header((
                block.header(),
                signer
            ))))
        );
    }

    {
        // Full block
        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::GetFullBlock(1)),
                },
            )
            .unwrap()
            .body;
        let other = minter.chain().get_block(1).unwrap();
        assert_eq!(res, Body::Response(rpc::Response::GetFullBlock(other)));
    }

    // Invalid height
    let res = minter.send_req(rpc::Request::GetFullBlock(2)).unwrap();
    assert_eq!(res, Err(ErrorKind::InvalidHeight));
}

#[test]
fn get_block_range_unfiltered() {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let (tx, rx) = futures::sync::oneshot::channel();

    runtime.spawn(futures::lazy(move || {
        let minter = TestMinter::new();
        let (mut state, rx) = create_uninit_state();
        for _ in 0..100 {
            minter.produce_block().unwrap();
        }
        assert_eq!(minter.chain().get_chain_height(), 101);

        let res = minter.send_msg(
            &mut state,
            Msg {
                id: 123,
                body: Body::Request(rpc::Request::GetBlockRange(0, 100)),
            },
        );
        assert_eq!(res, None);

        let height = Arc::new(AtomicU64::new(0));
        rx.map(move |msg| {
            let msg = match msg {
                Message::Binary(msg) => msg,
                _ => panic!("Expected binary response"),
            };
            let mut cur = Cursor::<&[u8]>::new(&msg);
            Msg::deserialize(&mut cur).unwrap()
        })
        .for_each({
            let height = Arc::clone(&height);
            move |msg: Msg| {
                assert_eq!(msg.id, 123);

                match msg.body {
                    Body::Response(rpc::Response::GetBlock(block)) => {
                        let height = height.fetch_add(1, Ordering::SeqCst);
                        assert!(height <= 100);
                        match block {
                            FilteredBlock::Block(block) => {
                                assert_eq!(block.height(), height);
                            }
                            _ => panic!("Expected a full block"),
                        }
                    }
                    Body::Response(rpc::Response::GetBlockRange) => {
                        assert_eq!(height.load(Ordering::Acquire), 101);
                    }
                    unexp @ _ => panic!("Expected GetBlock response: {:?}", unexp),
                };

                Ok(())
            }
        })
        .and_then(move |()| {
            assert_eq!(height.load(Ordering::Acquire), 101);
            tx.send(()).unwrap();
            Ok(())
        })
    }));

    runtime.block_on(rx).unwrap();
}

#[test]
fn get_block_range_filter_all() {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let (tx, rx) = futures::sync::oneshot::channel();

    runtime.spawn(futures::lazy(move || {
        let minter = TestMinter::new();
        let (mut state, rx) = create_uninit_state();
        for _ in 0..100 {
            minter.produce_block().unwrap();
        }
        assert_eq!(minter.chain().get_chain_height(), 101);

        let res = minter
            .send_msg(
                &mut state,
                Msg {
                    id: 0,
                    body: Body::Request(rpc::Request::SetBlockFilter(BlockFilter::new())),
                },
            )
            .unwrap()
            .body;
        assert_eq!(res, Body::Response(rpc::Response::SetBlockFilter));

        let res = minter.send_msg(
            &mut state,
            Msg {
                id: 123,
                body: Body::Request(rpc::Request::GetBlockRange(0, 100)),
            },
        );
        assert_eq!(res, None);

        let height = Arc::new(AtomicU64::new(0));
        rx.map(move |msg| {
            let msg = match msg {
                Message::Binary(msg) => msg,
                _ => panic!("Expected binary response"),
            };
            let mut cur = Cursor::<&[u8]>::new(&msg);
            Msg::deserialize(&mut cur).unwrap()
        })
        .for_each({
            let height = Arc::clone(&height);
            move |msg: Msg| {
                assert_eq!(msg.id, 123);

                match msg.body {
                    Body::Response(rpc::Response::GetBlock(block)) => {
                        let height = height.fetch_add(1, Ordering::SeqCst);
                        assert!(height <= 100);
                        match block {
                            FilteredBlock::Header((header, _)) => match header {
                                BlockHeader::V0(header) => {
                                    assert_eq!(header.height, height);
                                }
                            },
                            _ => panic!("Expected a partial block"),
                        }
                    }
                    Body::Response(rpc::Response::GetBlockRange) => {
                        assert_eq!(height.load(Ordering::Acquire), 101);
                    }
                    _ => panic!("Expected GetBlock response"),
                };

                Ok(())
            }
        })
        .and_then(move |()| {
            assert_eq!(height.load(Ordering::Acquire), 101);
            tx.send(()).unwrap();
            Ok(())
        })
    }));

    runtime.block_on(rx).unwrap();
}

#[test]
fn get_address_info() {
    let minter = TestMinter::new();
    let addr = (&minter.genesis_info().script).into();
    let res = minter.send_req(rpc::Request::GetAddressInfo(addr)).unwrap();

    let expected = Ok(rpc::Response::GetAddressInfo(AddressInfo {
        net_fee: constants::MARK_FEE_MIN,
        addr_fee: constants::MARK_FEE_MIN
            .checked_mul(constants::MARK_FEE_MULT)
            .unwrap(),
        balance: get_asset("1000.00000 MARK"),
    }));
    assert_eq!(res, expected);
}

#[test]
fn receives_pong_after_ping() {
    let minter = TestMinter::new();
    let (mut state, _) = create_uninit_state();

    let res = minter.send_msg(
        &mut state,
        Msg {
            id: u32::max_value(),
            body: Body::Ping(123),
        },
    );
    assert_eq!(
        res,
        Some(Msg {
            id: u32::max_value(),
            body: Body::Pong(123),
        })
    );
}

#[test]
fn error_with_bytes_remaining() {
    let minter = TestMinter::new();

    let buf = {
        let req = Msg {
            id: 123456789,
            body: Body::Request(rpc::Request::GetBlock(0)),
        };
        let mut buf = Vec::with_capacity(4096);
        req.serialize(&mut buf);

        // Push an extra byte that should trigger the error
        buf.push(0);

        buf
    };

    let res = minter
        .send_bin_msg(&mut create_uninit_state().0, buf)
        .unwrap();
    assert_eq!(res.id, 123456789);
    assert_eq!(res.body, Body::Error(ErrorKind::BytesRemaining));
}

#[test]
fn eof_returns_max_u32_id() {
    let minter = TestMinter::new();

    let buf = {
        let req = Msg {
            id: 123456789,
            body: Body::Request(rpc::Request::GetBlock(0)),
        };
        let mut buf = Vec::with_capacity(4096);
        req.serialize(&mut buf);

        // Delete an extra byte causing an EOF error triggering a failure to deserialize the message
        buf.truncate(buf.len() - 1);

        buf
    };

    let res = minter
        .send_bin_msg(&mut create_uninit_state().0, buf)
        .unwrap();
    assert_eq!(res.id, u32::max_value());
    assert_eq!(res.body, Body::Error(ErrorKind::Io));
}

#[test]
fn response_id_matches_request() {
    let minter = TestMinter::new();
    let addr = (&minter.genesis_info().script).into();

    let buf = {
        let req = Msg {
            id: 123456789,
            body: Body::Request(rpc::Request::GetAddressInfo(addr)),
        };
        let mut buf = Vec::with_capacity(4096);
        req.serialize(&mut buf);

        buf
    };
    let res = minter
        .send_bin_msg(&mut create_uninit_state().0, buf)
        .unwrap();

    let expected = Msg {
        id: 123456789,
        body: Body::Response(rpc::Response::GetAddressInfo(AddressInfo {
            net_fee: constants::MARK_FEE_MIN,
            addr_fee: constants::MARK_FEE_MIN
                .checked_mul(constants::MARK_FEE_MULT)
                .unwrap(),
            balance: get_asset("1000.00000 MARK"),
        })),
    };
    assert_eq!(res, expected);
}

fn create_uninit_state() -> (WsState, futures::sync::mpsc::Receiver<Message>) {
    let (tx, rx) = futures::sync::mpsc::channel(8);
    (
        WsState::new(SocketAddr::from(([127, 0, 0, 1], 7777)), tx),
        rx,
    )
}
