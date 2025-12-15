//! See [`MuralPayMock`].

use {
    crate::{
        Account, AccountId, BankDetailsResponse, Counterparty, CounterpartyId, CreateCounterparty,
        CreatePayout, FiatAndRailCode, FiatFeeRequest, FiatPayoutFee, MuralError, Organization,
        OrganizationId, PayoutMethod, PayoutMethodDetails, PayoutMethodId, PayoutRequest,
        PayoutRequestId, PayoutStatusFilter, SearchParams, SearchRequest, SearchResponse,
        TokenFeeRequest, TokenPayoutFee, UpdateCounterparty,
        transaction::{Transaction, TransactionId},
    },
    std::fmt::{self, Debug},
};

macro_rules! impl_mock {
    (
        $(fn $fn:ident ( $( $ty:ty ),* ) -> $ret:ty);* $(;)?
    ) => {
        /// Mock data returned by [`crate::MuralPay`].
        pub struct MuralPayMock {
            $(
            pub $fn: Box<dyn Fn($($ty),*) -> $ret + Send + Sync>,
            )*
        }

        impl Default for MuralPayMock {
            fn default() -> Self {
                Self {
                    $(
                    $fn: Box::new(|$(_: $ty),*| panic!("missing mock for `{}`", stringify!($fn))),
                    )*
                }
            }
        }
    };
}

impl_mock! {
    fn get_all_accounts() -> Result<Vec<Account>, MuralError>;
    fn get_account(AccountId) -> Result<Account, MuralError>;
    fn create_account(&str, Option<&str>) -> Result<Account, MuralError>;
    fn search_payout_requests(Option<PayoutStatusFilter>, Option<SearchParams<PayoutRequestId>>) -> Result<SearchResponse<PayoutRequestId, PayoutRequest>, MuralError>;
    fn get_payout_request(PayoutRequestId) -> Result<PayoutRequest, MuralError>;
    fn get_fees_for_token_amount(&[TokenFeeRequest]) -> Result<Vec<TokenPayoutFee>, MuralError>;
    fn get_fees_for_fiat_amount(&[FiatFeeRequest]) -> Result<Vec<FiatPayoutFee>, MuralError>;
    fn create_payout_request(AccountId, Option<&str>, &[CreatePayout]) -> Result<PayoutRequest, MuralError>;
    fn execute_payout_request(PayoutRequestId) -> Result<PayoutRequest, MuralError>;
    fn cancel_payout_request(PayoutRequestId) -> Result<PayoutRequest, MuralError>;
    fn get_bank_details(&[FiatAndRailCode]) -> Result<BankDetailsResponse, MuralError>;
    fn search_payout_methods(CounterpartyId, Option<SearchParams<PayoutMethodId>>) -> Result<SearchResponse<PayoutMethodId, PayoutMethod>, MuralError>;
    fn get_payout_method(CounterpartyId, PayoutMethodId) -> Result<PayoutMethod, MuralError>;
    fn create_payout_method(CounterpartyId, &str, &PayoutMethodDetails) -> Result<PayoutMethod, MuralError>;
    fn delete_payout_method(CounterpartyId, PayoutMethodId) -> Result<(), MuralError>;
    fn search_organizations(SearchRequest) -> Result<SearchResponse<OrganizationId, Organization>, MuralError>;
    fn get_organization(OrganizationId) -> Result<Organization, MuralError>;
    fn search_counterparties(Option<SearchParams<CounterpartyId>>) -> Result<SearchResponse<CounterpartyId, Counterparty>, MuralError>;
    fn get_counterparty(CounterpartyId) -> Result<Counterparty, MuralError>;
    fn create_counterparty(&CreateCounterparty) -> Result<Counterparty, MuralError>;
    fn update_counterparty(CounterpartyId, &UpdateCounterparty) -> Result<Counterparty, MuralError>;
    fn get_transaction(TransactionId) -> Result<Transaction, MuralError>;
    fn search_transactions(AccountId, Option<SearchParams<AccountId>>) -> Result<SearchResponse<AccountId, Account>, MuralError>;
}

impl Debug for MuralPayMock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MuralPayMock").finish_non_exhaustive()
    }
}
