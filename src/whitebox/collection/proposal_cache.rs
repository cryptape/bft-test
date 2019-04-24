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

#[cfg(test)]
mod test {
    use super::*;

    fn generate_proposal(height: u64, round: u64, proposer: Vec<u8>) -> Proposal {
        Proposal {
            height,
            round,
            content: vec![1, 2, 3],
            proposer,
            lock_round: None,
            lock_votes: Vec::new(),
        }
    }

    #[test]
    fn test_proposal_cache() {
        let mut cache = ProposalCache::new();
        assert_eq!(cache.add(generate_proposal(1, 0, vec![4, 5, 6])), true);
        assert_eq!(cache.add(generate_proposal(1, 0, vec![7, 5, 6])), false);
        assert_eq!(
            cache.get_proposal(1, 0),
            Some(Proposal {
                height: 1,
                round: 0,
                content: vec![1, 2, 3],
                proposer: vec![4, 5, 6],
                lock_round: None,
                lock_votes: Vec::new(),
            })
        );
    }
}
