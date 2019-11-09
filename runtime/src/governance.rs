use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure };
use system::ensure_signed;
use codec::{Encode, Decode};


// Option: {title: String, pot: u64, voters: <Vec:T::AccountId>}
// Voter: {accountId, votedVotes:<Vec: u64>, timeLastVoted: timestamp, balance: balances}
// Vote: {id, creator, method, timestamp, expiredate, voters:<Vec:T::AccountId>, options:<Vec: Option>
#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote<AccountId, Timestamp> {
    id: u64,
    creator: AccountId,
    // index: u64,
    // id: Hash,
    // enum Vote_type　{
    //     Majority Rule, // Majority Rule(expire, threshold: {adaptive quorum})
    //     Stake Locking, // Stake Locking 
    //     Quadratic Voting
    // },
    // option: map u8 => String
    aye: u64,
    nay: u64,
    when: Timestamp, //T::Moment
    // expireblock: 
}


// #[derive(PartialEq, Eq, RuntimeDebug)]
// pub enum Ballot {
//     Aye,
//     Nay
// }

// import Trait from balances, timestamp, event
pub trait Trait: balances::Trait + timestamp::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        //created, voted, withdrawn
		Created(AccountId, u64),
	}
);

decl_storage! {
    // -AllVoteCount: u64 -> increment every time any vote is created
    // -VotesByIndex: map u64 -> Vote<T::AccountId>;
    // -VoteCreator: map u64 => Option<T::AccountId>;
    // VoteByCreator: map T::AccountId => <Vec: u64>;
    trait Store for Module<T: Trait> as GovernanceModule {
        AllVoteCount get(all_vote_count): u64;
        VotesByIndex: map u64 => Vote<T::AccountId, T::Moment>;
        VoteCreator get(creator_of): map u64 => Option<T::AccountId>;
        CreatedVoteCount get(created_by): map T::AccountId => u64; // increment everytime created
        // VoteByCreatorArray get(created_by): map T::AccountId => <Vec: u64>;
        VoteByCreatorArray get(created_by_and_index): map (T::AccountId, u64) => u64;
    }
    // VoteByVoter: map T::AccountId => <Vec: u64>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;
        // Creator Modules
        // create_vote_majority_rule(Title, Option(comma separated), Expiretime)
        fn create_vote(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let vote_num = <AllVoteCount>::get();
            let new_vote_num = <AllVoteCount>::get().checked_add(1)
                .ok_or("Overflow adding vote count")?;
            let vote_count_by_sender = <CreatedVoteCount<T>>::get(sender.clone()).checked_add(1)
                .ok_or("Overflow adding vote count to the sender")?;

            let new_vote = Vote{
                id: vote_num,
                creator: sender.clone(),
                when: <timestamp::Module<T>>::get(),
                aye: 0,
                nay: 0,
            };

            Self::mint_vote(sender, vote_num, new_vote, vote_count_by_sender, new_vote_num)?;
        
            Ok(())
        }
        
        //TODO: add voter modules: {vote}
        // pub fn cast_ballot(origin, reference_index, vote: Ballot) Result -> {
        //     let voter = ensure_signed(origin)?;
        //     ensure!(<VotesByIndex<T>>::exists(&reference_index), "Vote doesn't exists");
        //     // ensure the vote index exists
        //     // ensure the voter hasnt voted in the vote
            
        //     // increment aye or nay
        //     match vote {
        //         Ballot::aye -> {
        //             <VotesByIndex>::get(reference_index).nay.checked_add(1)
        //                 .ok_or("Overflow adding vote count")?;
        //         }
        //         Ballot::nay -> {
        //             <VotesByIndex>::get(reference_index).nay.checked_add(1)
        //                 .ok_or("Overflow adding vote count")?;
        //         }
        //     }
        // }
        
    }
}

impl<T: Trait> Module<T> {
    fn mint_vote(sender: T::AccountId, vote_num: u64, new_vote: Vote<T::AccountId, T::Moment>, vote_count_by_sender: u64, new_vote_num: u64 ) -> Result{
        ensure!(!<VotesByIndex<T>>::exists(&vote_num), "Vote already exists");

        <VotesByIndex<T>>::insert(vote_num.clone(), new_vote);
        <VoteCreator<T>>::insert(vote_num.clone(), sender.clone());
        <CreatedVoteCount<T>>::insert(sender.clone(), vote_count_by_sender);
        <VoteByCreatorArray<T>>::insert((sender.clone(), vote_count_by_sender), vote_num.clone());
        <AllVoteCount>::put(new_vote_num);

        Self::deposit_event(RawEvent::Created(sender, vote_num));

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
