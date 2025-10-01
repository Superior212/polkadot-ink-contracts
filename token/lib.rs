#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod token {
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    /// Stores a mapping from AccountId to u128 for token balances.
    #[ink(storage)]
    pub struct Token {
        /// Mapping from AccountId to token balance (u128)
        balances: Mapping<AccountId, u128>,
    }

    impl Default for Token {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Token {
        /// Constructor that initializes the token contract with empty balances.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                balances: Mapping::new(),
            }
        }

        /// Constructor that initializes the token contract with empty balances.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Mint tokens to a specific account.
        /// Increases the balance of the specified account by the given amount.
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) {
            let current_balance = self.balances.get(to).unwrap_or(0);
            let new_balance = current_balance.checked_add(amount)
                .expect("Balance overflow");
            self.balances.insert(to, &new_balance);
        }

        /// Get the balance of a specific account.
        /// Returns 0 if the account has no balance.
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
        }

        /// Transfer tokens from the caller to another account.
        /// Returns an error if the caller has insufficient balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<(), Error> {
            let caller = self.env().caller();
            let caller_balance = self.balances.get(caller).unwrap_or(0);
            
            if caller_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(to).unwrap_or(0);
            
            let new_caller_balance = caller_balance.checked_sub(amount)
                .expect("Balance underflow");
            let new_to_balance = to_balance.checked_add(amount)
                .expect("Balance overflow");
            
            self.balances.insert(caller, &new_caller_balance);
            self.balances.insert(to, &new_to_balance);
            
            Ok(())
        }
    }

    /// Custom error types for the token contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let token = Token::default();
            // Test that a new account has zero balance
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            assert_eq!(token.balance_of(accounts.alice), 0);
        }

        /// We test minting functionality.
        #[ink::test]
        fn mint_works() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Mint 100 tokens to Alice
            token.mint(accounts.alice, 100);
            assert_eq!(token.balance_of(accounts.alice), 100);
            
            // Mint more tokens to Alice
            token.mint(accounts.alice, 50);
            assert_eq!(token.balance_of(accounts.alice), 150);
        }

        /// We test transfer functionality.
        #[ink::test]
        fn transfer_works() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Mint tokens to Alice
            token.mint(accounts.alice, 100);
            assert_eq!(token.balance_of(accounts.alice), 100);
            assert_eq!(token.balance_of(accounts.bob), 0);
            
            // Set Alice as the caller for the transfer
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            
            // Transfer 30 tokens from Alice to Bob
            let result = token.transfer(accounts.bob, 30);
            assert!(result.is_ok());
            assert_eq!(token.balance_of(accounts.alice), 70);
            assert_eq!(token.balance_of(accounts.bob), 30);
        }

        /// We test transfer with insufficient balance.
        #[ink::test]
        fn transfer_insufficient_balance() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Set Alice as the caller (she has no balance)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            
            // Try to transfer more than Alice has
            let result = token.transfer(accounts.bob, 100);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::InsufficientBalance);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TokenRef::default();

            // When
            let contract = client
                .instantiate("token", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Token>();

            // Then
            let balance_of = call_builder.balance_of(ink_e2e::alice().account_id());
            let balance_result = client.call(&ink_e2e::alice(), &balance_of).dry_run().await?;
            assert_eq!(balance_result.return_value(), 0);

            Ok(())
        }

        /// We test that we can mint tokens and check balances.
        #[ink_e2e::test]
        async fn mint_and_balance_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TokenRef::new();
            let contract = client
                .instantiate("token", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Token>();

            // When - mint 100 tokens to Alice
            let mint = call_builder.mint(ink_e2e::alice().account_id(), 100);
            let _mint_result = client
                .call(&ink_e2e::alice(), &mint)
                .submit()
                .await
                .expect("mint failed");

            // Then - check Alice's balance
            let balance_of = call_builder.balance_of(ink_e2e::alice().account_id());
            let balance_result = client.call(&ink_e2e::alice(), &balance_of).dry_run().await?;
            assert_eq!(balance_result.return_value(), 100);

            Ok(())
        }

        /// We test that we can transfer tokens between accounts.
        #[ink_e2e::test]
        async fn transfer_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TokenRef::new();
            let contract = client
                .instantiate("token", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Token>();

            // Mint tokens to Alice
            let mint = call_builder.mint(ink_e2e::alice().account_id(), 100);
            let _mint_result = client
                .call(&ink_e2e::alice(), &mint)
                .submit()
                .await
                .expect("mint failed");

            // When - transfer 30 tokens from Alice to Bob
            let transfer = call_builder.transfer(ink_e2e::bob().account_id(), 30);
            let _transfer_result = client
                .call(&ink_e2e::alice(), &transfer)
                .submit()
                .await
                .expect("transfer failed");

            // Then - check balances
            let alice_balance = call_builder.balance_of(ink_e2e::alice().account_id());
            let alice_balance_result = client.call(&ink_e2e::alice(), &alice_balance).dry_run().await?;
            assert_eq!(alice_balance_result.return_value(), 70);

            let bob_balance = call_builder.balance_of(ink_e2e::bob().account_id());
            let bob_balance_result = client.call(&ink_e2e::alice(), &bob_balance).dry_run().await?;
            assert_eq!(bob_balance_result.return_value(), 30);

            Ok(())
        }
    }
}
