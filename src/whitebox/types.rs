use crate::error::{BftError, FrameError};
use serde_derive::{Deserialize, Serialize};

pub(crate) type Hash = Vec<u8>;
pub(crate) type Address = Vec<u8>;
/// BFT result.
pub type BftResult<T> = Result<T, BftError>;
/// Test framework result.
pub type FrameResult<T> = Result<T, FrameError>;

/// Framework receive message types.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FrameRecv {
    /// Proposal message.
    Proposal(Proposal),
    /// Vote message.
    Vote(Vote),
}

/// Framework send message types.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FrameSend {
    /// Proposal message.
    Proposal(Proposal),
    /// Vote message.
    Vote(Vote),
    /// Proposal content message.
    Feed(Feed),
    /// Rich status message.
    Status(Status),
}

/// A proposal.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Proposal {
    /// The height of a proposal
    pub height: u64,
    /// The round of a proposal.
    pub round: u64,
    /// The propoal content.
    pub content: Hash,
    /// The address of proposer.
    pub proposer: Address,
    /// The lock round of a proposal. If the proposal has not been locked, it should be `None`.
    pub lock_round: Option<u64>,
    /// The lock votes of a proposal. If the proposal has not been locked, it should be an empty `Vec`.
    pub lock_votes: Vec<Vote>,
}

/// A vote.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Vote {
    /// The height of a vote.
    pub height: u64,
    /// The round of a vote.
    pub round: u64,
    /// The vote type.
    pub vote_type: VoteType,
    /// The proposal of a vote.
    pub proposal: Hash,
    /// The address of voter.
    pub voter: Address,
}

/// A commit.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Commit {
    /// The commit node ID.
    pub node: u8,
    /// The height of commit.
    pub height: u64,
    /// The consensus result.
    pub result: Vec<u8>,
}

/// The proposal content.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Feed {
    /// The height of a feed.
    pub height: u64,
    /// The content of a feed.
    pub proposal: Vec<u8>,
}

/// A rich status.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Status {
    /// The height of a status.
    pub height: u64,
    /// The new authority list of next height.
    pub authority_list: Vec<Address>,
}

/// Vote type.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VoteType {
    /// Prevote vote type.
    Prevote,
    /// Precommit vote type.
    Precommit,
}

/// Whitebox test support.
pub trait Support {
    /// Send a `FrameSend` message to the testing node.
    fn send(&self, msg: FrameSend);
    /// Receive a `FrameRecv` message from the testing node.
    fn recv(&self) -> FrameRecv;
    /// Try once to get a commit message from the testing node.
    /// If it does not commit, return `None`.
    fn try_get_commit(&self) -> Option<Commit>;
    /// Stop the testing node.
    fn stop(&self);
    /// Determine the proposer index in the authority list by
    /// the given height and round.
    fn cal_proposer(&self, height: u64, round: u64) -> usize;
}
