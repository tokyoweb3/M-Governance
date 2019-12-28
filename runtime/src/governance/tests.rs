#![cfg(test)]
use super::*;
use crate::certificate;
use support::{
    impl_outer_origin, assert_ok, assert_noop, parameter_types,
    traits::{Currency}
};
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

impl certificate::Trait for Test {
    type Event = ();
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
type Certificate = certificate::Module<Test>;

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
}

#[test]
fn should_pass_vote_creation() {
    TestExternalities::default().execute_with(|| {
        // fail no option provided
        assert_noop!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 0, [].to_vec()), "At least one option should be provided.");

        // create a normal vote with account #10.
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
        
        // Vote number shoud be incremented by 1
        assert_eq!(Governance::all_vote_count(), 1);
        assert_eq!(Governance::created_by(10), 1);

        // Creator of vote #1 should be account #10
        assert_eq!(Governance::creator_of(1), Some(10));

        // create a lockvote with account #10
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 10, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
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
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
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
#[test]
fn cast_ballot_with_options() {
    TestExternalities::default().execute_with(|| {
        let ballot = Ballot::Aye;
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 0, [[00].to_vec(), [01].to_vec(), [02].to_vec()].to_vec()));
        
        // cast ballot on the first option
        assert_ok!(Governance::cast_ballot_with_options(Origin::signed(1), 1, 0));

        // cannot ballot on the same option
        assert_noop!(Governance::cast_ballot_with_options(Origin::signed(1), 1, 0), "Provided option is already registered.");
        // out of range 
        assert_noop!(Governance::cast_ballot_with_options(Origin::signed(1), 1, 5), "Provided option out of range.");

        // can update option. Change 0 => 1
        assert_ok!(Governance::cast_ballot_with_options(Origin::signed(1), 1, 1));
        assert_eq!(<AccountsByOption<Test>>::get((1, 0)).len() as u8, 0);
        assert_eq!(<AccountsByOption<Test>>::get((1, 1)).len() as u8, 1);
        assert_eq!(<VotedOption<Test>>::get((1, 1)), 1);
        
    });
}

fn set_free_balance() {
    let total_balance_before = Balances::total_balance(&1);
    assert_eq!(total_balance_before, 100);
    Balances::make_free_balance_be(&1, 100);
    Balances::make_free_balance_be(&2, 100);
    assert_eq!(total_balance_before, Balances::free_balance(&1));
}

#[test]
fn account_should_be_registered() {
    build_ext().execute_with(|| {
        let ballot = Ballot::Aye;
        let ca_hash = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
        let cert = sr_primitives::traits::BlakeTwo256::hash(&[221, 222, 223, 224]);
        let signature = sr_primitives::traits::BlakeTwo256::hash(&[11, 12, 13, 14]);
        // new ca at index 1
        assert_ok!(Certificate::register_ca(Origin::signed(1), ca_hash));

        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
        // requires ca at 1
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 10, [00].to_vec(), 1, [[00].to_vec()].to_vec()));

        // should suceed casting ballot which doesnt require account 1 to be approved
        assert_ok!(Governance::cast_ballot(Origin::signed(1), 1, ballot));

        // should fail casting ballot because the account 1 is not approved
        assert_noop!(Governance::cast_ballot(Origin::signed(1), 2, ballot), "Your account is not registered for this CAHash.");

        // approve account 1
        assert_ok!(Certificate::register_account(Origin::signed(1), ca_hash, cert, signature));

        // should suceed
        assert_ok!(Governance::cast_ballot(Origin::signed(1), 2, ballot));
    });
}
#[test]
fn cast_lockvote() {
    build_ext().execute_with(|| {
        set_free_balance();

        let ballot = Ballot::Aye;
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 5, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 5, [00].to_vec(), 0, [[00].to_vec()].to_vec()));

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

        // [BalanceLock { id: [0, 0, 0, 0, 0, 0, 0, 2], amount: 1, until: `18446744073709551615`, reasons: WithdrawReasons { mask: 14 } }]
        let locked_balance = Balances::locks(&1);
        assert_eq!(1, locked_balance[0].amount);
        assert_eq!(u64::max_value().saturated_into::<u64>(), Balances::locks(&1)[0].until);

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
        set_free_balance();
        // create vote. vote.vote_ends = 1 + 5 = 6
        assert_ok!(Governance::create_vote(Origin::signed(10), 1, 5, [00].to_vec(), 0, [[00].to_vec()].to_vec()));
        // cast_lock vote
        assert_ok!(Governance::cast_lockvote(Origin::signed(1), 1, Ballot::Aye, 1, 10));

        let locked_balance = Balances::locks(&1);
        assert_eq!(1, locked_balance[0].amount);

        // cannot withdraw unless vote.concluded == true && LockInfo.until < Block_number
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You have to wait at least until the vote concludes!");

        // cannot conclude before vote is expired
        assert_noop!(Governance::conclude_vote(Origin::signed(1),1), "This vote hasn\'t been expired yet.");

        // proceed #1 -> #7. vote.vote_ends is 6 because #1 + duration.
        run_to_block(7);
        assert_eq!(System::block_number(), 7);
        
        // still, need to be concluded
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You have to wait at least until the vote concludes!");

        // conclude the vote
        assert_ok!(Governance::conclude_vote(Origin::signed(1), 1));

        // withdraw after conclude. Still need to wait until the lock period is over
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You need to wait until the lock period is over!");

        // proceed #5 -> #15
        run_to_block(15);
        assert_eq!(System::block_number(), 15);

        assert_ok!(Governance::withdraw(Origin::signed(1), 1));

        // cannot withdraw twice
        assert_noop!(Governance::withdraw(Origin::signed(1), 1), "You need to participate lockvoting to call this function!");

        // locked balance should be zero
        let locked_balance = Balances::locks(&1);
        assert_eq!(locked_balance, Balances::locks(&100));
    });
}

#[test]
fn conclude() {
    TestExternalities::default().execute_with(|| {
        assert_ok!(Governance::create_vote(Origin::signed(10), 0, 5, [00].to_vec(), 0, [[00].to_vec()].to_vec()));

        // proceed #1 -> #15
        run_to_block(15);

        assert_eq!(System::block_number(), 15);
        assert_ok!(Governance::conclude_vote(Origin::signed(1), 1));
        assert_noop!(Governance::conclude_vote(Origin::signed(1), 1), "This vote has already concluded.");
    });
}

