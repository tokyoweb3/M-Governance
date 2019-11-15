use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure };
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::prelude::Vec;
use sr_primitives::traits::{CheckedAdd };

// Option: {title: String, pot: u64, voters: <Vec:T::AccountId>}
// Voter: {accountId, votedVotes:<Vec: u64>, timeLastVoted: timestamp, balance: balances}
// Vote: {id, creator, method, timestamp, expiredate, voters:<Vec:T::AccountId>, options:<Vec: Option>
#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote<AccountId, Timestamp> {
    id: u64,
    creator: AccountId,
    when: Timestamp, //T::Moment
    vote_ends: Timestamp,
    vote_type: u8,
    aye: Vec<AccountId>,
    nay: Vec<AccountId>, //or :map u64 -> AccountId   vec.len().push()
    // expireblock: 
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Ballot {
    Aye,
    Nay,
}

pub type ReferenceIndex = u64;

// import Trait from balances, timestamp, event
pub trait Trait: balances::Trait + timestamp::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        //created, voted, withdrawn, finalized
        Created(AccountId, u64),
        Voted(AccountId, u64, Ballot),
	}
);

decl_storage! {
    // -AllVoteCount: u64 -> increment every time any vote is created
    // -VotesByIndex: map u64 -> Vote<T::AccountId>;
    // -VoteCreator: map u64 => Option<T::AccountId>;
    // VoteByCreator: map T::AccountId => <Vec: u64>;
    trait Store for Module<T: Trait> as GovernanceModule {
        // All votes
        AllVoteCount get(all_vote_count): u64;
        VotesByIndex get(index_of): map u64 => Vote<T::AccountId, T::Moment>;

        // Creator
        VoteCreator get(creator_of): map u64 => Option<T::AccountId>;
        CreatedVoteCount get(created_by): map T::AccountId => u64; // increment everytime created

        // VoteByCreatorArray get(created_by): map T::AccountId => <Vec: u64>;
        VoteByCreatorArray get(created_by_and_index): map (T::AccountId, u64) => Vote<T::AccountId, T::Moment>;

        VoteResults: map u64 => Vec<u64>;
    }
    // VoteByVoter: map T::AccountId => <Vec: u64>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;
        // Creator Modules
        fn create_vote(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let new_vote_num = <AllVoteCount>::get().checked_add(1)
                .ok_or("Overflow adding vote count")?;
            let vote_count_by_sender = <CreatedVoteCount<T>>::get(sender.clone()).checked_add(1)
                .ok_or("Overflow adding vote count to the sender")?;
            let exp_length = 12000.into(); // 2 mins for test
            let now = <timestamp::Module<T>>::now();
            // check if resolved if now > vote_exp
            let vote_exp = now.checked_add(&exp_length).ok_or("Overflow when setting application expiry.")?;

            let new_vote = Vote{
                id: new_vote_num,
                vote_type: 0,
                creator: sender.clone(),
                when: now,
                vote_ends: vote_exp,
                aye: Vec::new(),
                nay: Vec::new()
            };

            Self::mint_vote(sender, new_vote, vote_count_by_sender, new_vote_num)?;
            
            Ok(())
        }

        // Voter modules
        fn cast_ballot(origin, reference_index: ReferenceIndex, ballot: Ballot) -> Result {
            let sender = ensure_signed(origin)?;
            let vote = <VotesByIndex<T>>::get(&reference_index);
            let now = <timestamp::Module<T>>::now();
            ensure!(<VotesByIndex<T>>::exists(&reference_index), "Vote doesn't exists");
            ensure!(vote.vote_ends > now, "This vote has already been expired.");
            // ensure the voter hasnt voted in the vote or change it to the other
            
            // push voter id to aye or nay
            // TODO: decode?
            match ballot {
                Ballot::Aye => {
                    // create a new updated Vote, remove the previous Vote, insert the new Vote
                    let mut new_vote = <VotesByIndex<T>>::get(reference_index);
                    new_vote.aye.push(sender.clone());
                    Self::update_vote(reference_index, new_vote, sender, ballot)?;
                }
                Ballot::Nay => {
                    let mut new_vote = <VotesByIndex<T>>::get(reference_index);
                    new_vote.nay.push(sender.clone());
                    Self::update_vote(reference_index, new_vote, sender, ballot)?;
                }
            }
            Ok(())
        }

        fn conclude_vote(origin, reference_index: ReferenceIndex) -> Result {
            let _sender = ensure_signed(origin)?;
            // ensure the vote is expired before tallying
            let now = <timestamp::Module<T>>::now();
            let vote = <VotesByIndex<T>>::get(reference_index);
            ensure!(now > vote.vote_ends, "This vote hasn't been expired yet.");
            Self::tally(reference_index)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn mint_vote(sender: T::AccountId, new_vote: Vote<T::AccountId, T::Moment>, vote_count_by_sender: u64, new_vote_num: u64 ) -> Result{
        ensure!(!<VotesByIndex<T>>::exists(&new_vote_num), "Vote already exists");

        <VotesByIndex<T>>::insert(new_vote_num.clone(), &new_vote);
        <VoteCreator<T>>::insert(new_vote_num.clone(), sender.clone());
        <CreatedVoteCount<T>>::insert(sender.clone(), vote_count_by_sender);
        <VoteByCreatorArray<T>>::insert((sender.clone(), vote_count_by_sender), new_vote);
        <AllVoteCount>::put(new_vote_num);

        Self::deposit_event(RawEvent::Created(sender, new_vote_num));

        Ok(())
    }

    // updated after ballot being casted
    fn update_vote(reference_index: ReferenceIndex, new_vote: Vote<T::AccountId, T::Moment>, sender: T::AccountId, ballot: Ballot) -> Result {
        <VotesByIndex<T>>::remove(&reference_index);
        <VotesByIndex<T>>::insert(&reference_index, &new_vote);
        <VoteByCreatorArray<T>>::remove((&sender, &reference_index));
        <VoteByCreatorArray<T>>::insert((&sender, reference_index), new_vote);
        
        Self::deposit_event(RawEvent::Voted(sender, reference_index, ballot));
        Ok(())
    }

    // only called after the vote expired
    fn tally(reference_index: u64) -> Result {
        // ensure: vote has ended(expiringblock < currentblocknumber), called by the module?
        let vote = <VotesByIndex<T>>::get(&reference_index);
        let aye_count = vote.aye.len() as u64;
        let nay_count = vote.nay.len() as u64;
        let mut result:Vec<u64> = Vec::new();
        result.push(aye_count);
        result.push(nay_count);
        <VoteResults>::insert(reference_index, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
	use super::*;
    use support::{decl_module, decl_storage, decl_event, dispatch::Result };
    use system::ensure_signed;
    use codec::{Encode, Decode};

	#[test]
	fn vote_creation() {
        
	}
}
