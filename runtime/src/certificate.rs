/// Runtime module implementing Certificate authentication functions
/// with register, update, verify, revoke functions.
/// With this runtime module, anyone with official certificate card provided by Japan Agency for Local Authority
/// can prove his/her key pair is legit and tied to the digital certificate of Certificate card.
/// This module works together with governance module, to permit users to vote only one time, allowing the implementation of more secure voting method.

// use crate::rsa::*;
use rstd::prelude::*;
use support::{dispatch::Result, decl_storage, decl_module, decl_event, ensure, print};
use system::{ensure_signed};
use codec::{Encode, Decode};
// use sr_primitives::traits::{};
// use primitives::{};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


#[derive(PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Certification <Hash> {
    cert: Hash,  // CA certificate
    signature: Hash,     // account signed with pubkey
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Registered(AccountId, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Certificate {
        AccountCount get(accounts_count): u64;
        AccountStore: map T::AccountId => Certification<T::Hash>;

        CAHashCount get(cahash_count): u64;
        pub CAHashByIndex get(cahash_by_index): map u64 => T::Hash;
        IndexByCAHash get(index_by_cahash): map T::Hash => u64;
        CAHashes get(cahashes): Vec<T::Hash>;

        CADataByIndex get(ca_data_by_index): map u64 => Vec<u8>;
        CAData get(ca_data): Vec<Vec<u8>>;

        AccountsByCAHash get(accounts_by_cahash): map T::Hash => Vec<T::AccountId>;
        CAHashesByAccount get(cahashes_by_account): map T::AccountId => Vec<T::Hash>;

        CertificateStore get(certificate_store): map (T::AccountId, T::Hash) => Certification<T::Hash>;
        // used for checking if any duplicate exists.
        CertHashes get(cert_hashes): Vec<T::Hash>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // register new ca. Takes CAHash
        // checks:
        //  - Hash doesn't exist
        pub fn register_ca(origin, ca_hash: T::Hash, data: Vec<u8>) -> Result{
            let _sender = ensure_signed(origin)?;
            let new_count: u64 = Self::cahash_count().checked_add(1)
                .ok_or("Overflow adding CAHash count.")?;

            // Hash should be unique
            ensure!(!<IndexByCAHash<T>>::exists(&ca_hash), "Provided CAHash is already registered.");
            ensure!(!<CAHashByIndex<T>>::exists(&new_count), "Error: Overlapping count exists.");
            ensure!(!<CADataByIndex>::exists(&new_count), "Provided CA is already registered.");

            let mut new_hashes = Self::cahashes();
            new_hashes.push(ca_hash);

            <CAHashByIndex<T>>::insert(new_count, ca_hash);
            <IndexByCAHash<T>>::insert(ca_hash, new_count);

            <CAHashes<T>>::put(new_hashes);

            <CADataByIndex>::insert(new_count, &data);

            let mut ca_data_arr = Self::ca_data();
            ca_data_arr.push(data);
            <CAData>::put(ca_data_arr);

            <CAHashCount>::put(new_count);
            print("New CA has been successly registered!");
            Ok(())
        }

        // register account. Takes, CAHash, certificate, signature
        // checks:
        //  - a: CAHash should exists
        //  - b: Account doesn't exist in AccountsByCAHash
        //  - c: Hash doesn't exist in HashByAccount
        //  - d: Nothing exists in CertificateStore(Account, Hash)
        //  - e: Cert isn't used for registering another account
        pub fn register_account(origin, ca_hash: T::Hash, cert: T::Hash, signature: T::Hash) -> Result{
            let sender = ensure_signed(origin)?;

            let mut accounts = <AccountsByCAHash<T>>::get(&ca_hash);
            let mut hashes = <CAHashesByAccount<T>>::get(&sender);
            let mut certs = Self::cert_hashes();
            // a
            ensure!(<IndexByCAHash<T>>::exists(ca_hash), "Provided CAHash doesn't exist.");
            // b
            ensure!(!accounts.contains(&sender), "Provided account is already registered for this CAHash.");
            // c
            ensure!(!hashes.contains(&ca_hash), "CAHash is already resigtered for this account");
            // d 
            ensure!(!<CertificateStore<T>>::exists((&sender, &ca_hash)), "This account for this specific CAHash already has Certification.");
            // e
            ensure!(!certs.contains(&cert), "Your cert is already used to register another account.");

            let certificate = Certification {
                cert,
                signature,
            };
            
            accounts.push(sender.clone());
            <AccountsByCAHash<T>>::remove(&ca_hash);
            <AccountsByCAHash<T>>::insert(&ca_hash, accounts);

            hashes.push(ca_hash.clone());
            <CAHashesByAccount<T>>::remove(&sender);
            <CAHashesByAccount<T>>::insert(&sender, hashes);

            certs.push(cert);
            <CertHashes<T>>::put(certs);

            <CertificateStore<T>>::insert((&sender, ca_hash), certificate);
            print("Account successfully registered!");
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
    pub fn check_account(sender:T::AccountId, cahash:T::Hash) -> Result {
        // account should be registered for the provided CAHash
        let accounts = Self::accounts_by_cahash(cahash);
        ensure!(accounts.contains(&sender), "Your account is not registered for this CAHash.");
        Ok(())
    }   
}


#[cfg(test)]
mod tests {
    use super::*;
    // use crate::rsa::*;
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
    // use rustc_hex::{FromHex, ToHex};
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

    type Certificate = Module<Test>;
    
    //public modulus
    #[test]
    fn can_register_ca() {
      TestExternalities::default().execute_with(||{
        let CAHash = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
        let data = [11, 12, 13, 14].to_vec();

        assert_eq!(Certificate::cahash_count(), 0);

        // register new ca
        assert_ok!(Certificate::register_ca(Origin::signed(1), CAHash, data.clone()));
        
        // respective storage changes
        assert_eq!(Certificate::cahash_count(), 1);
        assert_eq!(Certificate::cahash_by_index(1), CAHash);
        assert_eq!(Certificate::index_by_cahash(CAHash), 1);

        // cannot register same CAhash
        assert_noop!(Certificate::register_ca(Origin::signed(1), CAHash, data), "Provided CAHash is already registered.");
      });
    }

    #[test]
    fn can_register_account() {
      TestExternalities::default().execute_with(||{
        let CAHash = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
        let cert = sr_primitives::traits::BlakeTwo256::hash(&[221, 222, 223, 224]);
        let signature = sr_primitives::traits::BlakeTwo256::hash(&[11, 12, 13, 14]);
        let certificate = Certification {
          cert,
          signature,
        };
        let data = [11, 12, 13, 14].to_vec();

        // cannot register for non-existing CA
        assert_noop!(Certificate::register_account(Origin::signed(1), CAHash, cert, signature), "Provided CAHash doesn't exist.");
        assert_ok!(Certificate::register_ca(Origin::signed(1), CAHash, data));
        assert_ok!(Certificate::register_account(Origin::signed(1), CAHash, cert, signature));

        // respective storage changes
        assert_eq!(Certificate::accounts_by_cahash(CAHash).len(), 1);
        assert_eq!(Certificate::cahashes_by_account(1).len(), 1);
        assert_eq!(Certificate::certificate_store((1, CAHash)), certificate);

        // cannot register account with the same hash twice
        assert_noop!(Certificate::register_account(Origin::signed(1), CAHash, cert, signature), "Provided account is already registered for this CAHash.");
        // cannot use same certHash twice
        assert_noop!(Certificate::register_account(Origin::signed(2), CAHash, cert, signature), "Your cert is already used to register another account.");
        
      });
    }
    // fn can_register() {
    //     TestExternalities::default().execute_with(||{
    //         // register account
    //         // let pubkey: Vec<u8> = pub_str.from_hex().unwrap();
    //         let pubkey: Vec<u8> = [11, 22, 33, 44].to_vec();
    //         // let cert: H256 = sr_primitives::traits::Hash::hash(&[444, 555, 66, 777]);
    //         let cert = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
    //         let signed_account = sr_primitives::traits::BlakeTwo256::hash(&[122, 122, 122, 122]);

    //         assert_ok!(Certificate::register_account(Origin::signed(1), pubkey.clone(), cert, signed_account));

    //         let certification = Certification {
    //             pubkey,
    //             cert,
    //             signature: signed_account,
    //         };
            
    //         assert_eq!(<AccountStore<Test>>::get(1), certification);
    //     });
    // }
}