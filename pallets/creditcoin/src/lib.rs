#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![recursion_limit = "256"]

#[macro_export]
macro_rules! warn_or_panic {
	($($arg:tt)*) => {
		log::warn!($($arg)*);
		if cfg!(feature = "try-runtime") {
			panic!()
		}
	};
}

extern crate alloc;

use frame_support::traits::StorageVersion;
pub use pallet::*;
use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[allow(clippy::unnecessary_cast)]
pub mod weights;

mod benchmarking;
#[cfg(test)]
mod tests;

#[macro_use]
mod helpers;
pub mod migrations;
pub mod ocw;
mod types;

use ocw::tasks::collect_coins::GCreContract;
pub use types::{
	loan_terms, Address, AddressId, AskOrder, AskOrderId, AskTerms, BidOrder, BidOrderId, BidTerms,
	Blockchain, CollectedCoinsId, CollectedCoinsStruct, DealOrder, DealOrderId, Duration,
	ExternalAddress, ExternalAmount, ExternalTxId, Guid, InterestRate, InterestType, LegacySighash,
	LoanTerms, Offer, OfferId, OrderId, RatePerPeriod, Task, TaskId, TaskOutput, Transfer,
	TransferId, TransferKind, UnverifiedCollectedCoins, UnverifiedTransfer,
};

pub(crate) use types::{DoubleMapExt, Id};

#[cfg(test)]
pub(crate) use types::test;

pub type BalanceFor<T> = <T as pallet_balances::Config>::Balance;

pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use crate::{helpers::non_paying_error, types::OwnershipProof};
	use frame_support::{
		dispatch::{DispatchResult, PostDispatchInfo},
		fail,
		pallet_prelude::*,
		traits::tokens::{currency::Currency as CurrencyT, fungible::Mutate, ExistenceRequirement},
		transactional,
	};
	use frame_system::{ensure_signed, offchain::CreateSignedTransaction, pallet_prelude::*};
	use ocw::errors::VerificationFailureCause;
	use pallet_offchain_task_scheduler::authority::AuthorityController;
	use pallet_offchain_task_scheduler::tasks::{TaskScheduler, TaskV2};
	use sp_runtime::traits::{IdentifyAccount, UniqueSaturatedFrom, UniqueSaturatedInto, Verify};
	use tracing as log;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_balances::Config
		+ pallet_timestamp::Config
		+ CreateSignedTransaction<Call<Self>>
	where
		<Self as frame_system::Config>::BlockNumber: UniqueSaturatedInto<u64>,
		<Self as pallet_timestamp::Config>::Moment:
			UniqueSaturatedInto<u64> + UniqueSaturatedFrom<u64>,
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Call: From<Call<Self>>;

		type Signer: From<sp_core::ecdsa::Public>
			+ IdentifyAccount<AccountId = <Self as frame_system::Config>::AccountId>
			+ Parameter;

		type SignerSignature: Verify<Signer = Self::Signer>
			+ From<sp_core::ecdsa::Signature>
			+ Parameter;

		// in order to turn a `Hash` into a U256 for checking the nonces on
		// ethless transfers we need the `Hash` type to implement
		// the BigEndianHash trait. This effectively constrains the Hash
		// type to H256, which sort of defeats the purpose of it being an associated type.
		// However a lot of code refers to Config::Hash, so right now this is the least invasive way
		// to get the compiler to let us do the Config::Hash -> U256 conversion
		type HashIntoNonce: IsType<<Self as frame_system::Config>::Hash>
			+ ethereum_types::BigEndianHash<Uint = sp_core::U256>
			+ Clone;

		type UnverifiedTaskTimeout: Get<<Self as frame_system::Config>::BlockNumber>;

		type WeightInfo: WeightInfo;

		type TaskScheduler: TaskScheduler<
				BlockNumber = Self::BlockNumber,
				Hash = Self::Hash,
				Task = Task<Self::AccountId, Self::BlockNumber, Self::Hash, Self::Moment>,
			> + AuthorityController<AccountId = Self::AccountId>;
	}

	pub trait WeightInfo {
		fn migration_v6(t: u32) -> Weight;
		fn migration_v7(t: u32) -> Weight;
		fn on_initialize(a: u32, b: u32, o: u32, d: u32, f: u32) -> Weight;
		fn register_address() -> Weight;
		fn claim_legacy_wallet() -> Weight;
		fn add_ask_order() -> Weight;
		fn add_bid_order() -> Weight;
		fn add_offer() -> Weight;
		fn add_deal_order() -> Weight;
		fn add_authority() -> Weight;
		fn persist_transfer() -> Weight;
		fn fail_transfer() -> Weight;
		fn fund_deal_order() -> Weight;
		fn lock_deal_order() -> Weight;
		fn register_funding_transfer() -> Weight;
		fn register_repayment_transfer() -> Weight;
		fn close_deal_order() -> Weight;
		fn exempt() -> Weight;
		fn register_deal_order() -> Weight;
		fn request_collect_coins() -> Weight;
		fn persist_collect_coins() -> Weight;
		fn fail_collect_coins() -> Weight;
		fn remove_authority() -> Weight;
		fn set_collect_coins_contract() -> Weight;
		fn register_address_v2() -> Weight;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type LegacyWallets<T: Config> = StorageMap<_, Twox128, LegacySighash, T::Balance>;

	#[pallet::storage]
	pub type LegacyBalanceKeeper<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn deal_orders)]
	pub type DealOrders<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Identity,
		T::Hash,
		DealOrder<T::AccountId, T::BlockNumber, T::Hash, T::Moment>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn addresses)]
	pub type Addresses<T: Config> =
		StorageMap<_, Blake2_128Concat, AddressId<T::Hash>, Address<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn used_guids)]
	pub type UsedGuids<T: Config> = StorageMap<_, Blake2_128Concat, Guid, ()>;

	#[pallet::storage]
	#[pallet::getter(fn ask_orders)]
	pub type AskOrders<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Identity,
		T::Hash,
		AskOrder<T::AccountId, T::BlockNumber, T::Hash>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn bid_orders)]
	pub type BidOrders<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Identity,
		T::Hash,
		BidOrder<T::AccountId, T::BlockNumber, T::Hash>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Identity,
		T::Hash,
		Offer<T::AccountId, T::BlockNumber, T::Hash>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn transfers)]
	pub type Transfers<T: Config> = StorageMap<
		_,
		Identity,
		TransferId<T::Hash>,
		Transfer<T::AccountId, T::BlockNumber, T::Hash, T::Moment>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collected_coins)]
	pub type CollectedCoins<T: Config> = StorageMap<
		_,
		Identity,
		CollectedCoinsId<T::Hash>,
		types::CollectedCoinsStruct<T::Hash, T::Balance>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collect_coins_contract)]
	pub type CollectCoinsContract<T: Config> = StorageValue<_, GCreContract, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An address on an external chain has been registered.
		/// [registered_address_id, registered_address]
		AddressRegistered(AddressId<T::Hash>, Address<T::AccountId>),

		/// Collecting coins from Eth ERC-20 has been registered and will be verified.
		/// [collected_coins_id, registered_collect_coins]
		CollectCoinsRegistered(CollectedCoinsId<T::Hash>, types::UnverifiedCollectedCoins),

		/// An external transfer has been registered and will be verified.
		/// [registered_transfer_id, registered_transfer]
		TransferRegistered(
			TransferId<T::Hash>,
			Transfer<T::AccountId, T::BlockNumber, T::Hash, T::Moment>,
		),

		/// An external transfer has been successfully verified.
		/// [verified_transfer_id]
		TransferVerified(TransferId<T::Hash>),

		/// CollectCoins has been successfully verified and minted.
		/// [collected_coins_id, collected_coins]
		CollectedCoinsMinted(
			types::CollectedCoinsId<T::Hash>,
			types::CollectedCoinsStruct<T::Hash, T::Balance>,
		),

		/// An external transfer has been processed and marked as part of a loan.
		/// [processed_transfer_id]
		TransferProcessed(TransferId<T::Hash>),

		/// An ask order has been added by a prospective lender. This indicates that the lender
		/// is looking to issue a loan with certain terms.
		/// [ask_order_id, ask_order]
		AskOrderAdded(
			AskOrderId<T::BlockNumber, T::Hash>,
			AskOrder<T::AccountId, T::BlockNumber, T::Hash>,
		),

		/// A bid order has been added by a prospective borrower. This indicates that the borrower
		/// is looking for a loan with certain terms.
		/// [bid_order_id, bid_order]
		BidOrderAdded(
			BidOrderId<T::BlockNumber, T::Hash>,
			BidOrder<T::AccountId, T::BlockNumber, T::Hash>,
		),

		/// An offer has been added by a lender. This indicates that the lender
		/// is interested in entering a loan with the owner of the bid order.
		/// [offer_id, offer]
		OfferAdded(OfferId<T::BlockNumber, T::Hash>, Offer<T::AccountId, T::BlockNumber, T::Hash>),

		/// A deal order has been added by a borrower. This indicates that the borrower
		/// has accepted a lender's offer and intends to enter the loan.
		/// [deal_order_id, deal_order]
		DealOrderAdded(
			DealOrderId<T::BlockNumber, T::Hash>,
			DealOrder<T::AccountId, T::BlockNumber, T::Hash, T::Moment>,
		),

		/// A deal order has been funded by a lender. This indicates that the lender
		/// has initiated the actual loan by transferring the loan amount to the borrower
		/// on an external chain.
		/// [funded_deal_order_id]
		DealOrderFunded(DealOrderId<T::BlockNumber, T::Hash>),

		/// A deal order has been locked by a borrower. This indicates that the borrower
		/// is preparing to make a repayment and locks the loan from being sold or transferred
		/// to another party.
		/// [deal_order_id]
		DealOrderLocked(DealOrderId<T::BlockNumber, T::Hash>),

		/// A deal order has been closed by a borrower. This indicates that the borrower
		/// has repaid the loan in full and is now closing out the loan.
		/// [closed_deal_order_id]
		DealOrderClosed(DealOrderId<T::BlockNumber, T::Hash>),

		/// A loan exemption has been granted by a lender. This indicates that the lender
		/// is releasing all of the outstanding debt on the loan. The borrower
		/// is no longer responsible for repaying the amount.
		/// [exempted_deal_order_id]
		LoanExempted(DealOrderId<T::BlockNumber, T::Hash>),

		/// A legacy wallet from Creditcoin 1.X has been claimed. The balance of the legacy wallet
		/// has been transferred to the owner's Creditcoin 2.0 account.
		/// [legacy_wallet_claimer, legacy_wallet_sighash, legacy_wallet_balance]
		LegacyWalletClaimed(T::AccountId, LegacySighash, T::Balance),

		TransferFailedVerification(TransferId<T::Hash>, VerificationFailureCause),

		/// exchanging vested ERC-20 CC for native CC failed.
		/// [collected_coins_id, cause]
		CollectCoinsFailedVerification(CollectedCoinsId<T::Hash>, VerificationFailureCause),
	}

	// Errors inform users that something went wrong.
	#[derive(PartialEq, Eq)]
	#[pallet::error]
	pub enum Error<T> {
		/// The specified address has already been registered to another account.
		AddressAlreadyRegistered,

		/// The specified address has already been registered to this account.
		AddressAlreadyRegisteredByCaller,

		/// The specified address does not exist.
		NonExistentAddress,

		/// The specified deal order does not exist.
		NonExistentDealOrder,

		/// The specified ask order does not exist.
		NonExistentAskOrder,

		/// The specified bid order does not exist.
		NonExistentBidOrder,

		/// The specified offer does not exist.
		NonExistentOffer,

		/// The specified transfer does not exist.
		NonExistentTransfer,

		/// The transfer has already been registered.
		TransferAlreadyRegistered,

		/// The coin collection has already been registered.
		CollectCoinsAlreadyRegistered,

		/// The account that registered the transfer does
		/// not match the account attempting to use the transfer.
		TransferAccountMismatch,

		///The specified deal order ID does not match the transfer deal order ID.
		TransferDealOrderMismatch,

		///The amount on the deal order does not match the transfer amount.
		TransferAmountMismatch,

		/// The transfer has already been processed and cannot be used.
		TransferAlreadyProcessed,

		/// The transfer amount is less than the amount in the loan terms.
		TransferAmountInsufficient,

		/// The transfer is malformed and has a block number greater than the
		/// tip. This is an internal error.
		MalformedTransfer,

		/// The specified transfer type is not currently supported by
		/// the blockchain the loan is executed on.
		UnsupportedTransferKind,

		/// The node does not have sufficient authority to verify a transfer.
		InsufficientAuthority,

		/// The specified ID has already been used.
		DuplicateId,

		/// The address cannot be used because the user does not own it.
		NotAddressOwner,

		/// Failed to send an offchain callback transaction. This is likely
		/// an internal error.
		OffchainSignedTxFailed,

		/// The node is an authority but there is no account to create a
		/// callback transaction. This is likely an internal error.
		NoLocalAcctForSignedTx,

		RepaymentOrderNonZeroGain,

		/// The addresses specified are not on compatible external chains.
		AddressBlockchainMismatch,

		/// The account is already an authority.
		AlreadyAuthority,

		/// The account you are trying to remove is not  an authority.
		NotAnAuthority,

		/// The offer has already been made.
		DuplicateOffer,

		/// The deal cannot be locked because it is not funded yet.
		DealNotFunded,

		/// The deal order is already funded and cannot be funded again.
		DealOrderAlreadyFunded,
		/// The deal order is already closed and cannot be closed again.
		DealOrderAlreadyClosed,

		/// The deal order is already locked and cannot be locked again.
		DealOrderAlreadyLocked,

		/// The deal order must be locked before it can be closed.
		DealOrderMustBeLocked,

		/// The deal order already exists.
		DuplicateDealOrder,

		/// The deal order has expired and is no longer valid.
		DealOrderExpired,

		/// The ask order has expired and is no longer valid.
		AskOrderExpired,

		/// The bid order has expired and is no longer valid.
		BidOrderExpired,

		/// The offer order has expired and is no longer valid.
		OfferExpired,

		/// The terms of the ask and bid order do not agree.
		AskBidMismatch,

		/// The bid order is owned by the user, a user cannot lend to themself.
		SameOwner,

		/// The signature does not match the public key and message.
		InvalidSignature,

		/// Only the borrower can perform the action.
		NotBorrower,

		/// The deal order is malformed and has a block number greater than the
		/// tip. This is an internal error.
		MalformedDealOrder,

		/// Only the lender can perform the action.
		NotLender,

		/// Repayment orders are not currently supported.
		RepaymentOrderUnsupported,

		/// The legacy wallet is not owned by the user.
		NotLegacyWalletOwner,

		/// There is no legacy wallet corresponding to the public key.
		LegacyWalletNotFound,

		/// There is no legacy balance keeper, so no legacy wallets can be claimed.
		/// This is a configuration error and should only occur during local development.
		LegacyBalanceKeeperMissing,

		/// The specified guid has already been used and cannot be re-used.
		GuidAlreadyUsed,

		/// The value of the loan term's term length is zero, which is invalid.
		InvalidTermLength,

		/// The external address is malformed or otherwise invalid for the platform.
		MalformedExternalAddress,

		/// The address format was not recognized for the given blockchain and external address.
		AddressFormatNotSupported,

		/// The address retrieved from the proof-of-ownership signature did not match the external address being registered.
		OwnershipNotSatisfied,

		/// The currency has already been registered.
		CurrencyAlreadyRegistered,

		/// A valid external address could not be generated for the specified blockchain and recovered public key
		EthSignExternalAddressGenerationFailed,

		/// ECDSA public key recovery failed for an ownership proof using EthSign
		EthSignPublicKeyRecoveryFailed,

		/// A valid external address could not be generated for the specified blockchain and recovered public key
		PersonalSignExternalAddressGenerationFailed,

		/// ECDSA public key recovery failed for an ownership proof using PersonalSign
		PersonalSignPublicKeyRecoveryFailed,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub legacy_wallets: Vec<(LegacySighash, T::Balance)>,
		pub legacy_balance_keeper: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { legacy_wallets: Vec::new(), legacy_balance_keeper: None }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (sighash, balance) in &self.legacy_wallets {
				LegacyWallets::<T>::insert(sighash, balance);
			}
			if let Some(acct) = &self.legacy_balance_keeper {
				LegacyBalanceKeeper::<T>::put(acct.clone());
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			log::debug!("Cleaning up expired entries");

			let ask_count = AskOrders::<T>::clear_prefix(block_number, u32::MAX, None).backend;
			let bid_count = BidOrders::<T>::clear_prefix(block_number, u32::MAX, None).backend;
			let offer_count = Offers::<T>::clear_prefix(block_number, u32::MAX, None).backend;

			let mut deals_count = 0u32;
			let deals_to_keep: Vec<_> = DealOrders::<T>::drain_prefix(block_number)
				.filter_map(|(hash, deal)| {
					deals_count = deals_count.saturating_add(1);
					if deal.funding_transfer_id.is_some() {
						Some((DealOrderId::with_expiration_hash::<T>(block_number, hash), deal))
					} else {
						None
					}
				})
				.collect();
			let funded_deals_count = deals_to_keep.len().unique_saturated_into();
			let deals_count = deals_count.saturating_sub(funded_deals_count);
			for (key, deal) in deals_to_keep {
				DealOrders::<T>::insert_id(key, deal);
			}

			<T as Config>::WeightInfo::on_initialize(
				ask_count,
				bid_count,
				offer_count,
				deals_count,
				funded_deals_count,
			)
		}

		fn on_runtime_upgrade() -> Weight {
			migrations::migrate::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claims legacy wallet and transfers the balance to the sender's account.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::claim_legacy_wallet())]
		pub fn claim_legacy_wallet(
			origin: OriginFor<T>,
			public_key: sp_core::ecdsa::Public,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let account_id_of_key = T::Signer::from(public_key).into_account();
			ensure!(account_id_of_key == who, Error::<T>::NotLegacyWalletOwner);

			let sighash = LegacySighash::from(&public_key);

			let legacy_balance =
				LegacyWallets::<T>::get(&sighash).ok_or(Error::<T>::LegacyWalletNotFound)?;

			let legacy_keeper =
				LegacyBalanceKeeper::<T>::get().ok_or(Error::<T>::LegacyBalanceKeeperMissing)?;

			<pallet_balances::Pallet<T> as CurrencyT<T::AccountId>>::transfer(
				&legacy_keeper,
				&who,
				legacy_balance,
				ExistenceRequirement::AllowDeath,
			)?;
			LegacyWallets::<T>::remove(&sighash);
			Self::deposit_event(Event::<T>::LegacyWalletClaimed(who, sighash, legacy_balance));

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		/// Registers an external address on `blockchain` and `network` with value `address`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::register_address())]
		pub fn register_address(
			origin: OriginFor<T>,
			blockchain: Blockchain,
			address: ExternalAddress,
			ownership_proof: sp_core::ecdsa::Signature,
		) -> DispatchResult {
			// Get `origin` to look like the message to which the `ownership_proof` signature corresponds.
			// TODO: Why do we hash twice? I presume signed blake2 digest should be sufficient.
			// The signing happens here: https://github.com/gluwa/creditcoin/blob/d1918252b27069afccbca290f35e4ecd8fce0640/creditcoin-js/src/utils.ts#L9
			let who = ensure_signed(origin)?;
			let message = sp_io::hashing::sha2_256(who.encode().as_slice());
			// Prep message for public key recovery/extraction
			let message = &sp_io::hashing::blake2_256(message.as_ref());

			// Extract public key of keypair used to sign the address of the caller
			let signature = <[u8; 65]>::from(ownership_proof);
			let raw_pubkey = secp256k1_ecdsa_recover_compressed(&signature, message)
				.map_err(|_| Error::<T>::InvalidSignature)?;

			// Build the external address from the public key
			let recreated_address = helpers::generate_external_address(
				&blockchain,
				&address,
				sp_core::ecdsa::Public::from_raw(raw_pubkey),
			)
			.ok_or(Error::<T>::AddressFormatNotSupported)?;
			// Check if external address of keypair used to sign AccountID
			// is the same one mentioned in this call to register_address
			ensure!(recreated_address == address, Error::<T>::OwnershipNotSatisfied);

			let address_id = AddressId::new::<T>(&blockchain, &address);

			if let Ok(account_id) = Addresses::<T>::try_get(&address_id) {
				// Already registered, let's figure out who owns it so we can
				// return a nice error
				if who == account_id.owner {
					fail!(Error::<T>::AddressAlreadyRegisteredByCaller);
				}
				fail!(Error::<T>::AddressAlreadyRegistered);
			}

			// note: this error condition is unreachable!
			// AddressFormatNotSupported or OwnershipNotSatisfied will error out first
			ensure!(
				helpers::address_is_well_formed(&blockchain, &address),
				Error::<T>::MalformedExternalAddress
			);

			let entry = Address { blockchain, value: address, owner: who };
			Self::deposit_event(Event::<T>::AddressRegistered(address_id.clone(), entry.clone()));
			<Addresses<T>>::insert(address_id, entry);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::add_ask_order())]
		pub fn add_ask_order(
			origin: OriginFor<T>,
			address_id: AddressId<T::Hash>,
			terms: LoanTerms,
			expiration_block: BlockNumberFor<T>,
			guid: Guid,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let head = Self::block_number();
			ensure!(expiration_block >= head, Error::<T>::AskOrderExpired);

			let ask_order_id = AskOrderId::new::<T>(expiration_block, &guid);
			ensure!(!AskOrders::<T>::contains_id(&ask_order_id), Error::<T>::DuplicateId);

			let address = Self::get_address(&address_id)?;
			ensure!(address.owner == who, Error::<T>::NotAddressOwner);

			Self::use_guid(&guid)?;

			let ask_order = AskOrder {
				blockchain: address.blockchain,
				lender_address_id: address_id,
				terms: terms.try_into().map_err(Error::<T>::from)?,
				expiration_block,
				block: <frame_system::Pallet<T>>::block_number(),
				lender: who,
			};

			Self::deposit_event(Event::<T>::AskOrderAdded(ask_order_id.clone(), ask_order.clone()));
			AskOrders::<T>::insert_id(ask_order_id, ask_order);
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::add_bid_order())]
		pub fn add_bid_order(
			origin: OriginFor<T>,
			address_id: AddressId<T::Hash>,
			terms: LoanTerms,
			expiration_block: BlockNumberFor<T>,
			guid: Guid,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let head = Self::block_number();
			ensure!(expiration_block >= head, Error::<T>::BidOrderExpired);

			let bid_order_id = BidOrderId::new::<T>(expiration_block, &guid);
			ensure!(!BidOrders::<T>::contains_id(&bid_order_id), Error::<T>::DuplicateId);

			let address = Self::get_address(&address_id)?;
			ensure!(address.owner == who, Error::<T>::NotAddressOwner);

			Self::use_guid(&guid)?;

			let bid_order = BidOrder {
				blockchain: address.blockchain,
				borrower_address_id: address_id,
				terms: terms.try_into().map_err(Error::<T>::from)?,
				expiration_block,
				block: <frame_system::Pallet<T>>::block_number(),
				borrower: who,
			};

			Self::deposit_event(Event::<T>::BidOrderAdded(bid_order_id.clone(), bid_order.clone()));
			BidOrders::<T>::insert_id(bid_order_id, bid_order);
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::add_offer())]
		pub fn add_offer(
			origin: OriginFor<T>,
			ask_order_id: AskOrderId<T::BlockNumber, T::Hash>,
			bid_order_id: BidOrderId<T::BlockNumber, T::Hash>,
			expiration_block: BlockNumberFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let ask_order = try_get_id!(AskOrders<T>, &ask_order_id, NonExistentAskOrder)?;

			ensure!(ask_order.lender == who, Error::<T>::NotLender);

			let head = Self::block_number();

			ensure!(ask_order.expiration_block >= head, Error::<T>::AskOrderExpired);

			let bid_order = try_get_id!(BidOrders<T>, &bid_order_id, NonExistentBidOrder)?;

			ensure!(bid_order.borrower != who, Error::<T>::SameOwner);

			ensure!(bid_order.expiration_block >= head, Error::<T>::BidOrderExpired);

			ensure!(
				ask_order.blockchain == bid_order.blockchain,
				Error::<T>::AddressBlockchainMismatch
			);

			ensure!(ask_order.terms.match_with(&bid_order.terms), Error::<T>::AskBidMismatch);

			let offer_id = OfferId::new::<T>(expiration_block, &ask_order_id, &bid_order_id);

			ensure!(!Offers::<T>::contains_id(&offer_id), Error::<T>::DuplicateOffer);

			let offer = Offer {
				ask_id: ask_order_id,
				bid_id: bid_order_id,
				block: Self::block_number(),
				blockchain: ask_order.blockchain,
				expiration_block,
				lender: who,
			};

			Self::deposit_event(Event::<T>::OfferAdded(offer_id.clone(), offer.clone()));
			Offers::<T>::insert_id(offer_id, offer);

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::add_deal_order())]
		pub fn add_deal_order(
			origin: OriginFor<T>,
			offer_id: OfferId<T::BlockNumber, T::Hash>,
			expiration_block: BlockNumberFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let deal_order_id = DealOrderId::new::<T>(expiration_block, &offer_id);
			ensure!(!DealOrders::<T>::contains_id(&deal_order_id), Error::<T>::DuplicateDealOrder);

			let offer = try_get_id!(Offers<T>, &offer_id, NonExistentOffer)?;

			let head = Self::block_number();

			ensure!(offer.expiration_block >= head, Error::<T>::OfferExpired);

			let ask_order = try_get_id!(AskOrders<T>, &offer.ask_id, NonExistentAskOrder)?;

			let bid_order = try_get_id!(BidOrders<T>, &offer.bid_id, NonExistentBidOrder)?;

			ensure!(bid_order.borrower == who, Error::<T>::NotBorrower);

			let agreed_terms = ask_order
				.terms
				.agreed_terms(bid_order.terms)
				.ok_or(Error::<T>::AskBidMismatch)?;

			let deal_order = DealOrder {
				blockchain: offer.blockchain,
				offer_id,
				lender_address_id: ask_order.lender_address_id,
				borrower_address_id: bid_order.borrower_address_id,
				terms: agreed_terms,
				expiration_block,
				block: Some(Self::block_number()),
				timestamp: Self::timestamp(),
				borrower: who,
				funding_transfer_id: None,
				lock: None,
				repayment_transfer_id: None,
			};

			Self::deposit_event(Event::<T>::DealOrderAdded(
				deal_order_id.clone(),
				deal_order.clone(),
			));
			DealOrders::<T>::insert_id(deal_order_id, deal_order);

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::lock_deal_order())]
		pub fn lock_deal_order(
			origin: OriginFor<T>,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			DealOrders::<T>::try_mutate(
				deal_order_id.expiration(),
				deal_order_id.hash(),
				|value| -> DispatchResult {
					let deal_order = value.as_mut().ok_or(Error::<T>::NonExistentDealOrder)?;

					ensure!(deal_order.lock.is_none(), Error::<T>::DealOrderAlreadyLocked);
					ensure!(deal_order.funding_transfer_id.is_some(), Error::<T>::DealNotFunded);
					ensure!(deal_order.borrower == who, Error::<T>::NotBorrower);

					deal_order.lock = Some(who);
					Self::deposit_event(Event::<T>::DealOrderLocked(deal_order_id.clone()));
					Ok(())
				},
			)?;

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::fund_deal_order())]
		pub fn fund_deal_order(
			origin: OriginFor<T>,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
			transfer_id: TransferId<T::Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::try_mutate_deal_order_and_transfer(
				&deal_order_id,
				&transfer_id,
				|deal_order| {
					let lender =
						try_get!(Addresses<T>, &deal_order.lender_address_id, NonExistentAddress)?;

					ensure!(lender.owner == who, Error::<T>::NotLender);

					let now = Self::timestamp();
					ensure!(now >= deal_order.timestamp, Error::<T>::MalformedDealOrder);

					ensure!(
						deal_order.funding_transfer_id.is_none(),
						Error::<T>::DealOrderAlreadyFunded
					);
					let head = Self::block_number();
					ensure!(deal_order.expiration_block >= head, Error::<T>::DealOrderExpired);

					deal_order.funding_transfer_id = Some(transfer_id.clone());
					deal_order.timestamp = now;

					Ok(Some(Event::<T>::DealOrderFunded(deal_order_id.clone())))
				},
				|transfer, deal_order| {
					ensure!(
						transfer.order_id == OrderId::Deal(deal_order_id.clone()),
						Error::<T>::TransferDealOrderMismatch
					);
					ensure!(
						transfer.amount == deal_order.terms.amount,
						Error::<T>::TransferAmountMismatch
					);
					ensure!(transfer.account_id == who, Error::<T>::TransferAccountMismatch);
					ensure!(!transfer.is_processed, Error::<T>::TransferAlreadyProcessed);

					transfer.is_processed = true;
					Ok(Some(Event::<T>::TransferProcessed(transfer_id.clone())))
				},
			)?;

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::register_deal_order())]
		pub fn register_deal_order(
			origin: OriginFor<T>,
			lender_address_id: AddressId<T::Hash>,
			borrower_address_id: AddressId<T::Hash>,
			terms: LoanTerms,
			expiration_block: BlockNumberFor<T>,
			ask_guid: Guid,
			bid_guid: Guid,
			borrower_key: T::Signer,
			borrower_signature: T::SignerSignature,
		) -> DispatchResult {
			let lender_account = ensure_signed(origin)?;
			let borrower_account = borrower_key.into_account();

			let message = expiration_block
				.encode()
				.into_iter()
				.chain(ask_guid.encode())
				.chain(bid_guid.encode())
				.chain(terms.encode())
				.collect::<Vec<u8>>();

			ensure!(
				borrower_signature.verify(message.as_slice(), &borrower_account),
				Error::<T>::InvalidSignature
			);

			let borrower = Self::get_address(&borrower_address_id)?;
			ensure!(borrower.owner == borrower_account, Error::<T>::NotAddressOwner);

			let lender = Self::get_address(&lender_address_id)?;
			ensure!(lender.owner == lender_account, Error::<T>::NotAddressOwner);

			ensure!(lender.matches_chain_of(&borrower), Error::<T>::AddressBlockchainMismatch);

			let ask_order_id = AskOrderId::new::<T>(expiration_block, &ask_guid);
			ensure!(!AskOrders::<T>::contains_id(&ask_order_id), Error::<T>::DuplicateId);

			let bid_order_id = BidOrderId::new::<T>(expiration_block, &bid_guid);
			ensure!(!BidOrders::<T>::contains_id(&bid_order_id), Error::<T>::DuplicateId);

			let offer_id = OfferId::new::<T>(expiration_block, &ask_order_id, &bid_order_id);
			ensure!(!Offers::<T>::contains_id(&offer_id), Error::<T>::DuplicateOffer);

			let deal_order_id = DealOrderId::new::<T>(expiration_block, &offer_id);
			ensure!(!DealOrders::<T>::contains_id(&deal_order_id), Error::<T>::DuplicateDealOrder);

			let current_block = Self::block_number();

			let ask_order = AskOrder {
				blockchain: lender.blockchain.clone(),
				lender_address_id: lender_address_id.clone(),
				terms: terms.clone().try_into().map_err(Error::<T>::from)?,
				expiration_block,
				block: current_block,
				lender: lender_account.clone(),
			};

			let bid_order = BidOrder {
				blockchain: lender.blockchain.clone(),
				borrower_address_id: borrower_address_id.clone(),
				terms: terms.clone().try_into().map_err(Error::<T>::from)?,
				expiration_block,
				block: current_block,
				borrower: borrower_account.clone(),
			};

			let offer = Offer {
				ask_id: ask_order_id.clone(),
				bid_id: bid_order_id.clone(),
				block: current_block,
				blockchain: lender.blockchain.clone(),
				expiration_block,
				lender: lender_account,
			};

			let deal_order = DealOrder {
				blockchain: lender.blockchain,
				offer_id: offer_id.clone(),
				lender_address_id,
				borrower_address_id,
				terms,
				expiration_block,
				timestamp: Self::timestamp(),
				block: Some(Self::block_number()),
				borrower: borrower_account,
				funding_transfer_id: None,
				lock: None,
				repayment_transfer_id: None,
			};

			AskOrders::<T>::insert_id(ask_order_id.clone(), ask_order.clone());
			Self::deposit_event(Event::<T>::AskOrderAdded(ask_order_id, ask_order));

			BidOrders::<T>::insert_id(bid_order_id.clone(), bid_order.clone());
			Self::deposit_event(Event::<T>::BidOrderAdded(bid_order_id, bid_order));

			Offers::<T>::insert_id(offer_id.clone(), offer.clone());
			Self::deposit_event(Event::<T>::OfferAdded(offer_id, offer));

			DealOrders::<T>::insert_id(deal_order_id.clone(), deal_order.clone());
			Self::deposit_event(Event::<T>::DealOrderAdded(deal_order_id, deal_order));
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::close_deal_order())]
		pub fn close_deal_order(
			origin: OriginFor<T>,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
			transfer_id: TransferId<T::Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::try_mutate_deal_order_and_transfer(
				&deal_order_id,
				&transfer_id,
				|deal_order| {
					let borrower = try_get!(
						Addresses<T>,
						&deal_order.borrower_address_id,
						NonExistentAddress
					)?;

					ensure!(borrower.owner == who, Error::<T>::NotBorrower);

					let now = Self::timestamp();
					ensure!(now >= deal_order.timestamp, Error::<T>::MalformedDealOrder);

					ensure!(
						deal_order.repayment_transfer_id.is_none(),
						Error::<T>::DealOrderAlreadyClosed
					);

					ensure!(deal_order.lock.is_some(), Error::<T>::DealOrderMustBeLocked);

					deal_order.repayment_transfer_id = Some(transfer_id.clone());

					Ok(Some(Event::<T>::DealOrderClosed(deal_order_id.clone())))
				},
				|transfer, _deal_order| {
					ensure!(
						transfer.order_id == OrderId::Deal(deal_order_id.clone()),
						Error::<T>::TransferDealOrderMismatch
					);

					ensure!(transfer.block <= Self::block_number(), Error::<T>::MalformedTransfer);
					ensure!(transfer.account_id == who, Error::<T>::TransferAccountMismatch);
					ensure!(!transfer.is_processed, Error::<T>::TransferAlreadyProcessed);

					transfer.is_processed = true;
					Ok(Some(Event::<T>::TransferProcessed(transfer_id.clone())))
				},
			)?;

			Ok(())
		}

		#[transactional]
		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::request_collect_coins())]
		pub fn request_collect_coins(
			origin: OriginFor<T>,
			evm_address: ExternalAddress,
			tx_id: ExternalTxId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let contract = Self::collect_coins_contract();

			let pending = types::UnverifiedCollectedCoins { to: evm_address, tx_id, contract };

			let collect_coins_id = TaskV2::<T>::to_id(&pending);

			ensure!(
				!<UnverifiedCollectedCoins as TaskV2<T>>::is_persisted(&collect_coins_id),
				Error::<T>::CollectCoinsAlreadyRegistered
			);

			let deadline = T::TaskScheduler::deadline();

			ensure!(
				!T::TaskScheduler::is_scheduled(&deadline, &collect_coins_id),
				Error::<T>::CollectCoinsAlreadyRegistered
			);

			let address_id = AddressId::new::<T>(&pending.contract.chain, &pending.to);
			let address = Self::addresses(address_id).ok_or(Error::<T>::NonExistentAddress)?;
			ensure!(address.owner == who, Error::<T>::NotAddressOwner);

			T::TaskScheduler::insert(&deadline, &collect_coins_id, Task::from(pending.clone()));

			Self::deposit_event(Event::<T>::CollectCoinsRegistered(
				collect_coins_id.into(),
				pending,
			));

			Ok(())
		}

		#[transactional]
		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::register_funding_transfer())]
		pub fn register_funding_transfer(
			origin: OriginFor<T>,
			transfer_kind: TransferKind,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
			blockchain_tx_id: ExternalTxId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = try_get_id!(DealOrders<T>, &deal_order_id, NonExistentDealOrder)?;

			let (transfer_id, transfer) = Self::register_transfer_internal(
				who,
				order.lender_address_id,
				order.borrower_address_id,
				transfer_kind,
				order.terms.amount,
				OrderId::Deal(deal_order_id),
				blockchain_tx_id,
			)?;
			Self::deposit_event(Event::<T>::TransferRegistered(transfer_id, transfer));

			Ok(())
		}

		#[transactional]
		#[pallet::call_index(12)]
		#[pallet::weight(<T as Config>::WeightInfo::register_repayment_transfer())]
		pub fn register_repayment_transfer(
			origin: OriginFor<T>,
			transfer_kind: TransferKind,
			repayment_amount: ExternalAmount,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
			blockchain_tx_id: ExternalTxId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = try_get_id!(DealOrders<T>, &deal_order_id, NonExistentDealOrder)?;

			let (transfer_id, transfer) = Self::register_transfer_internal(
				who,
				order.borrower_address_id,
				order.lender_address_id,
				transfer_kind,
				repayment_amount,
				OrderId::Deal(deal_order_id),
				blockchain_tx_id,
			)?;
			Self::deposit_event(Event::<T>::TransferRegistered(transfer_id, transfer));

			Ok(())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::exempt())]
		pub fn exempt(
			origin: OriginFor<T>,
			deal_order_id: DealOrderId<T::BlockNumber, T::Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			DealOrders::<T>::try_mutate(
				deal_order_id.expiration(),
				deal_order_id.hash(),
				|value| -> DispatchResult {
					let deal_order =
						value.as_mut().ok_or(crate::Error::<T>::NonExistentDealOrder)?;
					ensure!(
						deal_order.repayment_transfer_id.is_none(),
						Error::<T>::DealOrderAlreadyClosed
					);

					let lender = Self::get_address(&deal_order.lender_address_id)?;
					ensure!(who == lender.owner, Error::<T>::NotLender);

					let fake_transfer = Transfer {
						order_id: OrderId::Deal(deal_order_id.clone()),
						block: Self::block_number(),
						account_id: who,
						amount: ExternalAmount::zero(),
						is_processed: true,
						kind: TransferKind::Native,
						tx_id: ExternalTxId::try_from(b"0".to_vec()).expect(
							"0 is a length of one which will always be < size bound of ExternalTxId",
						),
						blockchain: lender.blockchain,
						from: deal_order.lender_address_id.clone(),
						to: deal_order.lender_address_id.clone(),
						timestamp: Some(Self::timestamp()),
					};
					let fake_transfer_id =
						TransferId::new::<T>(&fake_transfer.blockchain, &fake_transfer.tx_id);

					deal_order.repayment_transfer_id = Some(fake_transfer_id);

					Ok(())
				},
			)?;

			Self::deposit_event(Event::<T>::LoanExempted(deal_order_id));
			Ok(())
		}

		#[transactional]
		#[pallet::call_index(16)]
		#[pallet::weight(match &task_output {
			crate::TaskOutput::CollectCoins(..) => <T as Config>::WeightInfo::persist_collect_coins(),
			crate::TaskOutput::VerifyTransfer(..) => <T as Config>::WeightInfo::persist_transfer(),
		})]
		pub fn persist_task_output(
			origin: OriginFor<T>,
			deadline: T::BlockNumber,
			task_output: TaskOutput<T::AccountId, T::Balance, T::BlockNumber, T::Hash, T::Moment>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(T::TaskScheduler::is_authority(&who), Error::<T>::InsufficientAuthority);

			let (task_id, event) = match task_output {
				TaskOutput::VerifyTransfer(id, transfer) => {
					ensure!(
						!Transfers::<T>::contains_key(&id),
						non_paying_error(Error::<T>::TransferAlreadyRegistered)
					);

					let mut transfer = transfer;
					transfer.block = frame_system::Pallet::<T>::block_number();

					Transfers::<T>::insert(&id, transfer);
					(id.clone().into_inner(), Event::<T>::TransferVerified(id))
				},
				TaskOutput::CollectCoins(id, collected_coins) => {
					ensure!(
						!CollectedCoins::<T>::contains_key(&id),
						non_paying_error(Error::<T>::CollectCoinsAlreadyRegistered)
					);

					let address = Self::addresses(&collected_coins.to)
						.ok_or(Error::<T>::NonExistentAddress)?;

					<pallet_balances::Pallet<T> as Mutate<T::AccountId>>::mint_into(
						&address.owner,
						collected_coins.amount,
					)?;

					CollectedCoins::<T>::insert(&id, collected_coins.clone());
					(id.clone().into_inner(), Event::<T>::CollectedCoinsMinted(id, collected_coins))
				},
			};
			T::TaskScheduler::remove(&deadline, &task_id);

			Self::deposit_event(event);

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		#[pallet::call_index(17)]
		#[pallet::weight(match &task_id {
			crate::TaskId::VerifyTransfer(..) => <T as Config>::WeightInfo::fail_transfer(),
			crate::TaskId::CollectCoins(..) => <T as Config>::WeightInfo::fail_collect_coins(),
		})]
		pub fn fail_task(
			origin: OriginFor<T>,
			deadline: T::BlockNumber,
			task_id: TaskId<T::Hash>,
			cause: VerificationFailureCause,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(T::TaskScheduler::is_authority(&who), Error::<T>::InsufficientAuthority);

			let (task_id, event) = match task_id {
				TaskId::VerifyTransfer(transfer_id) => {
					ensure!(
						!Transfers::<T>::contains_key(&transfer_id),
						Error::<T>::TransferAlreadyRegistered
					);
					(
						transfer_id.clone().into_inner(),
						Event::<T>::TransferFailedVerification(transfer_id, cause),
					)
				},
				TaskId::CollectCoins(collected_coins_id) => {
					ensure!(
						!CollectedCoins::<T>::contains_key(&collected_coins_id),
						Error::<T>::CollectCoinsAlreadyRegistered
					);
					(
						collected_coins_id.clone().into_inner(),
						Event::<T>::CollectCoinsFailedVerification(collected_coins_id, cause),
					)
				},
			};
			T::TaskScheduler::remove(&deadline, &task_id);
			Self::deposit_event(event);

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		#[pallet::call_index(18)]
		#[pallet::weight(<T as Config>::WeightInfo::add_authority())]
		pub fn add_authority(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(!T::TaskScheduler::is_authority(&who), Error::<T>::AlreadyAuthority);

			T::TaskScheduler::insert_authority(&who);

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		#[transactional]
		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config>::WeightInfo::set_collect_coins_contract())]
		pub fn set_collect_coins_contract(
			origin: OriginFor<T>,
			contract: GCreContract,
		) -> DispatchResult {
			ensure_root(origin)?;
			CollectCoinsContract::<T>::put(contract);
			Ok(())
		}

		#[transactional]
		#[pallet::call_index(21)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_authority())]
		pub fn remove_authority(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(T::TaskScheduler::is_authority(&who), Error::<T>::NotAnAuthority);

			T::TaskScheduler::remove_authority(&who);

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		/// Registers an address on an external blockchain as the property of an onchain address.
		/// To prove ownership, a signature is provided. To create the signature, the public key of the external address is used to sign a hash of the account_id of whoever is submitting this transaction.
		/// The signature type allows the caller to specify if this address was signed using the older an insecure EthSign method or the new PersonalSign method. See here for details https://docs.metamask.io/wallet/how-to/sign-data/
		#[pallet::call_index(22)]
		#[pallet::weight(<T as Config>::WeightInfo::register_address_v2())]
		pub fn register_address_v2(
			origin: OriginFor<T>,
			blockchain: Blockchain,
			address: ExternalAddress,
			ownership_proof: OwnershipProof,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let encoded = who.encode();
			let account = encoded.as_slice();

			match helpers::try_extract_address::<T>(ownership_proof, account, &blockchain, &address)
			{
				Ok(recreated_address) => {
					// Check if external address of keypair used to sign AccountID
					// is the same one mentioned in this call to register_address
					ensure!(recreated_address == address, Error::<T>::OwnershipNotSatisfied);
					let address_id = AddressId::new::<T>(&blockchain, &address);

					if let Ok(account_id) = Addresses::<T>::try_get(&address_id) {
						// Already registered, let's figure out who owns it so we can
						// return a nice error
						if who == account_id.owner {
							fail!(Error::<T>::AddressAlreadyRegisteredByCaller);
						}
						fail!(Error::<T>::AddressAlreadyRegistered);
					}

					// note: this error condition is unreachable!
					// AddressFormatNotSupported or OwnershipNotSatisfied will error out first
					ensure!(
						helpers::address_is_well_formed(&blockchain, &address),
						Error::<T>::MalformedExternalAddress
					);

					let entry = Address { blockchain, value: address, owner: who };
					Self::deposit_event(Event::<T>::AddressRegistered(
						address_id.clone(),
						entry.clone(),
					));
					<Addresses<T>>::insert(address_id, entry);
					Ok(())
				},
				Err(e) => {
					fail!(e)
				},
			}
		}
	}
}
