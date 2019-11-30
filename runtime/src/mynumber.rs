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
        pub fn register_account(origin, pubkey: Pubkey, encrypted_account: T::Hash, cert: T::Hash) -> Result{
            ensure!(pubkey.len() = 256, "RSA public key should be 256 bytes");

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

        // verify
        pub fn get_account(origin) -> Result {
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
            let pub_str: String = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEArtIADvVx7dIsMwZe60Fgb3cB2wQXo372wXdafJlZnA0ZwhCd9CxrRmlYCvq86c6hYtF/nBDjjNBvvzGPgain+KwZkiM5mHOKV7phACEvt6pGCjTza4HPEQ+VX+Lk+T5WvkXvSQ4WMa0cQ6GA4A39TG/2d2P7U9bhm/DW2A0bd8gUVb7O171zcX49cq1bHidUNu3rBKqcTQESvaQCNKtiHtqK9WBrkcAtl3w0vBNEcyRM9cVRpzQCZF6N+H3KToXccroIbgJvjcx5rZsaZZrMFJsLY7eFbWU3wJ7RDT8+LhEtbHRelyBU0XhGekqIELibOWEmIa8qcNFjf1Zz/yLNHQIDAQAB".as_bytes().to_hex();
            // let pubkey: Vec<u8> = b"0xAED2000EF571EDD22C33065EEB41606F7701DB0417A37EF6C1775A7C99599C0D19C2109DF42C6B4669580AFABCE9CEA162D17F9C10E38CD06FBF318F81A8A7F8AC1992233998738A57BA6100212FB7AA460A34F36B81CF110F955FE2E4F93E56BE45EF490E1631AD1C43A180E00DFD4C6FF67763FB53D6E19BF0D6D80D1B77C81455BECED7BD73717E3D72AD5B1E275436EDEB04AA9C4D0112BDA40234AB621EDA8AF5606B91C02D977C34BC134473244CF5C551A73402645E8DF87DCA4E85DC72BA086E026F8DCC79AD9B1A659ACC149B0B63B7856D6537C09ED10D3F3E2E112D6C745E972054D178467A4A8810B89B39612621AF2A70D1637F5673FF22CD1D".to_vec();
            let pubkey: Vec<u8> = pub_str.from_hex().unwrap();
            let data: Vec<u8> = [22, 33, 111, 112].to_vec();
            assert_eq!(data, pubkey);
            // let cert: H256 = H256::from_slice(b"0x290cd4dc09c152b574df6e1834335d5ddb5326e1450c2501b471aa6980ed7065");
            // let signed_account: H256 = H256::from_slice(b"0x7a1d6c873fb9d4114e9806b4c4ad3a49d2146cbe31a87b0d15c1f38ebbfee93c");
        
            // let cert: H256 = sr_primitives::traits::Hash::hash(&[444, 555, 66, 777]);
            let cert = sr_primitives::traits::BlakeTwo256::hash(&[111, 112, 113, 114]);
            let signed_account = sr_primitives::traits::BlakeTwo256::hash(&[122, 122, 122, 122]);

            assert_ok!(MyNumber::register_account(Origin::signed(1), pubkey, cert, signed_account));

            // let certification = Certification {
            //     pubkey,
            //     cer,
            //     encrypted_account: signed_account,
            // };
            
            // assert_eq!(<AccountStore<Test>>::get(1), certification);
        });
    }

    // const PUBLIC_KEY_2048 = "551322645510721241055120100471111112304112510522620103551322653111502426455111204445111022344055215034265547712166455310120430167101214425211012064750111520234121103204445111012064554716220242521104202445646736053166115346445446631255167123164306010230631542130242734626215631557141310541671323304514613214040556144206641671623604147113133066522703427310315015461466572147213115221041156166204471521202167516615632260547167226254121153226553211722066465160156531131052064055016015472166124324415071101506057253242425201141265412665250255531303267312764242515666030246527107154505436320232101107250050716231031057712523352057332641421011445350470310610601422545254767142474671303067506116130634464151220610611652345254411320471063121250615611413545150511323441521110024645647122670564153344411271543504054316614073463143212471027123251171160244531431322065057253234331061242263151014330654157142222675621523544514716215274143132302715321223271132114346451061063123353163252535421221564516753160521041642126411414524444142125204745541073205406011134265545142322461051552125351716116060511152214471431722643054611623074457111242440121012424050455132051021051322645512010042116111230411251052262010355132265312426455";
    // #[test]
    // fn hex_to_vec(){
    //     TestExternalities::default().execute_with(||{

            
    //     });
    // }
}