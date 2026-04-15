use frame_support::{
    pallet_prelude::*,
    traits::{fungible::Credit, tokens::imbalance::ResolveTo, OnUnbalanced},
};
use frame_system::pallet_prelude::AccountIdFor;
use sp_runtime::traits::AccountIdConversion;


/// the Treasury pallet's account ID (derive from its PalletId)
pub struct TreasuryAccount<T>(sp_core::sp_std::marker::PhantomData<T>);

impl<T: pallet_treasury::Config> TypedGet for TreasuryAccount<T> {
    type Type = AccountIdFor<T>;

    fn get() -> AccountIdFor<T> {
        <T as pallet_treasury::Config>::PalletId::get().into_account_truncating()
    }
}

/// Handles transaction fees by sending them to the Treasury
pub struct DealWithFees<R>(sp_core::sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>> for DealWithFees<R>
where
    R: pallet_balances::Config + pallet_treasury::Config,
{
    fn on_unbalanceds(
        mut fees_then_tips: impl Iterator<Item = Credit<R::AccountId, pallet_balances::Pallet<R>>>,
    ) where
        Credit<R::AccountId, pallet_balances::Pallet<R>>:
            frame_support::traits::tokens::imbalance::TryMerge,
    {
        // the transaction fee
        if let Some(fees) = fees_then_tips.next() {
            // send 100% of fees to Treasury
            ResolveTo::<TreasuryAccount<R>, pallet_balances::Pallet<R>>::on_unbalanced(fees);
        }

        // transaction tip ( if it was provided)
        if let Some(tip) = fees_then_tips.next() {
            // send 100% of tip to Treasury
            // will be modifed to send tips to the validators
            ResolveTo::<TreasuryAccount<R>, pallet_balances::Pallet<R>>::on_unbalanced(tip);
        }
    }
}
