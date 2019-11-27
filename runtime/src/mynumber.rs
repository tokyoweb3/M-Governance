/// Runtime module implementing MyNumber authentication functions
/// with register, update, verify, revoke functions.
/// With this runtime module, anyone with official mynumber card provided by Japan Agency for Local Authority
/// can prove his/her key pair is legit and tied to the digital certificate of Mynumber card.
/// This module works together with governance module, to permit users to vote only one time, allowing the implementation of more secure voting method.

use support::{dispatch::Result, Parameter, decl_storage, decl_module, decl_event, ensure};
use system::{ensure_signed};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Registered(AccountId, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as MyNumber {

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // register

        // verify

        // update

        // revoke
    }
}