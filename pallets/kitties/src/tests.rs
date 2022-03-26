use crate::{mock::*};
use frame_support::{assert_noop, assert_ok};
use super::*;

/// 创建Kitty的测试用例
#[test]
fn create_kitties_work(){
	new_test_ext().execute_with(|| {
		// 账户0创建kitty，此时的kitty的id为0,其中需要质押4个代币
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 判断是否余额为96
		assert_eq!(Balances::free_balance(0),196);
		// 判断id为0的kitty的拥有者是否为账户0
		assert_eq!(Owner::<Test>::get(0),Some(0));
		// 判断创建完kitty后kitty的数量是否正确
		assert_eq!(KittiesCount::<Test>::get().unwrap(),1);
		// 判断创建kitty后，判断当前账户的kittyId列表中的kittyId是否为0
		assert_eq!(KittyBabies::<Test>::get(0)[0],0);
	})
}



/// 转移kitty的测试
#[test]
fn transfer_kitty_work(){
	new_test_ext().execute_with(|| {
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 判断创建kitty后，判断账户0的kittyId列表中的kittyId是否为0
		assert_eq!(KittyBabies::<Test>::get(0)[0],0);
		// 将id为0的kitty从账户0转移给账户1
		assert_ok!(KittiesModule::transfer(Origin::signed(0),1,0));
		// 判断转移后，账户0的kittyId数量为0
		assert_eq!(KittyBabies::<Test>::get(0).len(),0);
		// 判断转移后，账户1的kittyId列表中有0号kitty
		assert_eq!(KittyBabies::<Test>::get(1)[0],0);
		// 判断是否出现转移kitty的错误，因为此时账户0已经不是id为0的kitty的拥有者
		assert_noop!(KittiesModule::transfer(Origin::signed(0),1,0),Error::<Test>::NotOwner);
		// 判断是否id为0的拥有者是否为账户1
		assert_eq!(Owner::<Test>::get(0),Some(1));
	})
}


/// 繁殖kitty的测试
#[test]
fn breed_kitty_work(){
	new_test_ext().execute_with(|| {
		// 用账户0创建两个kitty
		// 创建id为0的kitty
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 创建id为1的kitty
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 账户0将两个kitty进行繁殖出id为2的子孙kitty
		assert_ok!(KittiesModule::breed(Origin::signed(0),0,1));
		// 查看是否已经繁殖了id为2的kitty，并且其主人为账户0
		assert_eq!(Owner::<Test>::get(2),Some(0));
		// 判断账户0的kittyId列表有三个kittyId
		assert_eq!(KittyBabies::<Test>::get(0).len(),3);

		// 判断列表的最后一个id是否为2
		assert_eq!(KittyBabies::<Test>::get(0)[2],2);
		// 创建id为3的kitty
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// 检测是否总共的kitty数量为4
		assert_eq!(KittiesCount::<Test>::get().unwrap(),4);
	})
}


/// 卖kitty的测试
#[test]
fn sell_kitty_work(){
	new_test_ext().execute_with(|| {
		// 账户0创建了一个id为0的kitty
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 账户1创建了一个id为1的kitty
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// 账户0将id为0的kitty放到kitty市场进行挂单,标价20个代币
		assert_ok!(KittiesModule::sell_kitties(Origin::signed(0),0,20));
		// 查看kitty市场，是否id为0的kitty的挂单价格为20
		assert_eq!(KittyMarket::<Test>::get(0),20);
		// 账户1试图将id为0的kitty放到市场进行挂单，就会出现错误，因为它不是该kitty的拥有者
		assert_noop!(KittiesModule::sell_kitties(Origin::signed(1),0,30),Error::<Test>::NotOwner);
	})
}


/// 买kitty的测试
#[test]
fn buy_kitty_work(){
	new_test_ext().execute_with(|| {
		// 账户0创建了一个id为0的kitty
		assert_ok!(KittiesModule::create(Origin::signed(0)));
		// 账户0将id为0的kitty放到kitty市场进行挂单,标价20个代币
		assert_ok!(KittiesModule::sell_kitties(Origin::signed(0),0,20));
		// 使用账户1购买id为0的kitty
		assert_ok!(KittiesModule::buy_kitties(Origin::signed(1),0));
		// 判断是否账户1为id为0的kitty的主人
		assert_eq!(Owner::<Test>::get(0),Some(1));
		// 判断账户0的kittyId列表是否为空
		assert_eq!(KittyBabies::<Test>::get(0).len(),0);
		// 判断账户1的kittyId列表中的kittyId是否为0
		assert_eq!(KittyBabies::<Test>::get(1)[0],0);
		// 判断是否市场上还有id为0的kitty,即是否还可以购买kitty 0，如果还可以购买就报错
		assert_noop!(KittiesModule::buy_kitties(Origin::signed(2),0),Error::<Test>::InvalidKittyIndex);
	})
}

/// 移除kitty测试
