use super::pallet::Index;
use crate::pallet::Error;
use crate::Config;
use extend::ext;
use frame_support::BoundedBTreeMap;
use frame_support::BoundedBTreeSet;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

type Runners<I, S, M> = BoundedBTreeMap<I, S, M>;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(MaxSize))]
pub struct Supporters<Who, Power, MaxSize> {
	partial: Power,
	accounts: BoundedBTreeSet<Who, MaxSize>,
}

#[ext(name = RunnersExt)]
impl<Item, Supporter, MaxSize> Runners<Item, Supporter, MaxSize> {}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(MaxSize))]
pub struct Entry<Who, Item, Power, MaxSize> {
	total: Power,
	pub(crate) runners: Runners<Item, Supporters<Who, Power, MaxSize>, MaxSize>,
}

#[ext(name = IndexExt)]
impl<T: Config> Index<T> {
	fn insert_once(who: &T::Who, id: &T::ItemId) -> Result<(), Error<T>> {
		Self::mutate(id, |set| {
			if set
				.get_or_insert(BoundedBTreeSet::new())
				.try_insert(who.clone())
				.map_err(|_| Error::TooManyVoters)?
			{
				Ok(())
			} else {
				Err(Error::<T>::DoubleVoting)
			}
		})
	}
}
