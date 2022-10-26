// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Migrate Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_std::prelude::*;
use sp_core::{H160, H256, U256};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub trait UniquesConverter<CollectionId, ItemId> {
		fn collection(i: H160) -> CollectionId;
		fn item(i: H256) -> ItemId;
	}

	impl<CollectionId: From<H160>, ItemId: From<H256>> UniquesConverter<CollectionId, ItemId> for () {
		fn collection(i: H160) -> CollectionId {
			i.into()
		}
		fn item(i: H256) -> ItemId {
			i.into()
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config + pallet_evm::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Converter: UniquesConverter<Self::CollectionId, Self::ItemId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// EvmAddress.
		EvmAddress { who: H160 },
	}

	/// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// NotOwner.
		NotOwner,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn migrate(
			origin: OriginFor<T>,
			contract: H160,
			owner_raw_key: H256,
			starting_raw_key: H256,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let h160_origin = Self::get_evm_address(&origin);

			// Creating a Collection
			let v = pallet_evm::AccountStorages::<T>::get(contract.clone(), owner_raw_key);
			let contract_owner = H160::from(v.clone());

			if h160_origin == contract_owner {
				// let owner: T::AccountId = T::AddressMapping::into_account_id(contract_owner.clone());
				let converted_contract = T::Converter::collection(contract.clone());

				if pallet_uniques::Pallet::<T>::collection_owner(converted_contract.clone()).is_none() {
					let _ = pallet_uniques::Pallet::<T>::do_create_collection(
						converted_contract.clone(),
						origin.clone(),
						origin.clone(),
						T::CollectionDeposit::get(),
						false,
						pallet_uniques::Event::Created {
							collection: converted_contract.clone(),
							creator: origin.clone(),
							owner: origin.clone()
						},
					);
				};
			}

			let mut u256_counter = U256::from_big_endian(&starting_raw_key[..]);
			let mut h256_counter = H256::default();
			loop {
				u256_counter.to_big_endian(&mut h256_counter[..]);
				let item_id = pallet_evm::AccountStorages::<T>::get(&contract, &h256_counter);

				u256_counter += U256::one();
				u256_counter.to_big_endian(&mut h256_counter[..]);
				let item_owner = pallet_evm::AccountStorages::<T>::get(&contract, &h256_counter);

				u256_counter += U256::one();
				if item_id == H256::default() && item_owner == H256::default() { break };
				// let owner = T::AddressMapping::into_account_id(H160::from(item_owner.clone()));
				if h160_origin == H160::from(item_owner.clone()) {
					let collection = T::Converter::collection(contract.clone());
					let item = T::Converter::item(item_id.clone());
					if pallet_uniques::Pallet::<T>::owner(collection.clone(), item.clone()).is_none() {
						let _ = pallet_uniques::Pallet::<T>::do_mint(
							collection,
							item,
							origin.clone(),
							|_| { Ok(()) }
						);
					}
				}
			}
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn evm_account(origin: OriginFor<T>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let who = Self::get_evm_address(&origin);
			Self::deposit_event(Event::<T>::EvmAddress { who });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_evm_address(account_id: &T::AccountId) -> H160 {
			H160::from_slice(&account_id.encode()[0..20])
		}
	}

}
