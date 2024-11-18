use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::{NeutronError, NeutronResult};
use neutron_std::shim::Any;
use neutron_std::types::cosmos::auth::v1beta1::BaseAccount as StdBaseAccount;
use neutron_std::types::neutron::interchainqueries::StorageValue;
use prost::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// A structure that can be reconstructed from **StorageValues**'s for the **Account Interchain Query**.
pub struct BaseAccount {
    pub address: String,
    pub account_number: String,
    pub sequence: String,
}

impl KVReconstruct for BaseAccount {
    fn reconstruct(storage_values: &[StorageValue]) -> NeutronResult<BaseAccount> {
        if storage_values.len() != 1 {
            return Err(NeutronError::InvalidQueryResultFormat(
                "storage_values length is not 1".into(),
            ));
        }

        // first level value is Any as sdk.AccountI implementation:
        // https://github.com/cosmos/cosmos-sdk/blob/853dbbf3e84900214137805d78e325ecd56fd68f/types/account.go#L9-L32
        let any_value: Any = Any::decode(storage_values[0].value.as_slice())?;
        // second level value is BaseAccount:
        // https://github.com/cosmos/cosmos-sdk/blob/853dbbf3e84900214137805d78e325ecd56fd68f/x/auth/types/auth.pb.go#L29-L37
        let std_acc: StdBaseAccount = StdBaseAccount::decode(any_value.value.as_slice())?;
        Ok(BaseAccount {
            address: std_acc.address,
            account_number: std_acc.account_number.to_string(),
            sequence: std_acc.sequence.to_string(),
        })
    }
}
