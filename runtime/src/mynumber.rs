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
use sr_primitives::traits::{SaturatedConversion};
use primitives::{U256};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type RSA = Vec<u8>; //2048bit

#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Certification<RSA> {
    pubkey: RSA,    // mynumber pubkey
    encrypted_pubkey: RSA,  // mynumber encrypted by the government
    encrypted_account: RSA,     // account encrypted with pubkey
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
        AccountStore: map T::AccountId => Certification<RSA>;
        
        // My Number => Hash of mynumber signed by Government
        // MyNumberStore get(signature): RSA => RSA;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // register
        pub fn register_account(origin, pubkey: RSA, privkey: RSA, encrypted_pubkey: RSA) -> Result{
            let sender = ensure_signed(origin)?;
            ensure!(!<AccountStore<T>>::exists(&sender), "Your account is already registered.");
            let id = Self::accounts_count();

            let data = &sender.encode();
            // encrypt sender account => result
            let encrypted_account = encrypt(data, &privkey).unwrap();

            let certificate = Certification {
                pubkey,
                encrypted_pubkey,
                encrypted_account,
            };
            
            <AccountStore<T>>::insert(sender, certificate);
            <AccountCount>::put(id + 1);
            print("Account successfully registered!");
            // Self::deposit_event(RawEvent::Registered(&sender, certificate));
            Ok(())
        }

        // verify
        
        // update

        // revoke
    }
}
impl<T: Trait> Module<T> {
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsa::*;
    use support::{
        impl_outer_origin, assert_ok, assert_noop, parameter_types,
    };
    use runtime_io::{TestExternalities};
    use sr_primitives::{
        Perbill, traits::{IdentityLookup},
        testing::{Header}
    };
    use primitives::{H256};
    
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
    const PUBLIC_KEY: [u8; 32] = [
        159, 152, 51, 63, 56, 236, 171, 124,
        45, 135, 54, 162, 205, 236, 198, 245,
        19, 46, 53, 100, 118, 84, 91, 52,
        154, 205, 76, 225, 199, 53, 134, 136
    ];

    //private exponent
    const PRIVATE_KEY: [u8; 32] = [
        25, 179, 118, 205, 152, 40, 219, 84,
        40, 144, 120, 121, 145, 37, 130, 26,
        36, 45, 66, 62, 172, 151, 163, 62,
        196, 188, 207, 172, 93, 93, 87, 81
    ];

    #[test]
    fn can_register() {
        TestExternalities::default().execute_with(||{
            let sender: u64 = 1997;

            // register account
            assert_ok!(MyNumber::register_account(Origin::signed(sender), PUBLIC_KEY.to_vec(), PRIVATE_KEY.to_vec(), PRIVATE_KEY.to_vec()));

            // encode account to byte array
            let account = &sender.encode();

            // encrypt account with private key
            let encrypted_account = encrypt(account, &PRIVATE_KEY).unwrap();

            let cert = Certification {
                pubkey: PUBLIC_KEY.to_vec(),
                encrypted_pubkey: PRIVATE_KEY.to_vec(),
                encrypted_account,
            };
            
            assert_eq!(<AccountStore<Test>>::get(&sender), cert);
        });
    }
}