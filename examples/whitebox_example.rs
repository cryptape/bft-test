pub mod util;

use bft_core::{types::*, Core};
use bft_test::whitebox::actuator::Actuator;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use env_logger::Builder;
use log::LevelFilter::Info;
use std::thread;
use util::whitebox_util::{generate_authority, TestSupport};

const INIT_HEIGHT: u64 = 0;
const INIT_ROUND: u64 = 0;

#[derive(Clone, Debug)]
struct BftTest {
    recv4test: Receiver<BftMsg>,
    recv4core: Receiver<BftMsg>,
    send2test: Sender<BftMsg>,
    send_commit: Sender<Commit>,
    bft: Core,
}

impl BftTest {
    fn start() -> (Sender<BftMsg>, Receiver<BftMsg>, Receiver<Commit>) {
        let (test2core, core4test) = unbounded();
        let (s_commit, r_commit) = unbounded();
        let (mut engine, recv4core) = BftTest::init(s_commit, core4test);
        engine.bft.to_bft_core(BftMsg::Start).unwrap();

        thread::spawn(move || loop {
            engine.process();
        });

        (test2core, recv4core, r_commit)
    }

    fn init(send_commit: Sender<Commit>, recv4test: Receiver<BftMsg>) -> (Self, Receiver<BftMsg>) {
        let (bft, recv4core) = Core::start(vec![0]);
        let (send2test, recv) = unbounded();
        (
            BftTest {
                recv4test,
                recv4core,
                send2test,
                send_commit,
                bft,
            },
            recv,
        )
    }

    fn process(&mut self) {
        select! {
            recv(self.recv4core) -> msg => {
                if let Ok(test_msg) = msg {
                    println!("Send {:?} to Test", test_msg.clone());
                    match test_msg {
                        BftMsg::Commit(c) => self.send_commit.send(c).unwrap(),
                        BftMsg::GetProposalRequest(_h) => return,
                        _ => self.send2test.send(test_msg.clone()).unwrap(),
                    }
                }
            }
            recv(self.recv4test) -> msg => {
                if let Ok(bft_msg) = msg {
                    println!("Send {:?} to BFT core",bft_msg.clone());
                    self.bft.to_bft_core(bft_msg).unwrap();
                }
            }
        }
    }
}

fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter(None, Info).init();

    let (s, r, r_commit) = BftTest::start();
    let ts = TestSupport::new(s, r, r_commit);
    let mut test = Actuator::new(
        ts,
        INIT_HEIGHT,
        INIT_ROUND,
        generate_authority(),
        "db/test.db",
    );
    // let case = bft_test::test_case::two_byzantine_one_offline();
    // let _ = test.proc_test(case).map_err(|err| panic!("bft error {:?}", err));
    let _ = test.all_test().map_err(|err| panic!("bft error {:?}", err));
    ::std::process::exit(0);
}
