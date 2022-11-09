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
use sp_runtime::traits::{AccountIdConversion, Zero};
use sha3::{Digest, Keccak256};
use hex;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		PalletId,
		traits::Currency,
	};
	use frame_system::pallet_prelude::*;
	use pallet_evm::{AddressMapping, Runner};

	type BalanceOf<T> =
		<<T as pallet_uniques::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		/// The NFT Migrator's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Converter: UniquesConverter<Self::CollectionId, Self::ItemId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// EvmAddress.
		EvmAddress { who: H160 },
		/// AccountId.
		AccountId { who: T::AccountId },
	}

	/// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// Fail.
		Fail,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn migrate_erc721_nfts(
			origin: OriginFor<T>,
			contract: H160,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			// Creating a Collection if it does not exist (setting this Pallet account as its owner).
			let master = Self::account_id();
			Self::create_collection(master.clone(), master.clone(), contract.clone());

			// For simple ERC721 the `mapping(uint256 => address)` is at storage slot 2.
			// So if we keccak(0x00...02) we'll get the first storage key (starting_key):
			let u256_slot_2 = U256::from(2);
			let mut h256_slot_2 = H256::default();
			u256_slot_2.to_big_endian(&mut h256_slot_2[..]);
			let starting_key = H256::from_slice(Keccak256::digest(&h256_slot_2[..]).as_slice());

			Self::migrate_nfts_from_starting_key(
				vec![master],
				contract,
				starting_key,
				true
			);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn claim_erc721_items(
			origin: OriginFor<T>,
			contract: H160,
			item_ids: Vec<H256>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			// Creating a Collection if it does not exist (setting this Pallet account as its owner).
			let master = Self::account_id();
			Self::create_collection(master.clone(), master.clone(), contract.clone());

			let collection = T::Converter::collection(contract.clone());

			let account_id_from_evm_address = T::AddressMapping::into_account_id(Self::get_evm_address(&origin));
			for item_id in item_ids {
				let item = T::Converter::item(item_id);
				if let Some(current_owner) = pallet_uniques::Pallet::<T>::owner(collection.clone(), item.clone()) {
					if current_owner == account_id_from_evm_address {
						Self::migrate_token_to_owner(origin.clone(), contract.clone(), item_id);
					}
				}
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn migrate_with_owner_of(
			origin: OriginFor<T>,
			contract: H160,
			token_ids: Vec<H256>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			// Creating a Collection if it does not exist (setting this Pallet account as its owner).
			let master = Self::account_id();
			Self::create_collection(master.clone(), master.clone(), contract.clone());

			for token_id in token_ids {
				if Self::is_owner_of(&origin, contract.clone(), token_id.clone()) {
					Self::migrate_token_to_owner(origin.clone(), contract.clone(), token_id);
				}
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn z_migrate_my_nfts(
			origin: OriginFor<T>,
			contract: H160,
			storage_slot: H256,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let starting_key = Self::get_starting_key(&origin, storage_slot.clone());
			Self::migrate_nfts_from_starting_key(vec![origin], contract, starting_key, false);
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn z_evm_address(origin: OriginFor<T>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let who = Self::get_evm_address(&origin);
			Self::deposit_event(Event::<T>::EvmAddress { who });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn z_evm_to_account_id(origin: OriginFor<T>, address: H160) -> DispatchResult {
			ensure_signed(origin)?;
			let who = T::AddressMapping::into_account_id(address);
			Self::deposit_event(Event::<T>::AccountId { who });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {

		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn get_evm_address(account_id: &T::AccountId) -> H160 {
			H160::from_slice(&account_id.encode()[0..20])
		}

		fn get_starting_key(origin: &T::AccountId, storage_slot: H256) -> H256 {
			let h256_address = H256::from(Self::get_evm_address(origin));
			let data: Vec<u8> = [&h256_address[..], &storage_slot[..]].concat();
			let hash = H256::from_slice(Keccak256::digest(&data).as_slice());
			H256::from_slice(Keccak256::digest(&hash[..]).as_slice())
		}

		fn create_collection(owner: T::AccountId, admin: T::AccountId, contract: H160) {
			let converted_contract = T::Converter::collection(contract.clone());
			if pallet_uniques::Pallet::<T>::collection_owner(converted_contract.clone()).is_none() {
				let _ = pallet_uniques::Pallet::<T>::do_create_collection(
					converted_contract.clone(),
					owner.clone(),
					admin.clone(),
					BalanceOf::<T>::zero(),
					true,
					pallet_uniques::Event::Created {
						collection: converted_contract,
						creator: owner.clone(),
						owner: owner.clone()
					},
				);
			};
		}

		fn migrate_nfts_from_starting_key(
			owners: Vec<T::AccountId>,
			contract: H160,
			starting_key: H256,
			check_for_owner: bool,
		) {
			let mut u256_counter = U256::from_big_endian(&starting_key[..]);
			let mut h256_counter = H256::default();
			loop {
				let mut owner = owners[0].clone();

				u256_counter.to_big_endian(&mut h256_counter[..]);
				let token_id = pallet_evm::AccountStorages::<T>::get(&contract, &h256_counter);

				let mut token_owner = H256::default();
				if check_for_owner {
					u256_counter += U256::one();
					u256_counter.to_big_endian(&mut h256_counter[..]);
					token_owner = pallet_evm::AccountStorages::<T>::get(&contract, &h256_counter);
					// We use pallet_evm AddressMapping to get an AccountId from EVM Account Address (H160):
					owner = T::AddressMapping::into_account_id(H160::from(token_owner.clone()));
					// for _owner in &owners {
					// 	if Self::get_evm_address(_owner) == H160::from(token_owner.clone()) {
					// 		owner = _owner.clone();
					// 	}
					// }
				}

				if token_id == H256::default() && token_owner == H256::default() { break };
				u256_counter += U256::one();

				Self::migrate_token_to_owner(owner, contract, token_id);
			}
		}

		fn migrate_token_to_owner(owner: T::AccountId, contract: H160, token_id: H256) {
			let collection = T::Converter::collection(contract.clone());
			let item = T::Converter::item(token_id);
			let current_owner = pallet_uniques::Pallet::<T>::owner(collection.clone(), item.clone());
			if current_owner.is_none() {
				let _ = pallet_uniques::Pallet::<T>::do_mint(
					collection,
					item,
					owner.clone(),
					|_| { Ok(()) }
				);
			} else if current_owner.unwrap() != owner.clone() {
				let _z = pallet_uniques::Pallet::<T>::do_transfer(
					collection,
					item,
					owner.clone(),
					|_, _| { Ok(()) }
				);
			}
		}

		fn is_owner_of(
			owner: &T::AccountId,
			contract: H160,
			token_id: H256,
		) -> bool {
			let source = Self::get_evm_address(owner);

			let owner_of_hash = hex::decode("6352211e").unwrap();
			let input = [owner_of_hash.as_slice(), &token_id.to_fixed_bytes()].concat();

			return match T::Runner::call(
				source,
				contract,
				input,
				U256::default(),
				T::BlockGasLimit::get().as_u64(),
				None,
				None,
				None,
				vec![],
				false,
				false,
				T::config(),
			) {
				Ok(info) => {
					if info.value.len() == 32 && source.as_bytes() == &info.value[12..] {
						return true;
					}
					false
				},
				Err(_) => false,
			}
		}

	}

}
