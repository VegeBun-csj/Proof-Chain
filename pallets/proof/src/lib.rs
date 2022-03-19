#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// 设置长度上限（这里主要是为了设置存证内容的hash）,因为链上的存证内容不能无限大，否则容易受到攻击
		type MaxAddend: Get<usize>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber)>;

	// 定义Event类型
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		ClaimOutLength,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 创建存证
		#[pallet::weight(0)] // 可以通过计算来获取weight合理的值，这里是简单起见，设置为0
		pub fn create_claim(
			origin: OriginFor<T>, // 交易的发送方
			claim: Vec<u8>,       // 存证的hash值
		) -> DispatchResultWithPostInfo {
			// 校验并获取发送方的AccountId
			let sender = ensure_signed(origin)?;

			// 限制claim的长度
			ensure!(claim.len().le(&(T::MaxAddend::get())), Error::<T>::ClaimOutLength);

			// 判断当前的存储单元中，是否已经存在了这样的存证记录，如果存在了，那就报已经存在的错误
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			// 向链上存储单元中插入一条存证记录
			Proofs::<T>::insert(
				&claim,                                                      /* key为 存证的hash值 */
				(sender.clone(), frame_system::Pallet::<T>::block_number()), /* value为 一个元组，其中包含(AccountId,BlockNumber) */
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}

		// 注销存证
		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin: OriginFor<T>, // 交易的发送方
			claim: Vec<u8>,       // 存证的hash值
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// 如果Proofs中没有，就报错，如果有，就用?将其中的值取出来
			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		// 转移存证
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			rec_account: T::AccountId, // 接受方的账户地址
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// 如果Proofs中没有，就报错，如果有，就用?将其中的值取出来
			let (owner, block_number) =
				Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 确定当前的存证是否是属于当前的发送者
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&claim);
			Proofs::<T>::insert(&claim, (rec_account, block_number));

			Self::deposit_event(Event::ClaimTransfered(sender, claim));

			Ok(().into())
		}

		// 销售存证
		// #[pallet::weight(0)]
		// pub fn sell_claim(
		// 	origin: OriginFor<T>, // 交易的发送方
		// 	claim: Vec<u8>, // 存证
		// ) -> DispatchResultWithPostInfo{

		// }
	}
}
