use crate as pallet_kitties;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// 定义一个u128的类型别名为Balance，下面的ReserveFee的设置以及Pallet_balances需要用到
pub type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	// 构造一个测试用的runtime Test
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// 一个是系统模块，一个是需要测试的模块
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		KittiesModule: pallet_kitties::{Pallet, Call, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	// 对于未使用的关联类型给的是一个空的tuple
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64; // u64，当在写测试的时候可以用一个整数来构造AccountId
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	// 这里需要设置为1,
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	// 设置需要质押的金额，可以在runtime里动态调整的，这里设置为4，即创建一个kitty需要质押4个代币
    pub const ReservationFee: Balance = 4;
}

impl pallet_kitties::Config for Test {
	type Event = Event;
	type Randomness = RandomnessCollectiveFlip;
	// 代币的类型使用上面引入的Balances,其类型为pallet_balances
	type Currency = Balances;
	type KittyIndex = u64;
	type ReservationFee = ReservationFee;
}

// Build genesis storage according to the mock runtime.
// 	
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test>{
		// 设置账户0的余额为100
		// 设置账户1的余额为200
		// 设置账户2的余额为300
		balances: vec![(0,200),(1,300),(2,400)],
	}.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	// 设置起始区块高度
	ext.execute_with(|| System::set_block_number(1));
	ext
}
