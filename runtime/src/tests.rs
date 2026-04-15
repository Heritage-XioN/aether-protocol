use super::{
    configs::{fee_handler::TreasuryAccount, *},
    mock::*,
    BalancesCall, EXISTENTIAL_DEPOSIT, UNIT,
};
use codec::Encode;
use frame_support::{
    assert_ok,
    dispatch::{GetDispatchInfo, PostDispatchInfo},
};
use sp_core::TypedGet;
use sp_runtime::{
    traits::{AccountIdConversion, DispatchTransaction},
    AccountId32,
};

pub type TestBalances = pallet_balances::Pallet<Test>;

#[test]
fn test_fees_go_to_treasury_account() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Get treasury account
        let treasury_account = TreasuryAccount::<Test>::get();
        // Give treasury some initial balance so it can pay for beneficiary transfers
        assert_ok!(TestBalances::force_set_balance(
            RuntimeOrigin::root(),
            treasury_account.clone(),
            10 * EXISTENTIAL_DEPOSIT,
        ));

        // get treasury initial balance
        let initial_treasury_balance = TestBalances::free_balance(&treasury_account);

        // Create two regular users
        let sender = [1; 32];
        let recipient = [2; 32];

        // Give sender some funds
        assert_ok!(TestBalances::force_set_balance(
            RuntimeOrigin::root(),
            AccountId32::new(sender),
            100 * UNIT
        ));

        let initial_sender_balance = TestBalances::free_balance(AccountId32::new(sender));
        let initial_recipient_balance = TestBalances::free_balance(AccountId32::new(recipient));

        // Execute a transfer (this generates a fee)
        let transfer_amount = 50 * UNIT;

        let call = RuntimeCall::Balances(BalancesCall::transfer_keep_alive {
            dest: AccountId32::new(recipient),
            value: transfer_amount,
        });

        let dispatch_info = call.get_dispatch_info();

        let tip = 0;
        let len = call.encoded_size();

        // simulate full transaction with fee charging
        pallet_transaction_payment::ChargeTransactionPayment::<Test>::from(tip)
            .test_run(
                RuntimeOrigin::signed(AccountId32::new(sender)),
                &call,
                &dispatch_info,
                len,
                0,
                |_| {
                    assert_ok!(TestBalances::transfer_keep_alive(
                        RuntimeOrigin::signed(AccountId32::new(sender)),
                        AccountId32::new(recipient),
                        transfer_amount
                    ));

                    Ok(PostDispatchInfo {
                        actual_weight: None,
                        pays_fee: Default::default(),
                    })
                },
            )
            .unwrap()
            .unwrap();

        // Calculate expected fee
        let final_sender_balance = TestBalances::free_balance(AccountId32::new(sender));
        let final_recipient_balance = TestBalances::free_balance(&AccountId32::new(recipient));
        let final_treasury_balance = TestBalances::free_balance(&treasury_account);

        // Sender lost: transfer_amount + fee
        let sender_loss = initial_sender_balance - final_sender_balance;
        assert!(sender_loss > transfer_amount, "Fee was not deducted");

        // Recipient gained exactly the transfer amount
        assert_eq!(
            final_recipient_balance - initial_recipient_balance,
            transfer_amount,
            "Recipient got wrong amount"
        );

        // Treasury gained the difference (the fee)
        let fee_amount = sender_loss - transfer_amount;
        let treasury_gain = final_treasury_balance - initial_treasury_balance;
        assert_eq!(
            treasury_gain, fee_amount,
            "Treasury did not receive the full fee amount"
        );

        // 5. Verify treasury account is correct (not zero address)
        assert_ne!(
            treasury_account,
            AccountId32::new([0; 32]),
            "Treasury account is zero address - funds would be lost!"
        );
    });
}

#[test]
fn test_treasury_account_is_derived_correctly() {
    new_test_ext().execute_with(|| {
        let treasury_via_fee_handler = TreasuryAccount::<Test>::get();

        // Manually derive using the same method
        let pallet_id = TreasuryPalletId::get();
        let treasury_via_manual = pallet_id.into_account_truncating();

        assert_eq!(
            treasury_via_fee_handler, treasury_via_manual,
            "Treasury account derivation mismatch!"
        );
    });
}
