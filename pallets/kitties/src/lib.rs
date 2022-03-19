#![cfg_attr(not(feature = "std"), no_std)]

mod types;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use frame_support::{
		dispatch::{fmt::Debug, Codec, DispatchResult},
		pallet_prelude::*,
		sp_io::hashing::blake2_128,
	};
	// 引入随机数以及代币、可质押代币ReservableCurrency（用于后续创建kitty时的质押）
	use frame_support::traits::{
		Currency, ExistenceRequirement::KeepAlive, Randomness, ReservableCurrency,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One, Zero, SaturatedConversion};
	use sp_std::{prelude::*, vec::Vec};

	pub use crate::types::{GetKittyMarketResult, KittyInfoById, KittyInfo, MarketKittyqueryError, Kitty};

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// 声明一个Randomness的实现，满足Hash和BlockNumber做为类型参数
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		// 引入代币的关联类型
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		// 在runtime中定义kittyIndex
		type KittyIndex: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen;

		// 质押费用
		type ReservationFee: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// 创建者，kittyid
		KittyCreate(T::AccountId, T::KittyIndex),
		// 原有主人，新主人，kittyid
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
		// 正在售卖的kittyid,金额
		SellingKitty(T::KittyIndex, BalanceOf<T>),
		// 已经出售的kittyid,原有的主人，新主人
		SeltKitty(T::KittyIndex, T::AccountId, T::AccountId),
	}

	/// 定义存储
	/// 1. 要记录kitties的index，首先需要记录kitties的数量，这样才能知道写一个kitty的id是多少
	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	/// 2. 每个kitty都有自己的数据
	/// kittyIndex作为key，kitty的数据作为value
	/// 使用Blake2_128Concat作为Hash函数的方法
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	/// 3. 每个kitty的所有者owner
	/// kittyIndex作为key，AccountId作为value
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	// 注意这里要加Config这个泛型，不然就会找不到关联类型AccountId
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	/// 4.定义一个kitty交易市场，用来对需要卖的kitty进行挂单
	/// kittyindex为key,挂单的金额为value
	#[pallet::storage]
	#[pallet::getter(fn kittymarket)]
	pub type KittyMarket<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, BalanceOf<T>, ValueQuery>;

	/// 5.定义一个存储，能够通过指定的账户查询到其拥有的所有kittyId
	#[pallet::storage]
	#[pallet::getter(fn kittybabies)]
	pub type KittyBabies<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::KittyIndex>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// kittyId溢出错误
		KittiesCountOverflow,
		/// 不是kittyId的owner
		NotOwner,
		/// 两个相同的祖先
		SameParentIndex,
		/// 指定的KittyIndex的数据不存在
		InvalidKittyIndex,
		/// 没有足够的质押金额
		NoSufficientBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 创建kitty
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			// 检查调用者的身份
			let who = ensure_signed(origin)?;

			// 创建kitty之前质押一定的金额
			T::Currency::reserve(&who, T::ReservationFee::get())
				.map_err(|_| Error::<T>::NoSufficientBalance)?;

			// 1. kitty的id
			// kitties_count()是获取的下一个kitty的id,它是用于获取上面KittiesCount存储项的getter函数
			let kitty_id: <T as Config>::KittyIndex = match Self::kitties_count() {
				Some(id) => {
					// 如果当前kittyId的id已经超过了KittyIndex的最大值(u32最大值)，说明就无法再创建新的Index，就报溢出错误
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					// 否则就从当前id开始
					id
				},
				// 如果没有获取到，就从0开始
				None => Zero::zero(),
			};

			// 2. kitty的data
			// 通过随机的方式来获取,通过random_value方法来获取，其实现在impl中
			let dna = Self::random_value(&who);

			// 3. 更新三个Storage的存储信息，分别是kitty数据，kitty所有者，kitty的数量
			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id + One::one()); // 更新下一个kitty的index，即+1

			// 更新账户拥有的kittyId列表
			Self::push_kitty_babies_list(&who.clone(), kitty_id);

			// 4. 抛出event(已经创建了新的kitty)
			Self::deposit_event(Event::KittyCreate(who, kitty_id));

			Ok(().into())
		}

		/// 转移Kitty的所有者
		/// para1： 新的owner
		/// para2:  需要转移的kitty_id
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			new_owner: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// 核查当前的调用kittyid的人是否是它的owner，如果不是就没有权限进行调用
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			// 新的拥有者需要质押一定的金额
			T::Currency::reserve(&new_owner.clone(), T::ReservationFee::get())
				.map_err(|_| Error::<T>::NoSufficientBalance)?;

			// 插入新的Owner
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

			// 更新新的owner之后,退回原拥有者的质押金额
			T::Currency::unreserve(&who, T::ReservationFee::get());

			// 将kitty_id从old owner的kittyId列表中移除
			Self::remove_kittyid_from_kitty_babies_list(&who.clone(), kitty_id.clone());

			// 更新新owner的kittyId列表
			Self::push_kitty_babies_list(&new_owner.clone(), kitty_id);

			Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));
			Ok(())
		}

		/// 繁殖kitty
		/// para1: 父kitty
		/// para2: 母kitty
		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 确保进行繁殖的kittyId是不同的
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			// 确保两个kittyId是否已经有数据了，如果没有就抛出异常
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			// 得到child kitty的Id
			let child_kitty_id = match Self::kitties_count() {
				Some(id) => {
					// 如果当前kittyId的id已经超过了KittyIndex的最大值(u32最大值)，说明就无法再创建新的Index，就报溢出错误
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					// 否则就从当前id开始
					id
				},
				// 如果没有获取到，就从0开始
				None => Zero::zero(),
			};

			// 根据两个Parent的DNA进行混淆，产生新的child的DNA
			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			// 根据调用者的账户身份产生一个random作为一个Selector
			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				// 当selector的位为1的时候就使用dna_1的值，当selector的位为0的时候，就使用dna_2的值，作为新的dna值（通过位运算）
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			// 更新 child Kitty 的存储
			Kitties::<T>::insert(child_kitty_id, Some(Kitty(new_dna)));
			Owner::<T>::insert(child_kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(child_kitty_id + One::one());

			// 更新当前用户的kittyId列表
			Self::push_kitty_babies_list(&who.clone(), child_kitty_id);
			// 创建成功事件
			Self::deposit_event(Event::KittyCreate(who, child_kitty_id));

			Ok(())
		}

		/// 卖Kitty，将需要卖的kittyIndex放进kittymarket进行挂单
		#[pallet::weight(0)]
		pub fn sell_kitties(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			selling_value: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 确保挂单的人是Kitty的拥有者
			ensure!(Some(who) == <Owner<T>>::get(kitty_id), <Error<T>>::NotOwner);

			// 将kittyid以及对应的金额挂单到kitty的交易市场
			<KittyMarket<T>>::insert(kitty_id, selling_value);

			Self::deposit_event(Event::SellingKitty(kitty_id, selling_value));
			Ok(())
		}

		/// 买kitty，从kittymarket接单，买入
		#[pallet::weight(0)]
		pub fn buy_kitties(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let new_owner = ensure_signed(origin)?;

			// 判断需要购买的kitty_id在kitty市场中是否存在，如果指定的kitty的价格为0，那么就不存在
			ensure!(<KittyMarket<T>>::get(kitty_id) != Zero::zero(), <Error<T>>::InvalidKittyIndex);

			// 获取kitty交易市场中指定kittyindex的挂单价格
			let kitty_price = <KittyMarket<T>>::get(kitty_id);

			// 获取kitty_id的拥有者(因为Owner中的值为option类型，所以需要unwrap一下)
			let old_owner = <Owner<T>>::get(kitty_id).unwrap();

			// 判断当前账户的余额是否大于需要质押的金额，如果不是，就报错
			ensure!(
				T::Currency::free_balance(&new_owner) > kitty_price,
				<Error<T>>::NoSufficientBalance
			);

			// 将当前新的账户扣除挂单的金额给原始账户，用于购买kitty
			T::Currency::transfer(&old_owner, &new_owner, kitty_price, KeepAlive)?;

			// 并将kitty给新的owner
			<Owner<T>>::insert(kitty_id, Some(new_owner.clone()));

			// 将kitty_id从old owner的kittyId列表中移除
			Self::remove_kittyid_from_kitty_babies_list(&old_owner.clone(), kitty_id.clone());

			// 将买的kittyId追加到新的owner的kittyId列表中
			Self::push_kitty_babies_list(&new_owner.clone(),kitty_id);

			// 将该kitty_id的原有账户释放质押金额
			T::Currency::unreserve(&old_owner, T::ReservationFee::get());

			// 此时就已经完成了交易市场中的kitty的交易，将该id从交易市场中剔除
			<KittyMarket<T>>::remove(kitty_id);

			Self::deposit_event(Event::SeltKitty(kitty_id, old_owner, new_owner));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	{
		/// 获取随机数
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				// 随机数
				T::Randomness::random_seed(),
				// 调用该方法的Account
				&sender,
				// 这笔交易在block中的Index
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			// 使用blake2_128将payload进行hash产生一个128位的数据，作为kitty的DNA（数据）
			payload.using_encoded(blake2_128)
		}

		// 更新kitty_babies_list
		fn push_kitty_babies_list(who: &T::AccountId, kitty_id: <T as Config>::KittyIndex) {
			// 更新特定账户的 kittybabies 存储
			if KittyBabies::<T>::contains_key(&who.clone()) {
				let mut babies = KittyBabies::<T>::get(who.clone());
				babies.push(kitty_id);
				KittyBabies::<T>::insert(&who.clone(), babies);
			} else {
				let mut babies: Vec<<T as Config>::KittyIndex> = [].to_vec();
				babies.push(kitty_id);
				KittyBabies::<T>::insert(&who.clone(), babies);
			}
		}

		// 从kitty_babies_list中移除指定的kitty_id
		fn remove_kittyid_from_kitty_babies_list(who: &T::AccountId, kitty_id: <T as Config>::KittyIndex){
			let mut babies = KittyBabies::<T>::get(who.clone());
			// 遍历kitty_babies_list,找到kitty_id的下标，通过下标移除该kitty_id
			for (index, val) in babies.clone().iter().enumerate(){
				if val == &kitty_id{
					babies.swap_remove(index);
				}
			}
			KittyBabies::<T>::insert(&who.clone(),babies);
		}

		// 查询当前账户在kittiy_market中的kitty信息(kittyid,price)
		pub fn query_kittiy_market_info() -> GetKittyMarketResult<<T as frame_system::Config>::AccountId, BalanceOf<T>>{
			log::info!("..............................");
			log::info!(">>>>>>>>>>>>>>>>>>>>>>> start query kittiy market info <<<<<<<<<<<<<<<<<<<<<<<<<<<");
			
			let mut market_info: Vec<KittyInfoById<T::AccountId, BalanceOf<T>>> = [].to_vec();
			//
			let mut count = KittiesCount::<T>::get().unwrap();
			// 将kittyIndex自定义类型转换为u64,得到kitty的数量
			let numbers = count.saturated_into::<u64>();

			log::info!("kitty数量为：{}",&numbers.clone());

			// 设置一个计数器
			let mut counter = numbers as i64;
			//这里按照kittyIndex进行倒序遍历
			while counter > 0 {
				// let mut num = KittiesCount::<T>::get().unwrap();
				counter = counter - 1;
				log::info!("当前计数为:{}",counter.clone());
				// 当前这个count就是kitty_id
				count = count - One::one();
				// 如果当前遍历的kittyid在market中可以查找到
				if KittyMarket::<T>::get(count) == Zero::zero(){
					log::info!("当前kittyid:{}的价格为:{:?},不在挂单市场",count.saturated_into::<u64>(),KittyMarket::<T>::get(count));
					continue;
				}else {
					log::info!("----开始查询market中的kitty信息-----");
					log::info!("当前kitty id为:{}", count.saturated_into::<u64>());
					// 查找当前kitty_id的price和owner
					let price:	BalanceOf<T> = KittyMarket::<T>::get(count);
					log::info!("当前kitty的价格为:{:?}",price.clone());
					let owner = Owner::<T>::get(count).unwrap();
					log::info!("当前kitty的主人为:{:?}",owner.clone());
					let kitty_dna = Kitties::<T>::get(count).unwrap();
					log::info!("当前kitty的dna为:{:?}",kitty_dna);
					// let owner = Owner::<T>::get(kitty_id).unwrap();
					// Result<Vec<KittyInfoById<AccountId, Balance>>, MarketKittyqueryError>
					market_info.push(
						KittyInfoById{
							kitty_index: count.saturated_into::<u64>(),
							info: KittyInfo{
								owner,
								price,
								kitty_dna,
							}
						}
					);
				}
			}
			log::info!("当前market_info列表为:{:?}",market_info);
			log::info!("..............................");
			log::info!(">>>>>>>>>>>>>>>>>>>>>>> End query kittiy market info <<<<<<<<<<<<<<<<<<<<<<<<<<<");
			market_info

		}

		// 查询某个账户在market中的kitty信息
			/* let mut kittyid_list = KittyBabies::get(who.clone());
			kittyid_list.reverse();  */
			/* 
			// 倒序遍历
			for i in 0..kittyid_list.clone().len(){
				if let Some(kitty_id) = kittyid_list.clone().get(i){
					let price =  KittyMarket::get(kitty_id);
					// Result<Vec<BTreeMap<AccountId, Vec<KittiyMarketInfo<Balance>>>>, MarketKittyqueryError>;
					let mut kitty_market_info: Vec<KittiyMarketInfo> = [].to_vec();
					kitty_market_info.push(
						KittiyMarketInfo{
							kitty_id,
							price,
						}
					);
				}
			} */
	}
}
