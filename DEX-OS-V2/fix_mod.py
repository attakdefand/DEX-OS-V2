with open('dex-core/src/governance/mod.rs', 'r') as f:
    content = f.read()

# Fix the fast_forward_to_voting_end function
content = content.replace(
    '''    fn fast_forward_to_voting_end(dao: &mut GlobalDAO, proposal_id: &str) {
        if let Some(proposal) = dao.proposals.get_mut(proposal_id) {
            proposal.voting_start = 0;
            proposal.voting_end = 0;
        }
    }''',
    '''    fn fast_forward_to_voting_end(dao: &mut GlobalDAO, proposal_id: &str) {
        if let Some(proposal) = dao.proposals.get_mut(proposal_id) {
            proposal.voting_end = 0;
        }
    }'''
)

with open('dex-core/src/governance/mod.rs', 'w') as f:
    f.write(content)