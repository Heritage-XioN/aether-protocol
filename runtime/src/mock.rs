use super::{
    configs::{
        fee_handler::{DealWithFees, TreasuryAccount},
        *,
    },
    AccountId, Balance, EXISTENTIAL_DEPOSIT,
};
use frame_support::{
    derive_impl,
    pallet_prelude::*,
    parameter_types,
    traits::{
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        ConstU128, ConstU32, ConstU8, VariantCountOf,
    },
    weights::{IdentityFee, WeightToFee},
};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
    // The main runtime
    #[runtime::runtime]
    // Runtime Types to be generated
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Test>;

    #[runtime::pallet_index(1)]
    pub type Balances = pallet_balances::Pallet<Test>;

    #[runtime::pallet_index(2)]
    pub type TransactionPayment = pallet_transaction_payment::Pallet<Test>;

    #[runtime::pallet_index(3)]
    pub type Treasury = pallet_treasury::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

pub struct TestWeightToFee;
impl WeightToFee for TestWeightToFee {
    type Balance = Balance;
    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        // Use constant 1 for predictable fees in tests
        weight.ref_time() as Balance
    }
}

impl pallet_transaction_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, DealWithFees<Test>>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = TestWeightToFee;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Test>;
}

impl pallet_treasury::Config for Test {
    type Currency = Balances;
    type RejectOrigin = frame_system::EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type PalletId = TreasuryPalletId;
    type BurnDestination = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Test>;
    type SpendFunds = ();
    type MaxApprovals = ConstU32<100>;
    type SpendOrigin = frame_system::EnsureWithSuccess<
        frame_system::EnsureRoot<AccountId>,
        AccountId,
        ConstU128<{ u128::MAX }>,
    >;
    type AssetKind = ();
    type Beneficiary = AccountId;
    type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount<Test>>;
    type BalanceConverter = UnityAssetBalanceConversion;
    type PayoutPeriod = PayoutPeriod;
    /// Helper type for benchmarks.
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
    type BlockNumberProvider = System;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
