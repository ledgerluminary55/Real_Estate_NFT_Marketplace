#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult, inherent::Vec, pallet_prelude::*, sp_runtime::traits::Hash,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	// Account are used in profile struct
	type AccountOf<T> = <T as frame_system::Config>::AccountId;

	// Struct for holding Profile information
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Profile<T: Config, BoundedString> {
		pub owner: AccountOf<T>,
		pub name:BoundedString,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn profile_count)]
	/// Storage Value that counts the total number of Profiles
	// pub(super) type ProfileCount<T: Config> = StorageValue<_, u32, ValueQuery>;
	pub(super) type ProfileCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn profiles)]
	/// Stores a profile unique properties in a StorageMap
	pub(super) type Profiles<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Profile<T, BoundedVec<u8, T::StringLimit>>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// Profile was successfully created.
		ProfileCreated { who: T::AccountId },

		/// Profile was successfully deleted.
		ProfileDeleted { who: T::AccountId },

		/// Profile was successfully updated.
		ProfileUpdated { who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Reached maximum number of profiles.
		ProfileCountOverflow,
		/// No permission to update this profile.
		NoUpdateAuthority,
		/// Profiles can only be deleted by the creator
		NoDeletionAuthority,
		/// One Account can only create a single profile.
		ProfileAlreadyCreated,
		/// This Account has not yet created a profile.
		NoProfileCreated,
		Badmetadata,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dispatchable call that enables every new actor to create personal profile in storage.
		#[pallet::weight(10_000)]
		pub fn create_profile(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer
			let account = ensure_signed(origin)?;

			// Call helper function to generate Profile struct
			let _profile_id = Self::generate_profile(&account, name)?;

			// Emit an event
			Self::deposit_event(Event::ProfileCreated { who: account });
			Ok(())
		}
	}

	// Helper internal functions
	impl<T: Config> Pallet<T> {
		pub fn generate_profile(owner: &T::AccountId, name: Vec<u8>) -> Result<T::Hash, Error<T>> {
			// Check if profile is already exists for owner
			ensure!(!Profiles::<T>::contains_key(&owner), Error::<T>::ProfileAlreadyCreated);

			let bounded_name: BoundedVec<u8, T::StringLimit> = name.clone().try_into().map_err(|_| Error::<T>::Badmetadata)?;

			// Populate profile struct
			let profile = Profile::<T, BoundedVec<u8, T::StringLimit>> { owner: owner.clone(), name: bounded_name };

			// Get hash of profile
			let profile_id = T::Hashing::hash_of(&profile);

			// Insert profile into HashMap
			<Profiles<T>>::insert(owner, profile);

			// Increase profile count
			let new_count =
				Self::profile_count().checked_add(1).ok_or(<Error<T>>::ProfileCountOverflow)?;
			<ProfileCount<T>>::put(new_count);

			Ok(profile_id)
		}
	}
}
