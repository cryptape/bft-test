use crate::whitebox::types::Proposal;
use lru_cache::LruCache;

#[derive(Debug, Clone)]
pub(crate) struct ProposalCache {
    pub(crate) proposals: LruCache<u64, ProposalRoundCollector>,
}

impl ProposalCache {
    pub(crate) fn new() -> Self {
        ProposalCache {
            proposals: LruCache::new(16),
        }
    }

    pub(crate) fn add(&mut self, proposal: Proposal) -> bool {
        let height = proposal.height;
        let round = proposal.round;

        if self.proposals.contains_key(&height) {
            self.proposals
                .get_mut(&height)
                .unwrap()
                .add(round, proposal)
        } else {
            let mut round_proposals = ProposalRoundCollector::new();
            round_proposals.add(round, proposal);
            self.proposals.insert(height, round_proposals);
            true
        }
    }

    pub(crate) fn get_proposal(&mut self, height: u64, round: u64) -> Option<Proposal> {
        self.proposals
            .get_mut(&height)
            .and_then(|prc| prc.get_proposal(round))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProposalRoundCollector {
    pub(crate) round_proposals: LruCache<u64, Proposal>,
}

impl ProposalRoundCollector {
    pub(crate) fn new() -> Self {
        ProposalRoundCollector {
            round_proposals: LruCache::new(16),
        }
    }

    pub(crate) fn add(&mut self, round: u64, proposal: Proposal) -> bool {
        if self.round_proposals.contains_key(&round) {
            false
        } else {
            self.round_proposals.insert(round, proposal);
            true
        }
    }

    pub(crate) fn get_proposal(&mut self, round: u64) -> Option<Proposal> {
        self.round_proposals.get_mut(&round).cloned()
    }
}
