/// Runtime module implementing MyNumber authentication functions
/// with register, update, verify, revoke functions.
/// With this runtime module, anyone with official mynumber card provided by Japan Agency for Local Authority
/// can prove his/her key pair is legit and tied to the digital certificate of Mynumber card.
/// This module works together with governance module, to permit users to vote only one time, allowing the implementation of more secure voting method.

use crate::rsa::*;
use rstd::prelude::*;
use support::{dispatch::Result, decl_storage, decl_module, decl_event, ensure, print};
use system::{ensure_signed};
use codec::{Encode, Decode};
use sr_primitives::traits::{};
use primitives::{};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type RSA = Vec<u8>; //2048bit
pub type Pubkey = Vec<u8>; //2048bits


#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Certification<Pubkey, Hash> {
    pubkey: Pubkey,    // mynumber pubkey
    cert: Hash,  // mynumber signed by CA
    encrypted_account: Hash,     // account encrypted with pubkey
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Registered(AccountId, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as MyNumber {
        AccountCount get(accounts_count): u64;
        // Public Key => MyNumber, Signed Hashed of Public Key with MyNumberPublicKey: 
        AccountStore: map T::AccountId => Certification<Pubkey, T::Hash>;
        
        // My Number => Hash of mynumber signed by Government
        // MyNumberStore get(signature): RSA => RSA;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // register
        pub fn register_account(origin, pubkey: Pubkey, cert: T::Hash, encrypted_account: T::Hash) -> Result{
            ensure!(pubkey.len() <= 256, "RSA public key should be 256 bytes");

            let sender = ensure_signed(origin)?;
            ensure!(!<AccountStore<T>>::exists(&sender), "Your account is already registered.");
            let id = Self::accounts_count();

            let certificate = Certification {
                pubkey,
                cert,
                encrypted_account,
            };
            
            <AccountStore<T>>::insert(sender, certificate);
            <AccountCount>::put(id + 1);
            print("Account successfully registered!");
            // Self::deposit_event(RawEvent::Registered(&sender, certificate));
            Ok(())
        }

        pub fn get_account_hex(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let key1: T::AccountId = sender;
            let key1_as_vec: Vec<u8> = key1.encode();
            print(&key1_as_vec[..]);
            Ok(())
            // d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        }
        // update

        // revoke
    }
}


impl<T: Trait> Module<T> {
    // helper function to check if the account is registered.
    // TODO: verify certification. Now only checks runtime storage.
    pub fn check_account(sender:T::AccountId) -> Result {
        ensure!(<AccountStore<T>>::exists(sender), "Your account was not found in AccountStore!");
        Ok(())
    }   
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsa::*;
    use support::{
        impl_outer_origin, assert_ok, assert_noop, parameter_types
    };
    use runtime_io::{TestExternalities};
    use sr_primitives::{
        Perbill, traits::{IdentityLookup, Hash, Member, },
        testing::{Header}
    };
    use primitives::{H256};
    // extern crate rustc_hex;
    use rustc_hex::{FromHex, ToHex};
    // use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    impl Trait for Test {
        type Event = ();
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
        type Version = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
    }

    type MyNumber = Module<Test>;
    
    //public modulus
    #[test]
    fn can_register() {
        TestExternalities::default().execute_with(||{
            // register account
            // let pubkey: Vec<u8> = pub_str.from_hex().unwrap();
            let pubkey: Vec<u8> = [11, 22, 33, 44].to_vec();
            // let cert: H256 = sr_primitives::traits::Hash::hash(&[444, 555, 66, 777]);
            let cert = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
            let signed_account = sr_primitives::traits::BlakeTwo256::hash(&[122, 122, 122, 122]);

            assert_ok!(MyNumber::register_account(Origin::signed(1), pubkey.clone(), cert, signed_account));

            let certification = Certification {
                pubkey,
                cert,
                encrypted_account: signed_account,
            };
            
            assert_eq!(<AccountStore<Test>>::get(1), certification);
        });
    }
}