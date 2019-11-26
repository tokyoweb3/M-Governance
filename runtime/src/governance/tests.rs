#![cfg(test)]

use super::*;
// use system::ensure_signed;
// use codec::{Encode, Decode};
use support::{
    impl_outer_origin, assert_ok, assert_noop, parameter_types,
    traits::{Currency,}
};
// use rstd::prelude::Vec;
// use sr_primitives::traits::{Hash, CheckedAdd, SaturatedConversion};
use runtime_io::{TestExternalities};
use primitives::{H256};
use sr_primitives::{
    Perbill, traits::{IdentityLookup},
    testing::{Header}
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl Trait for Test {
    type Event = ();
    type Currency = balances::Module<Test>;
    type LockPeriod = BlockHashCount;
}
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumPeriod: u64 = 1;
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
}


impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
    type Hashing = ::sr_primitives::traits::BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type AvailableBlockRatio = AvailableBlockRatio;
    type MaximumBlockLength = MaximumBlockLength;
    type Version = ();
}

impl balances::Trait for Test {
    type Balance = u64;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = ();
    type TransferPayment = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
}

type Balances = balances::Module<Test>;
type Governance = Module<Test>;
type System = system::Module<Test>;

fn build_ext() -> runtime_io::TestExternalities {
    let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
    balances::GenesisConfig::<Test>{
        balances: vec![(1, 100), (2, 100), (10, 100)],
        vesting: vec![],
    }.assimilate_storage(&mut t).unwrap();
    // GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();
    t.into()
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        // System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        // System::on_initialize(System::block_number());
    }
}

#[test]
fn it_works() {
    build_ext().execute_with(|| {
        assert!(true);
    });
    // TestExternalities::default().execute_with(|| {
    //     assert!(true);
    // });
}

#[test]
fn should_pass_vote_creation() {
    TestExternalities::default().execute_with(|| {
        // create a normal vote with account #10.
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec()));
        
        // Vote number shoud be incremented by 1
        assert_eq!(Governance::all_vote_count(), 1);
        assert_eq!(Governance::created_by(10), 1);

        // Creator of vote #1 should be account #10
        assert_eq!(Governance::creator_of(1), Some(10));

        // create a lockvote with account #10
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 10, [00].to_vec()));
        assert_eq!(Governance::all_vote_count(), 2);

        let vote = Governance::votes(2);
        let vote2 = Governance::created_by_and_index((10, 2));

        // vote in VotesByIndex and VoteByCreatorArray should be the same
        assert_eq!(vote, vote2);

        // vote expiry block is the sum of the creation block and given blocknumber
        assert_eq!(vote.vote_ends, vote.when + 10);
        
    });
}

#[test]
fn cast_ballot() {
    TestExternalities::default().execute_with(|| {
        let ballot = Ballot::Aye;
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec()));
        // should pass cast ballot, check vote_type
        assert_ok!(Governance::cast_ballot(Origin::signed(1), 1, ballot));
        assert_noop!(Governance::cast_ballot(Origin::signed(10), 1, ballot), "You cannot vote your own vote.");
        
        let aye_accounts = <VotedAccounts<Test>>::get((1, 0));
        let nay_accounts = <VotedAccounts<Test>>::get((1, 1));
        
        // vote count adds up in respective VotedAccounts array
        assert_eq!(aye_accounts.len(), 1);
        assert_eq!(nay_accounts.len(), 0);
    });
}

fn set_free_balance() {
    let total_balance_before = Balances::total_balance(&1);
    assert_eq!(total_balance_before, 100);
    Balances::make_free_balance_be(&1, 100);
    Balances::make_free_balance_be(&2, 100);
    assert_eq!(total_balance_before, Balances::free_balance(&1) + Balances::reserved_balance(&1));
}
#[test]
fn cast_lockvote() {
    build_ext().execute_with(|| {
        set_free_balance();

        let ballot = Ballot::Aye;
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 5, [00].to_vec()));
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 5, [00].to_vec()));

        let vote = Governance::votes(1);
        // vote should be active
        assert_eq!(vote.concluded, false);
        
        // vote_type should be 1
        assert_noop!(Governance::cast_lockvote(Origin::signed(1), 1, ballot, 50, 10), "This vote is not LockVote.");
        // lock duration should be bigger than expiry time
        assert_noop!(Governance::cast_lockvote(Origin::signed(1), 2, ballot, 50, 4), "Lock duration should be or bigger than vote expiry.");
        // free balance should be bigger than deposit
        assert_noop!(Governance::cast_lockvote(Origin::signed(1), 2, ballot, 101, 10), "You cannot lock more than your free balance!");
        // should succeed casting lockvote
        assert_ok!(Governance::cast_lockvote(Origin::signed(1), 2, ballot, 1, 10));

        // new item in LockBalance
        let lock_vote = <LockBalance<Test>>::get((2, 1));
        // lockbalance deposit should be 1
        assert_eq!(lock_vote.deposit, 1);
        // deposit should be 100
        assert_eq!(lock_vote.duration, 10);
        // until should be 100 + current blocknumber
        assert_eq!(lock_vote.until, 10 + System::block_number());

        // proceed #1 -> #15
        run_to_block(15);
        assert_eq!(System::block_number(), 15);

        // This vote has already been expired.
        assert_noop!(Governance::cast_lockvote(Origin::signed(2), 2, ballot, 1, 10), "This vote has already been expired.");
    });
}

#[test]
fn withdraw() {
    build_ext().execute_with(|| {
        let free_balance_before = Balances::free_balance(&1);
        set_free_balance();
        // create vote
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 5, [00].to_vec()));
        // cast_lock vote
        assert_ok!(Governance::cast_lockvote(Origin::signed(1), 1, Ballot::Aye, 1, 10));

        let free_balance = Balances::free_balance(&1);
        let reserved_balance = Balances::reserved_balance(&1);
        assert_eq!(free_balance, free_balance_before - reserved_balance);
        // cannot withdraw unless LockInfo.until < Block_number
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You have to wait at least until the vote concludes!");

        // proceed #1 -> #15
        run_to_block(15);
        assert_eq!(System::block_number(), 15);
        
        // still, need to be concluded
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You have to wait at least until the vote concludes!");

        assert_ok!(Governance::conclude_vote(Origin::signed(1), 1));

        // withdraw after conclude
        assert_ok!(Governance::withdraw(Origin::signed(1), 1));

        // cannot withdraw twice
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You need to participate lockvoting to call this function!");

        // freebalance is increased by the ammount of deposit
        let free_balance_after = Balances::free_balance(&1);

        assert_eq!(free_balance_after, free_balance_before);
    });
}

#[test]
fn conclude() {
    TestExternalities::default().execute_with(|| {
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 5, [00].to_vec()));

        // proceed #1 -> #15
        run_to_block(15);

        assert_eq!(System::block_number(), 15);
        assert_ok!(Governance::conclude_vote(Origin::signed(1), 1));
        assert_noop!(Governance::conclude_vote(Origin::signed(1), 1), "This vote has already concluded.");
    });
}

