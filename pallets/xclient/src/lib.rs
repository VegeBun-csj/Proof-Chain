#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use scale_info::TypeInfo;

#[frame_support::pallet]
pub mod pallet {
	use cumulus_primitives_core::ParaId;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::{vec, vec::Vec};
	use xcm::latest::prelude::*;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
	pub struct XregisterCall<AccountId> {
		// 这里的call_index是一个长度为2的vec，其中第一个参数为pallet名，第二个参数为pallet的方法名
		// 也就是调用哪个pallet的哪个方法
		call_index: [u8; 2],
		// 下面两个参数是需要向xserver注册提供的<account,name>，作为Register这个storage来进行存储
		account: AccountId,
		name: Vec<u8>,
	}

	// 这是XregisterCall结构体的一个构造方法,它会构造一个XregisterCall结构体
	impl<AccountId> XregisterCall<AccountId> {
		pub fn new(pallet_index: u8, method_index: u8, account: AccountId, name: Vec<u8>) -> Self {
			XregisterCall { call_index: [pallet_index, method_index], account, name }
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The XCM sender module.
		type XcmSender: SendXcm;

		/// Xregister maximum weight
		type XregisterWeightAtMost: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// <Account,name>
		Xregister(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error to send xcm to Xregister server
		XcmSendError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn xregister(
			origin: OriginFor<T>,
			pallet_id: u8,
			method_id: u8,
			name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// 构造一个registercall的请求
			let call = XregisterCall::<T::AccountId>::new(
				pallet_id, // palletId
				method_id, // pallet中的mathod Id
				who.clone(),          // accountId
				name.clone(),                // 注册的name
			);

			// build the xcm transact message
			let message = Xcm(vec![Instruction::Transact {
				origin_type: OriginKind::Native,
				require_weight_at_most: T::XregisterWeightAtMost::get(), //在runtime中指定
				call: call.encode().into(),
			}]);

			// send the message to xregister server chain
			// 这里调用进行跨链调用交易的时候其实是一个层级关系:
			// 在当前的parachain上调用的时候，会先到relaychain，即parent父级，
			// 然后进入到目标链parachain,再进入到该链的pallet，然后是其中的method

			// 把下面的message发送到destination parachain上
			// 然后根据message中的call进行相关pallet方法的调用
			match T::XcmSender::send_xcm((1, Junction::Parachain(4000u32.into())), message) {
				// send_xcm结果是一个result
				Ok(()) => {
					// emit the event if send successfully
					Self::deposit_event(Event::Xregister(who, name));
					Ok(().into())
				},
				Err(_) => Err(Error::<T>::XcmSendError.into()),
			}
		}
	}
}
