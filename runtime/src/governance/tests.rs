#![cfg(test)]

use super::*;
use system::ensure_signed;
use codec::{Encode, Decode};
use support::{
    decl_module, decl_storage, decl_event, dispatch::Result, ensure, print, impl_outer_origin, assert_ok, assert_noop, parameter_types,
    traits::{
        LockableCurrency, WithdrawReason, WithdrawReasons, LockIdentifier, Get, Currency, Imbalance,
    }
};
use rstd::prelude::Vec;
use sr_primitives::traits::{Hash, CheckedAdd, SaturatedConversion};
use runtime_io::{TestExternalities};
use primitives::{H256, Blake2Hasher};
use sr_primitives::{
    BuildStorage, Perbill, traits::{BlakeTwo256, IdentityLookup},
    testing::{Digest, DigestItem, Header}
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

type PositiveImbalance<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalance<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

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

// type PositiveImbalance<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;
// type NegativeImbalance<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;
// type Currency = dyn LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
// type LockPeriod = dyn Get<Self::BlockNumber>;

type Governance = Module<Test>;
type System = system::Module<Test>;

// fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
//     system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
// }

// #[test]
// fn my_runtime_test() {
//     with_externalities(&mut new_test_ext(), || {
//         assert_ok!(Governance::start_auction());
//         //10ブロック進める
//         run_to_block(10);
//         assert_ok!(Governance::end_auction());
//     });
// }

#[test]
fn it_works() {
    TestExternalities::default().execute_with(|| {
        assert!(true);
    });
}

#[test]
fn should_pass_vote_creation() {
    TestExternalities::default().execute_with(|| {
        assert_ok!(Governance::create_vote(Origin::signed(1), 0, 10, [00].to_vec()));
        assert_eq!(Governance::all_vote_count(), 1);
        assert_ok!(Governance::create_vote(Origin::signed(1), 1, 10, [00].to_vec()));
        assert_eq!(Governance::all_vote_count(), 2);
    });
}

#[test]
fn cast_ballot() {
    TestExternalities::default().execute_with(|| {
        // should pass cast ballot, check vote_type
        // vote should be active
        // cannot vote twice
        // cannot vote own vote
        // vote count adds up in respective VotedAccounts array
        // if voted for the other option, change mutate the array
    });
}

#[test]
fn cast_lockvote() {
    TestExternalities::default().execute_with(|| {
        // deposit should be higher than free balance
        // vote should be active
        // duration should be longer than vote expiry
        // new item in LockBalance
    });
}

#[test]
fn withdraw() {
    TestExternalities::default().execute_with(|| {
        // should be the same person in LockBalance
        // vote should be concluded
        // should withdraw only once
        // freebalance is increased by the ammount of deposit
        // delete item in LockBalance after withdraw
    });
}

#[test]
fn conclude() {
    TestExternalities::default().execute_with(|| {
        // vote.concluded = true
        // cannot conclude twice
    });
}

