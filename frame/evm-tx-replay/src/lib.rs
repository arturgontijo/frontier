#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
mod data;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use ethereum::{LegacyTransaction, TransactionAction, TransactionSignature, TransactionV2};
    use fp_ethereum::ValidatedTransaction;
    use fp_evm::CallOrCreateInfo;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};
    use frame_system::pallet_prelude::*;
    use pallet_evm::{AddressMapping, GasWeightMapping};
    use sp_core::{H160, H256, U256};
    use sp_runtime::traits::UniqueSaturatedInto;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config + pallet_ethereum::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
    }

    /// Authority allowed to send replay_tx extrinsics.
    #[pallet::storage]
    #[pallet::getter(fn authority)]
    pub type Authority<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    type ExecutionIndex = u64;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The transaction was successfully replayed.
        TransactionReplayed(ExecutionIndex),
        /// A new authority was set
        AuthoritySet(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Invalid signature
        InvalidSignature,
        /// The transaction failed to replay.
        TransactionReplayFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({
            let without_base_extrinsic_weight = true;
            <T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
                gas_limit.as_u64().unique_saturated_into()
            }, without_base_extrinsic_weight)
        })]
        pub fn replay_tx(
            origin: OriginFor<T>,
            execution_index: ExecutionIndex,
            from: H160,
            nonce: U256,
            gas_price: U256,
            gas_limit: U256,
            gas_used: U256,
            to: Option<H160>,
            value: U256,
            data: sp_std::vec::Vec<u8>,
            v: u64,
            r: H256,
            s: H256,
        ) -> DispatchResultWithPostInfo {
            // Defined the same as extrinsic base weight


            let mut weight = Weight::from_parts(1_000_u64.saturating_mul(125_000), 0);

            // Note: extrinsic base weight already accounts for signature verification.
            let who = ensure_signed(origin)?;

            weight.saturating_accrue(<T as Config>::WeightInfo::is_authority());
            // if !Self::is_authority(who) {
            //     return Err(frame_support::sp_runtime::DispatchError::BadOrigin.into());
            // }

            let tx_signature =
                TransactionSignature::new(v, r, s).ok_or(Error::<T>::InvalidSignature)?;
            let tx = TransactionV2::Legacy(LegacyTransaction {
                nonce,
                gas_price,
                gas_limit,
                action: match to {
                    Some(to) => TransactionAction::Call(to),
                    None => TransactionAction::Create,
                },
                value,
                input: data,
                signature: tx_signature,
            });

            let from_id = T::AddressMapping::into_account_id(from);

            // [Artur]: Pre funding `from` with gasLimit+value amount, so the apply() does not break.
            let prefund = gas_limit
                .checked_mul(gas_price)
                .ok_or(Error::<T>::TransactionReplayFailed)?;
            let prefund = prefund
                .checked_add(value)
                .ok_or(Error::<T>::TransactionReplayFailed)?;
            T::Currency::deposit_creating(&from_id, prefund.as_u128().unique_saturated_into());

            // consume weight for TransactionSignature::new
            weight.saturating_accrue(<T as Config>::WeightInfo::tx_creation());
            match pallet_ethereum::ValidatedTransaction::<T>::apply(from, tx.clone()) {
                Ok((_, call_or_create_info)) => {

                    let used_gas = match call_or_create_info {
                        CallOrCreateInfo::Call(info) => info.used_gas.effective,
                        CallOrCreateInfo::Create(info) => info.used_gas.effective,
                    };

                    // [Artur]: `from` was already charged the `used_gas` in apply().
                    // [Artur]: So we withdraw `gas_limit - used_gas` from it.

                    // [Artur]: Example:

                    // [Artur]: initialBalance              :   1_000_000
                    // [Artur]: -gasUsed                    :      60_000
                    // [Artur]: finalBalance                :     940_000*

                    // [Artur]: initialBalance              :   1_000_000
                    // [Artur]: +gasLimit (100_000)         :     100_000
                    // [Artur]: -usedGas (90_000)           :   1_010_000
                    // [Artur]: -extra (gasUsed+gl-usedGas) :     940_000*

                    // finalBal = initBal + gL - uG - [extra]
                    // finalBal = initBal + gL - uG - [gU + (gL - uG)]
                    // 940_000 = 1_000_000 + 100_000 - 90_000 - [60_000 + (100_000 - 90_000)]
                    // 940_000 = 1_000_000 + 100_000 - 90_000 - [70_000]
                    // 940_000 = 940_000

                    let extra = gas_used
                        .checked_add(gas_limit)
                        .ok_or(Error::<T>::TransactionReplayFailed)?
                        .checked_sub(used_gas)
                        .ok_or(Error::<T>::TransactionReplayFailed)?
                        .checked_mul(gas_price)
                        .ok_or(Error::<T>::TransactionReplayFailed)?
                        .checked_add(value)
                        .ok_or(Error::<T>::TransactionReplayFailed)?
                        .as_u128();
                    T::Currency::withdraw(
                        &from_id,
                        extra.unique_saturated_into(),
                        WithdrawReasons::FEE,
                        ExistenceRequirement::AllowDeath
                    )?;
                },
                Err(_) => return Err(Error::<T>::TransactionReplayFailed.into()),
            }
            Ok(Some(weight).into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::set_authority())]
        pub fn set_authority(origin: OriginFor<T>, new_authority: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            <Authority<T>>::put(new_authority.clone());
            Self::deposit_event(Event::<T>::AuthoritySet(new_authority));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn is_authority(account: T::AccountId) -> bool {
            Self::authority() == Some(account)
        }
    }
}
