use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use scale_info::TypeInfo;

pub type KittyIndex = u64;

// pub type KittyIndex<T> = pallet::Config::KittyIndex;
/// 定义rpc返回的消息结构
///
/// [
///
/// 	kitty_1:{
/// 		price:
/// 		owner:
/// 		kitty_dna:
/// 	},
/// 	kitty_2:{
/// 		price:
/// 		owner:
/// 		kitty_dna:
/// 	},
///
/// ]
///
/// -
pub type GetKittyMarketResult<AccountId, Balance> = Vec<KittyInfoById<AccountId, Balance>>;

#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum MarketKittyqueryError {
	DoNotExistKitty,
}

/// 首先定义存储的数据ContractAccessError类型
/// 1.每一个kitty都需要存放数据，那么这个数据就可以用一个vec存放，为了存储方便，定义一个16字节的u8类型
/// 这样这些数据就可以通过256位的hash函数来获取
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, PartialOrd, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Kitty(pub [u8; 16]);

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct KittyInfoById<AccountId, Balance> {
	pub kitty_index: KittyIndex,
	pub info: KittyInfo<AccountId, Balance>,
}

// 定义返回消息的基础结构
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct KittyInfo<AccountId, Balance> {
	pub owner: AccountId,
	pub price: Balance,
	pub kitty_dna: Kitty,
}
