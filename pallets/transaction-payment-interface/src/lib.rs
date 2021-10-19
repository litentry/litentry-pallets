#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    #[pallet::config]
	pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::type_value]
	pub fn RatioOnEmpty() -> (u32, u32, u32) {
		(50, 30, 1)
	}

    #[pallet::storage]
	#[pallet::getter(fn get_ratio)]
	pub type Ratio<T: Config> =
		StorageValue<_, (u32, u32, u32), ValueQuery, RatioOnEmpty>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RatioChanged(u32, u32, u32),
    }

    #[pallet::error]
	pub enum Error<T> {
        RatioOverflow,
    }

    
    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10)]
        pub fn set_ratio(
            origin: OriginFor<T>,
            treasury_ratio: u32,
            author_ratio: u32,
            burned_ratio: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            if treasury_ratio + author_ratio + burned_ratio > 0 {
                <Ratio<T>>::put((treasury_ratio, author_ratio, burned_ratio));
                Self::deposit_event(Event::<T>::RatioChanged(treasury_ratio, author_ratio, burned_ratio));
                Ok(())
            } else {
                Err(Error::<T>::RatioOverflow.into())
            }
        }

    }

}


/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
	R: pallet_balances::Config + pallet_authorship::Config,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let numeric_amount = amount.peek();
		let author = <pallet_authorship::Pallet<R>>::author();
		<pallet_balances::Pallet<R>>::resolve_creating(
			&<pallet_authorship::Pallet<R>>::author(),
			amount,
		);
		<frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit(
			author,
			numeric_amount,
		));
	}
}


pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: Config + pallet_balances::Config + pallet_treasury::Config + pallet_authorship::Config,
	pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, (1) to treasury, (2) to author and (3) burned
            let ratio = Pallet::<R>::get_ratio();
            let (unburned, _) = fees.ration(ratio.0 + ratio.1, ratio.2);
			let mut split = unburned.ration(ratio.0, ratio.1);
            
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut split.1);
			}
			use pallet_treasury::Pallet as Treasury;
			<Treasury<R> as OnUnbalanced<_>>::on_unbalanced(split.0);
			<ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
		}
	}
}