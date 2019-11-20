use support::{
    decl_module, decl_storage, decl_event, dispatch::Result, ensure, print, Parameter,
    traits::{
        LockableCurrency, WithdrawReason, WithdrawReasons, LockIdentifier, Get, Currency,
    }
};
use system::ensure_signed;
use codec::{Encode, Decode};
use rstd::prelude::Vec;
use sr_primitives::traits::{Hash, CheckedAdd, Member};

const EXAMPLE_ID: LockIdentifier = *b"lockvote";

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
    concluded: bool,
    vote_type: u8,
     //or :map u64 -> AccountId   vec.len().push()
    // expireblock: 
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Ballot {
    Aye,
    Nay,
}

#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct LockInfo<Balance, Timestamp> {
    deposit: Balance,
    duration: Timestamp,
}

pub type ReferenceIndex = u64;
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

// import Trait from balances, timestamp, event
pub trait Trait: balances::Trait + timestamp::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
    type LockPeriod: Get<Self::BlockNumber>;
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        //created, voted, withdrawn, finalized
        Created(AccountId, u64),
        Voted(AccountId, u64, Ballot),
        Concluded(u64),
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
        VotesByIndex get(votes): map u64 => Vote<T::AccountId, T::Moment>;

        // Creator
        VoteCreator get(creator_of): map u64 => Option<T::AccountId>;
        CreatedVoteCount get(created_by): map T::AccountId => u64; // increment everytime created

        // VoteByCreatorArray get(created_by): map T::AccountId => <Vec: u64>;
        VoteByCreatorArray get(created_by_and_index): map (T::AccountId, u64) => Vote<T::AccountId, T::Moment>;

        VoteResults: map u64 => Vec<u64>;
        VoteIndexHash get(index_hash): map u64 => T::Hash;

        // VotedAccounts:[aye:[AccountId], nay:[AccountId],....]
        VotedAccounts: map (ReferenceIndex, u8) => Vec<T::AccountId>;

        LockBalance: map (ReferenceIndex, T::AccountId) => LockInfo<BalanceOf<T>, T::Moment>;
        LockCount get(lock_count): u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // Creator Modules
        // Create a new vote
        // TODO: Takes expiring time, title as data: Vec, voting_type
        pub fn create_vote(origin, vote_type:u8, data: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(data.len() <= 256, "listing data cannot be more than 256 bytes");

            let new_vote_num = <AllVoteCount>::get().checked_add(1)
                .ok_or("Overflow adding vote count")?;
            let vote_count_by_sender = <CreatedVoteCount<T>>::get(sender.clone()).checked_add(1)
                .ok_or("Overflow adding vote count to the sender")?;
            let exp_length = 60000.into(); // 30sec for test
            let now = <timestamp::Module<T>>::get();
            // check if resolved if now > vote_exp
            let vote_exp = now.checked_add(&exp_length).ok_or("Overflow when setting application expiry.")?;
            let hashed = <T as system::Trait>::Hashing::hash(&data);
            let new_vote = Vote{
                id: new_vote_num,
                vote_type,
                creator: sender.clone(),
                when: now,
                vote_ends: vote_exp,
                concluded: false,
            };

            Self::mint_vote(sender, new_vote, vote_count_by_sender, new_vote_num)?;
            <VoteIndexHash<T>>::insert(new_vote_num, hashed);
            Ok(())
        }

        fn cast_lockvote(origin, reference_index: ReferenceIndex, ballot: Ballot, deposit: BalanceOf<T>, duration: T::BlockNumber) -> Result {
            let sender = ensure_signed(origin)?;
            let vote = Self::votes(&reference_index);
            let now = <timestamp::Module<T>>::now();
            

            ensure!(<VotesByIndex<T>>::exists(&reference_index), "Vote doesn't exists");
            ensure!(vote.creator != sender, "You cannot vote your own vote.");
            ensure!(vote.vote_ends > now, "This vote has already been expired.");
            ensure!(vote.vote_type == 1, "This vote is not LockVote.");
            // lock function
            <LockBalance<T>>::mutate((reference_index, &sender), |lockinfo| lockinfo.deposit += deposit);
            T::Currency::set_lock(
                EXAMPLE_ID,
                &sender,
                deposit,
                T::LockPeriod::get(),
                WithdrawReasons::except(WithdrawReason::TransactionPayment),
            );
            // Balance_of(sender).checked_sub(deposit).ok_or("Underflow");

            Ok(())
        }
        fn withdraw(origin, reference_index: ReferenceIndex) -> Result {
            // ensure!(vote has ended)
            // ensure!(sender has voted the vote)
            // let deposit = <LockBalance<T>>::get((reference_index, sender));
            // Balance_of(sender).checked_add(deposit);
            // <LockBalance<T>>::remove((reference_index, sender));
            Ok(())
        }
        // Voter modules
        // cast_ballot checks
            // a. the vote exists
            // b. vote hasnt expired
            // c. the voter hasnt voted yet in the same option. If voted in different option, change the vote.
        fn cast_ballot(origin, reference_index: ReferenceIndex, ballot: Ballot) -> Result {
            let sender = ensure_signed(origin)?;
            let vote = <VotesByIndex<T>>::get(&reference_index);
            let now = <timestamp::Module<T>>::now();
            ensure!(<VotesByIndex<T>>::exists(&reference_index), "Vote doesn't exists");
            ensure!(vote.creator != sender, "You cannot vote your own vote.");
            ensure!(vote.vote_ends > now, "This vote has already been expired.");
            ensure!(vote.vote_type == 0, "This vote is LockVote. Use 'cast_lockvote' instead!");
            let mut accounts_aye = <VotedAccounts<T>>::get((reference_index, 0));
            let mut accounts_nay = <VotedAccounts<T>>::get((reference_index, 1));
            // TODO: Clean up with ::mutate instead of update func
            // keep track of voter's id in aye or nay vector in Vote
            // Voter can change his vote b/w aye and nay
            // Voter cannot vote twice
            match ballot {
                Ballot::Aye => {
                    ensure!(!accounts_aye.contains(&sender), "You have already voted aye.");
                    // if sender has voted for the other option, remove from the array
                    if accounts_nay.contains(&sender) {
                        let i = accounts_nay.iter().position(|x| x == &sender).unwrap() as usize;
                        accounts_nay.remove(i);
                    } 
                    accounts_aye.push(sender.clone());
                    // <VotedAccounts<T>>::mutate((reference_index, 0), |vec| {
                    //     *vec = accounts_aye
                    // });
                    <VotedAccounts<T>>::insert((reference_index, 0), accounts_aye);
                    print("Ballot casted Aye!");
                }
                Ballot::Nay => {
                    ensure!(!accounts_nay.contains(&sender), "You have already voted nay.");
                    if accounts_aye.contains(&sender) {
                        let i = accounts_aye.iter().position(|x| x == &sender).unwrap() as usize;
                        accounts_aye.remove(i);
                    } 
                    accounts_nay.push(sender.clone());
                    // <VotedAccounts<T>>::mutate((reference_index, 1), |vec| *vec = accounts_nay);
                    <VotedAccounts<T>>::insert((reference_index, 1), accounts_nay);
                    print("Ballot casted Nay!");
                }
            }
            Self::deposit_event(RawEvent::Voted(sender, reference_index, ballot));
            Ok(())
        }

        // conclude a vote given expired
        // anyone can call this function, and Vote.concluded returns true
        pub fn conclude_vote(_origin, reference_index: u64) -> Result {
            let vote = <VotesByIndex<T>>::get(reference_index);
            // ensure the vote is concluded before tallying
            ensure!(vote.concluded == false, "This vote has already concluded.");
            let now = <timestamp::Module<T>>::now();
            // double check
            ensure!(now > vote.vote_ends, "This vote hasn't been expired yet.");
            Self::tally(reference_index)?;
            // For some reason Storage is not reflected, but works.
            <VotesByIndex<T>>::mutate(reference_index, |vote| vote.concluded = true);
            <VoteByCreatorArray<T>>::mutate((vote.creator, reference_index), |vote| vote.concluded = true);
            Self::deposit_event(RawEvent::Concluded(reference_index));
            print("Vote concluded.");
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn lock(){

    }

    fn mint_vote(sender: T::AccountId, new_vote: Vote<T::AccountId, T::Moment>, vote_count_by_sender: u64, new_vote_num: u64 ) -> Result{
        ensure!(!<VotesByIndex<T>>::exists(&new_vote_num), "Vote already exists");

        <VotesByIndex<T>>::insert(new_vote_num.clone(), &new_vote);
        <VoteCreator<T>>::insert(new_vote_num.clone(), sender.clone());
        <CreatedVoteCount<T>>::insert(sender.clone(), vote_count_by_sender);
        <AllVoteCount>::put(new_vote_num.clone());
        <VoteByCreatorArray<T>>::insert((sender.clone(), new_vote_num), new_vote);

        Self::deposit_event(RawEvent::Created(sender, new_vote_num));
        print("Vote created!");
        Ok(())
    }

    // updated after ballot being casted
    fn update_vote(reference_index: u64, new_vote: Vote<T::AccountId, T::Moment>) -> Result {
        <VotesByIndex<T>>::remove(&reference_index);
        <VotesByIndex<T>>::insert(&reference_index, &new_vote);
        <VoteByCreatorArray<T>>::remove((<VotesByIndex<T>>::get(reference_index).creator, &reference_index));
        <VoteByCreatorArray<T>>::insert((<VotesByIndex<T>>::get(reference_index).creator, &reference_index), &new_vote);
        
        Ok(())
    }

    // only called after the vote expired
    fn tally(reference_index: u64) -> Result {
        // ensure: vote has ended(expiringblock < currentblocknumber), called by the module?
        let aye_count = <VotedAccounts<T>>::get((reference_index, 0)).len() as u64;
        let nay_count = <VotedAccounts<T>>::get((reference_index, 1)).len() as u64;
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
