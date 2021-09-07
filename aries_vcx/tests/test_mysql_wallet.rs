#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[cfg(feature = "mysql_test")]
#[macro_use]
extern crate sqlx;

#[cfg(feature = "mysql_test")]
mod dbtests {
    use sqlx::{Connection, MySqlConnection};
    use sqlx::Executor;
    use sqlx::prelude;
    use sqlx::prelude::*;

    #[cfg(feature = "mysql_test")]
    pub async fn setup_mysql_walletdb() -> Result<String, sqlx::Error> {
        info!("Running query.");
        let db_name = format!("mysqltest_{}", uuid::Uuid::new_v4().to_string()).replace("-", "_");
        let url = "mysql://root:mysecretpassword@localhost:3306";
        let mut connection = MySqlConnection::connect(url).await?;
        let query = format!("CREATE DATABASE {};", db_name);
        let query = sqlx::query(&query);
        let res = query.execute(&mut connection).await;
        warn!("Create database result: {:?}", res);
        connection.close().await;

        let url = format!("mysql://root:mysecretpassword@localhost:3306/{}", db_name);
        let mut connection = MySqlConnection::connect(&url).await?;
        let res = sqlx::migrate!("./migrations")
            .run(&mut connection)
            .await;
        warn!("Create tables result: {:?}", res);
        Ok(db_name)
    }

    #[cfg(test)]
    #[cfg(feature = "mysql_test")]
    mod test {
        use std::env;
        use std::fs::File;
        use std::io::Read;

        use serde_json::Value;
        use uuid::Uuid;

        use aries_vcx::{libindy, settings};
        use aries_vcx::handlers::connection::connection::Connection;
        use aries_vcx::init::{init_issuer_config, open_as_main_wallet};
        use aries_vcx::libindy::utils::wallet::{close_main_wallet, configure_issuer_wallet, create_wallet, WalletConfig};
        use aries_vcx::utils::devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
        use aries_vcx::utils::provision::{AgentProvisionConfig, provision_cloud_agent};
        use aries_vcx::utils::test_logger::LibvcxDefaultLogger;

        use futures::executor::block_on;
        use crate::dbtests::setup_mysql_walletdb;

        #[test]
        fn test_provision_cloud_agent_with_mysql_wallet() {
            LibvcxDefaultLogger::init_testing_logger();
            let db_name = block_on(setup_mysql_walletdb()).unwrap();

            let storage_config = json!({
            "read_host": "localhost",
            "write_host": "localhost",
            "port": 3306,
            "db_name": db_name,
            "default_connection_limit": 50
          }).to_string();
            let storage_credentials = json!({
            "user": "root",
            "pass": "mysecretpassword"
        }).to_string();
            let enterprise_seed = "000000000000000000000000Trustee1";
            let config_wallet = WalletConfig {
                wallet_name: format!("faber_wallet_{}", uuid::Uuid::new_v4().to_string()),
                wallet_key: settings::DEFAULT_WALLET_KEY.into(),
                wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
                wallet_type: Some("mysql".into()),
                storage_config: Some(storage_config),
                storage_credentials: Some(storage_credentials),
                rekey: None,
                rekey_derivation_method: None,
            };
            let config_provision_agent = AgentProvisionConfig {
                agency_did: AGENCY_DID.to_string(),
                agency_verkey: AGENCY_VERKEY.to_string(),
                agency_endpoint: AGENCY_ENDPOINT.to_string(),
                agent_seed: None,
            };
            create_wallet(&config_wallet).unwrap();
            open_as_main_wallet(&config_wallet).unwrap();
            let config_issuer = configure_issuer_wallet(enterprise_seed).unwrap();
            init_issuer_config(&config_issuer).unwrap();
            provision_cloud_agent(&config_provision_agent).unwrap();
            close_main_wallet().unwrap();
        }
    }
}