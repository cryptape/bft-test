use crate::error::BftError;
use crate::whitebox::{
    collection::{
        proposal_cache::ProposalCache, storage::Storage, util::Msg, vote_cache::VoteCache,
    },
    correctness::test_case::*,
    types::*,
};

use log::{debug, info};
use rand::{thread_rng, Rng};
use time::Timespec;

use std::collections::HashSet;
use std::thread;
use std::time::Instant;

/// A whitebox testing actuator.
pub struct Actuator<T> {
    function: T,

    height: u64,
    round: u64,
    lock_round: Option<u64>,
    lock_votes: Vec<Vote>,
    lock_proposal: Option<Vec<u8>>,
    authority_list: Vec<Address>,
    proposal: Vec<u8>,
    byzantine: Vec<Vec<u8>>,
    sleep_ms: u64,
    storage: Storage,
    vote_cache: VoteCache,
    proposal_cache: ProposalCache,
    msg_cache: HashSet<FrameRecv>,
    stime: Instant,
    htime: Timespec,
}

impl<T> Actuator<T>
where
    T: Support + Clone + Send + 'static,
{
    /// A function to create a new testing acutator. The `height` is the initial height
    /// and the `round` is the initial round. The `authority_list` should be a `Vec` with
    /// length 4. The first one in authority list should be the address of the testing
    /// node. The `db_path` is the path of database.
    pub fn new(
        function: T,
        height: u64,
        round: u64,
        authority_list: Vec<Address>,
        db_path: &str,
    ) -> Self {
        Actuator {
            function,
            height,
            round,
            lock_round: None,
            lock_votes: Vec::new(),
            lock_proposal: None,
            authority_list,
            proposal: Vec::new(),
            byzantine: byzantine_proposal(),
            sleep_ms: 150,
            storage: Storage::new(db_path),
            vote_cache: VoteCache::new(),
            proposal_cache: ProposalCache::new(),
            msg_cache: HashSet::new(),
            stime: Instant::now(),
            htime: Timespec::new(0, 0),
        }
    }

    /// A function to set a new authority list. Likewise it the new one should be with
    /// length 4, and the first one should be the address of the testing node.
    pub fn set_authority_list(&mut self, authority_list: Vec<Address>) {
        self.authority_list = authority_list;
    }

    /// A function to set a new sleep time as millisecond. The sleep time is the duration
    /// time after send precommit votes before commit. If the testing node is wait for
    /// more votes before commit when the framework get commit, it may return error of no
    /// commit. Use this function to set a longer duration. The default duration is 150 milliseconds.
    pub fn set_sleep_time(&mut self, ms: u64) {
        self.sleep_ms = ms;
    }

    /// A function to do whitebox testing with test cases input. The test cases are generated
    /// in `test_case`.
    pub fn proc_test(&mut self, cases: BftTest) -> BftResult<()> {
        self.init();
        for case in cases.iter() {
            println!("{:?}", case);
            if case == &SHOULD_COMMIT {
                thread::sleep(::std::time::Duration::from_millis(self.sleep_ms));
                if let Some(commit) = self.function.try_get_commit() {
                    self.storage_msg(Msg::Commit(commit.clone()));
                    self.check_commit(commit)?;
                    let status = self.generate_status();
                    self.function.send(FrameSend::Status(status));
                    debug!(
                        "Height {:?}, use time {:?}",
                        self.height,
                        time::get_time() - self.htime
                    );
                    self.goto_next_height();
                } else {
                    return Err(BftError::NoCommit(self.height));
                }
            } else if case == &NULL_ROUND {
                self.goto_next_round();
            } else if case == &SHOULD_NOT_COMMIT {
                thread::sleep(::std::time::Duration::from_millis(120));
                if self.function.try_get_commit().is_some() {
                    return Err(BftError::CommitInvalid(self.height));
                }
                self.goto_next_round();
            } else {
                let prevote = case[0..3].to_vec();
                let precommit = case[3..6].to_vec();
                let proposer = self.function.cal_proposer(self.height, self.round);

                if proposer == 0 {
                    let feed = self.generate_feed();
                    self.function.send(FrameSend::Feed(feed));
                    self.check_proposal()?;
                } else if proposer < self.authority_list.len() {
                    self.generate_proposal(proposer, self.lock_round, self.lock_votes.clone());
                } else {
                    panic!("Proposer index beyond authority list!");
                }

                self.generate_prevote(prevote);
                self.check_prevote()?;
                self.generate_precommit(precommit);
                self.check_precommit()?;
            }
        }
        info!(
            "Test success, total test time: {:?}",
            Instant::now() - self.stime
        );
        Ok(())
    }

    /// A function to do all whitebox tests.
    pub fn all_test(&mut self) -> BftResult<()> {
        let all_test_cases = all_cases();
        info!("Start all BFT test cases");
        // info!("Do test test round leap");
        // self.proc_test(round_leap_cases())?;
        for (test_name, test_case) in all_test_cases.into_iter() {
            info!("Do test {:?}", test_name);
            let _ = self
                .proc_test(test_case)
                .map_err(|err| panic!("Error in test {:?}: {:?}", test_name, err));
        }
        info!("All BFT test cases pass");
        Ok(())
    }

    fn generate_feed(&self) -> Feed {
        let mut proposal = vec![0, 0, 0, 0, 0, 0];
        while self.byzantine.contains(&proposal) {
            let mut rng = thread_rng();
            for ii in proposal.iter_mut() {
                *ii = rng.gen();
            }
        }
        let res = Feed {
            height: self.height,
            proposal,
        };
        self.storage_msg(Msg::Feed(res.clone()));
        res
    }

    fn generate_status(&self) -> Status {
        let res = Status {
            height: self.height,
            authority_list: self.authority_list.clone(),
        };
        self.storage_msg(Msg::Status(res.clone()));
        res
    }

    fn generate_proposal(
        &mut self,
        auth_index: usize,
        lock_round: Option<u64>,
        lock_votes: Vec<Vote>,
    ) {
        let proposal = if self.lock_proposal.is_some() {
            self.lock_proposal.clone().unwrap()
        } else {
            let mut tmp = vec![0, 0, 0, 0, 0, 0];
            while self.byzantine.contains(&tmp) {
                let mut rng = thread_rng();
                for ii in tmp.iter_mut() {
                    *ii = rng.gen();
                }
            }
            tmp
        };
        self.proposal = proposal.clone();

        let proposal = Proposal {
            height: self.height,
            round: self.round,
            content: proposal,
            proposer: self.authority_list[auth_index].clone(),
            lock_round,
            lock_votes,
        };
        self.proposal_cache.add(proposal.clone());
        self.storage_msg(Msg::Proposal(proposal.clone()));
        self.function.send(FrameSend::Proposal(proposal.clone()));
        debug!("Send proposal {:?}", proposal);
    }

    fn generate_prevote(&mut self, prevote: Vec<u8>) {
        let proposal = if self.lock_proposal.is_none() {
            self.proposal.clone()
        } else {
            self.lock_proposal.clone().unwrap()
        };

        for (i, item) in prevote.iter().enumerate().take(3) {
            if *item == NORMAL {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Prevote,
                    proposal: proposal.clone(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send normal prevote {:?}", vote);
            } else if *item == BYZANTINE {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Prevote,
                    proposal: self.byzantine[i].clone(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send byzantine prevote {:?}", vote);
            } else if *item == NIL {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Prevote,
                    proposal: Vec::new(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send nil prevote {:?}", vote);
            }
        }
    }

    fn generate_precommit(&mut self, precommit: Vec<u8>) {
        let proposal = if self.lock_proposal.is_none() {
            self.proposal.clone()
        } else {
            self.lock_proposal.clone().unwrap()
        };

        for (i, item) in precommit.iter().enumerate().take(3) {
            if *item == NORMAL {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Precommit,
                    proposal: proposal.clone(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send normal precommit {:?}", vote);
            } else if *item == BYZANTINE {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Precommit,
                    proposal: self.byzantine[i].clone(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send byzantine precommit {:?}", vote);
            } else if *item == NIL {
                let vote = Vote {
                    height: self.height,
                    round: self.round,
                    vote_type: VoteType::Precommit,
                    proposal: Vec::new(),
                    voter: self.authority_list[i + 1].clone(),
                };

                self.storage_msg(Msg::Vote(vote.clone()));
                self.function.send(FrameSend::Vote(vote.clone()));
                self.vote_cache.add(vote.clone());
                debug!("Send nil precommit {:?}", vote);
            }
        }
    }

    fn check_prevote(&mut self) -> BftResult<()> {
        let vote = self.receive_vote(VoteType::Prevote)?;
        debug!(
            "Check prevote at height {:?}, round {:?}",
            self.height, self.round
        );
        let mut clean_flag = true;

        if let Some(prevote_set) =
            self.vote_cache
                .get_voteset(self.height, self.round, VoteType::Prevote)
        {
            // check prevote condition
            for (p, count) in prevote_set.votes_by_proposal {
                if self.is_above_threshold(count).is_ok() {
                    clean_flag = false;
                    if !p.is_empty() {
                        self.set_polc(p);
                    } else {
                        self.clean_polc();
                    }
                }
            }
        } else {
            return Err(BftError::IllegalVote(vote));
        }
        if clean_flag {
            self.proposal = Vec::new();
        }
        Ok(())
    }

    fn check_precommit(&mut self) -> BftResult<()> {
        let vote = self.receive_vote(VoteType::Precommit)?;
        debug!(
            "Check precommit at height {:?}, round {:?}",
            self.height, self.round
        );

        if let Some(prevote_set) =
            self.vote_cache
                .get_voteset(self.height, self.round, VoteType::Prevote)
        {
            // check precommit condition
            self.is_above_threshold(prevote_set.count)?;
            for (p, count) in prevote_set.votes_by_proposal.iter() {
                if self.is_above_threshold(*count).is_ok() {
                    if p != &vote.proposal {
                        return Err(BftError::PrecommitErr(self.height, self.round));
                    }

                    let polc = prevote_set.extract_polc(
                        self.height,
                        self.round,
                        VoteType::Prevote,
                        &vote.proposal.clone(),
                    );
                    if polc.len() < 3 {
                        return Err(BftError::PrecommitDiffPoLC(self.height, self.round));
                    }
                    self.lock_votes = polc;
                }
            }
        } else {
            return Err(BftError::IllegalVote(vote));
        }
        Ok(())
    }

    fn check_commit(&mut self, commit: Commit) -> BftResult<()> {
        info!(
            "Check commit at height {:?}, round {:?}",
            self.height, self.round
        );

        if self.byzantine.contains(&commit.result)
            || self.lock_round.is_none()
            || self.proposal != commit.result
            || self
                .proposal_cache
                .get_proposal(self.height, self.round)
                .unwrap()
                .content
                != commit.result
        {
            return Err(BftError::CommitIncorrect(self.height));
        }

        if let Some(precommit_set) =
            self.vote_cache
                .get_voteset(self.height, self.round, VoteType::Precommit)
        {
            if precommit_set
                .extract_polc(self.height, self.round, VoteType::Precommit, &commit.result)
                .len()
                < 3
            {
                return Err(BftError::CommitIncorrect(self.height));
            }
        }
        Ok(())
    }

    fn check_proposal(&mut self) -> BftResult<()> {
        info!(
            "Check proposal at height {:?}, round{:?}",
            self.height, self.round
        );

        let mut msg;
        loop {
            let tmp = self.function.recv();
            if !self.msg_cache.contains(&tmp) {
                msg = tmp;
                break;
            }
        }
        self.msg_cache.insert(msg.clone());

        match msg {
            FrameRecv::Proposal(p) => Ok(p),
            _ => Err(BftError::IllegalProposal(self.height, self.round)),
        }
        .and_then(|p| {
            if self.lock_round.is_some() {
                if p.lock_round.is_none() || Some(p.content.clone()) != self.lock_proposal {
                    return Err(BftError::IllegalProposal(self.height, self.round));
                }
            } else if p.lock_round.is_some() {
                return Err(BftError::IllegalProposal(self.height, self.round));
            }
            self.proposal_cache.add(p.clone());
            self.proposal = p.content;
            Ok(())
        })
    }

    fn receive_vote(&mut self, vote_type: VoteType) -> BftResult<Vote> {
        let mut msg;
        let mut vote;
        loop {
            let tmp = self.function.recv();
            if !self.msg_cache.contains(&tmp) {
                msg = tmp;
                break;
            }
        }
        self.msg_cache.insert(msg.clone());

        match msg {
            FrameRecv::Proposal(p) => return Err(BftError::AbnormalProposal(p)),
            FrameRecv::Vote(v) => vote = v,
        }

        if vote.vote_type != vote_type || self.byzantine.contains(&vote.proposal) {
            // check vote type and vote proposal
            return Err(BftError::IllegalVote(vote));
        }
        self.vote_cache.add(vote.clone());
        self.storage_msg(Msg::Vote(vote.clone()));
        debug!("Receive vote {:?}", vote.clone());
        Ok(vote)
    }

    fn is_above_threshold(&self, num: usize) -> BftResult<()> {
        if num * 3 <= self.authority_list.len() * 2 {
            return Err(BftError::ShouldNotPrecommit(self.height, self.round));
        }
        Ok(())
    }

    fn set_polc(&mut self, proposal: Vec<u8>) {
        self.proposal = proposal.clone();
        self.lock_round = Some(self.round);
        self.lock_proposal = Some(proposal);
    }

    fn clean_polc(&mut self) {
        self.proposal = Vec::new();
        self.lock_round = None;
        self.lock_votes.clear();
        self.lock_proposal = None;
    }

    fn storage_msg(&self, msg: Msg) {
        let res = self.storage.insert(msg.clone());
        if res.is_err() {
            panic!("SQLite Error {:?} when insert {:?}", res, msg);
        }
    }

    fn goto_next_height(&mut self) {
        self.vote_cache.clear_prevote_count();
        self.msg_cache.clear();
        self.clean_polc();
        self.round = 0;
        self.height += 1;
        self.htime = time::get_time();
        info!("Go to next height");
    }

    fn goto_next_round(&mut self) {
        if self.lock_round.is_none() {
            self.proposal = Vec::new();
        } else {
            self.proposal = self.lock_proposal.clone().unwrap();
        }
        self.round += 1;
    }

    fn init(&mut self) {
        info!("Init a unit test");
        let gensis = self.generate_status();
        self.height += 1;
        self.storage_msg(Msg::Status(gensis.clone()));
        self.function.send(FrameSend::Status(gensis));
        self.htime = time::get_time();
    }
}
