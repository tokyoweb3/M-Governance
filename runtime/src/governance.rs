use support::{decl_module, decl_storage, decl_event, dispatch::Result };
// use rstd::vec::Vec;
use system::ensure_signed;
use codec::{Encode, Decode};


// Option: {title: String, pot: u64, voters: <Vec:T::AccountId>}
// Voter: {accountId, votedVotes:<Vec: u64>, timeLastVoted: timestamp, balance: balances}
// Vote: {id, creator, method, timestamp, expiredate, voters:<Vec:T::AccountId>, options:<Vec: Option>
#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Vote<AccountId> {
    id: u64,
    creator: AccountId,
}

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
    // -VoteCreator: map u64 => Option<T::AccountID>;
    // VoteByCreator: map T::AccountId => <Vec: u64>;
    trait Store for Module<T: Trait> as GovernanceModule {
        AllVoteCount get(all_vote_count): u64;
        VotesByIndex: map u64 => Vote<T::AccountId>;
        VoteCreator get(creator_of): map u64 => Option<T::AccountId>;
        CreatedVoteCount get(created_by): map T::AccountId => u64; // increment everytime created
        // VoteByCreaterArray get(created_by): map T::AccountId => <Vec: u64>;
    }
    // VoteByVoter: map T::AccountId => <Vec: u64>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;
        // creator modules
        fn create_vote(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let vote_num = <AllVoteCount>::get().checked_add(1)
                .ok_or("Overflow adding vote count")?;
            // let vote_num = <AllVoteCount>::get() + 1;
            let vote_count_by_sender = <CreatedVoteCount<T>>::get(sender.clone()).checked_add(1)
                .ok_or("Overflow adding vote count to the sender")?;

            let new_vote = Vote{
                id: vote_num,
                creator: sender.clone(),
            };
            <VotesByIndex<T>>::insert(vote_num.clone(), new_vote);
            <VoteCreator<T>>::insert(vote_num.clone(), sender.clone());
            <CreatedVoteCount<T>>::insert(sender.clone(), vote_count_by_sender);
            <AllVoteCount>::put(vote_num);

            Self::deposit_event(RawEvent::Created(sender, vote_num));

            Ok(())
        }

        //TODO: add voter modules: {vote}
        
    }
}