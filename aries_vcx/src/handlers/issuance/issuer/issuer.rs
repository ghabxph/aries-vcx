use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::issuance::issuer::state_machine::IssuerSM;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::messages::a2a::A2AMessage;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Issuer {
    issuer_sm: IssuerSM,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerConfig {
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum IssuerState {
    Initial,
    OfferSent,
    RequestReceived,
    CredentialSent,
    Finished,
    Failed,
}

impl Issuer {
    pub fn create(issuer_config: &IssuerConfig, credential_data: &str, source_id: &str) -> VcxResult<Issuer> {
        trace!("Issuer::issuer_create_credential >>> issuer_config: {:?}, credential_data: {:?}, source_id: {:?}", issuer_config, credential_data, source_id);

        let issuer_sm = IssuerSM::new(&issuer_config.cred_def_id.to_string(), credential_data, issuer_config.rev_reg_id.clone(), issuer_config.tails_file.clone(), source_id);
        Ok(Issuer { issuer_sm })
    }

    pub fn send_credential_offer(&mut self, send_message: impl Fn(&A2AMessage) -> VcxResult<()>, comment: Option<String>) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialInit(comment), Some(&send_message))
    }

    pub fn send_credential(&mut self, send_message: impl Fn(&A2AMessage) -> VcxResult<()>) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialSend(), Some(&send_message))
    }

    pub fn get_state(&self) -> IssuerState {
        self.issuer_sm.get_state()
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.issuer_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.issuer_sm.find_message_to_handle(messages)
    }

    pub fn revoke_credential(&self, publish: bool) -> VcxResult<()> {
        self.issuer_sm.revoke(publish)
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_reg_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_thread_id()
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        self.issuer_sm.is_revokable()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().handle_message(message, send_message)?;
        Ok(())
    }

    pub fn update_state(&mut self, connection: &Connection) -> VcxResult<IssuerState> {
        trace!("Issuer::update_state >>>");
        if self.is_terminal_state() { return Ok(self.get_state()); }
        let send_message = connection.send_message_closure()?;

        let messages = connection.get_messages()?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(msg.into(), Some(&send_message))?;
            connection.update_message_status(uid)?;
        }
        Ok(self.get_state())
    }
}

#[cfg(test)]
pub mod test {
    use crate::messages::issuance::credential::test_utils::_credential;
    use crate::messages::issuance::credential_offer::test_utils::_credential_offer;
    use crate::messages::issuance::credential_proposal::test_utils::{_credential_proposal, _cred_def_id};
    use crate::messages::issuance::credential_request::test_utils::_credential_request;
    use crate::messages::issuance::test::{_ack, _problem_report};
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;
    use crate::handlers::issuance::issuer::state_machine::test::{_send_message, _tails_file, _rev_reg_id};
    use agency_client::mocking::HttpClientMockResponse;

    use super::*;

    fn _cred_data() -> String {
        json!({"name": "alice"}).to_string()
    }

    fn _issuer() -> Issuer {
        let issuer_config = IssuerConfig {
            cred_def_id: _cred_def_id(),
            rev_reg_id: Some(_rev_reg_id()),
            tails_file: Some(_tails_file())
        };
        Issuer::create(&issuer_config, &_cred_data(), "test_source_id").unwrap()
    }

    fn _issuer_unrevokable() -> Issuer {
        let issuer_config = IssuerConfig {
            cred_def_id: _cred_def_id(),
            rev_reg_id: None,
            tails_file: None
        };
        Issuer::create(&issuer_config, &_cred_data(), "test_source_id").unwrap()
    }

    fn _send_message_but_fail() -> Option<&'static impl Fn(&A2AMessage) -> VcxResult<()>> {
        Some(&|_: &A2AMessage| Err(VcxError::from(VcxErrorKind::IOError)))
    }

    impl Issuer {
        fn to_offer_sent_state(mut self) -> Issuer {
            self.step(CredentialIssuanceMessage::CredentialInit(None), _send_message()).unwrap();
            self
        }

        fn to_request_received_state(mut self) -> Issuer {
            self.step(CredentialIssuanceMessage::CredentialInit(None), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> Issuer {
            self.step(CredentialIssuanceMessage::CredentialInit(None), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialSend(), _send_message()).unwrap();
            self
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_cant_revoke_without_revocation_details() {
        let _setup = SetupMocks::init();
        let issuer = _issuer_unrevokable().to_finished_state();
        assert_eq!(IssuerState::Finished, issuer.get_state());
        let revoc_result = issuer.revoke_credential(true);
        assert_eq!(revoc_result.unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails)
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_can_be_resent_after_failure() {
        let _setup = SetupMocks::init();
        let mut issuer = _issuer().to_request_received_state();
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        let send_result = issuer.send_credential(_send_message_but_fail().unwrap());
        assert_eq!(send_result.is_err(), true);
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        let send_result = issuer.send_credential(_send_message().unwrap());
        assert_eq!(send_result.is_err(), false);
        assert_eq!(IssuerState::Finished, issuer.get_state());
    }
}
