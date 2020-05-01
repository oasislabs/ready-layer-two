use std::time::SystemTime;

use oasis_std::{abi::*, collections::Map, Address, Context, Event};

#[derive(oasis_std::Service)]
struct Competition {
    /// The address of the Participant service used to authenticate and manage users.
    user_registry: Address,

    /// Public data used to train submitted models.
    train_dataset: AuthenticatedData,

    /// Private data that will be used by the evaluation program to score results.
    test_dataset: EncryptedData,

    /// The evaluation program that will be used to score the submissions.
    /// The evaluation program runs on trusted hardware and (by providing its attestation)
    /// is the only entity authorized to declare the winner of the competition.
    evaluation_program: AuthenticatedData,

    /// A collection of participant's current submission.
    submissions: Map<String /* username */, EncryptedData /* the model and its params */>,

    /// Unix timestamp at which this competition closes and submissions can be evaluated.
    end_timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedData {
    url: String,
    /// The expected hash of the data (or the measurement, if the data is a program).
    hash: Vec<u8>,
}

/// An on-chain pointer to encrypted off-chain data.
/// The key is only made available to the evaluation program after successful attestation.
#[derive(Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    url: String,
    cipher: Aes256GcmParams,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Aes256GcmParams {
    key: Vec<u8>,
    iv: Vec<u8>,
    tag: Vec<u8>,
}

impl Competition {
    pub fn new(
        _ctx: &Context,
        user_registry: Address,
        train_dataset: AuthenticatedData,
        test_dataset: EncryptedData,
        evaluation_program: AuthenticatedData,
        end_timestamp: u64,
    ) -> Self {
        Self {
            user_registry,
            train_dataset,
            test_dataset,
            evaluation_program,
            end_timestamp,
            submissions: Map::new(),
        }
    }

    pub fn get_public_state(&self, _ctx: &Context) -> PublicState {
        PublicState {
            user_registry: &self.user_registry,
            train_dataset: &self.train_dataset,
            evaluation_program: &self.evaluation_program,
            end_timestamp: self.end_timestamp,
        }
    }

    pub fn submit(
        &mut self,
        _ctx: &Context,
        participant_auth_token: String,
        model: EncryptedData,
    ) -> Result<(), Error> {
        if !self.is_accepting_submissions() {
            return Err(Error::SubmissionsClosed);
        }

        let p_reg_client = user_registry::UserRegistryClient::new(self.user_registry);
        match p_reg_client.verify_token(&Context::default(), &participant_auth_token) {
            Ok(Ok(user_info)) => {
                self.submissions.insert(user_info.name, model);
                Ok(())
            }
            Ok(Err(_)) => Err(Error::PermissionDenied),
            Err(_) => Err(Error::ParticipantRegistryUnreachable),
        }
    }

    fn is_accepting_submissions(&self) -> bool {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        current_time.unwrap().as_secs() < self.end_timestamp
    }
}

impl Competition {
    //! Evaluation methods. Can only be called by an attested evaluation program enclave.

    pub fn begin_evaluation(
        &self,
        _ctx: &Context,
        attestation: AttestationReport,
    ) -> Result<EvaluationSecrets, Error> {
        self.authorize_evaluation_program(&attestation)?;
        Ok(EvaluationSecrets {
            test_dataset: self.test_dataset.clone(),
            submissions: self.submissions.clone(),
        })
    }

    pub fn announce_winner(
        &self,
        _ctx: &Context,
        attestation: AttestationReport,
        winner: String,
    ) -> Result<(), Error> {
        self.authorize_evaluation_program(&attestation)?;
        Event::emit(&CompetitionCompleted { winner: &winner });
        Ok(())
    }

    fn authorize_evaluation_program(&self, attestation: &AttestationReport) -> Result<(), Error> {
        // if this were a real attestation, we'd validate the signature, but you get the idea.
        if self.is_accepting_submissions()
            || attestation.measurement != self.evaluation_program.hash
        {
            return Err(Error::PermissionDenied);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AttestationReport {
    measurement: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct EvaluationSecrets {
    test_dataset: EncryptedData,
    submissions: Map<String, EncryptedData>,
}

#[derive(Serialize)]
pub enum Error {
    ParticipantRegistryUnreachable,
    PermissionDenied,
    SubmissionsClosed,
}

#[derive(Serialize, Event)]
pub struct CompetitionCompleted<'a> {
    /// The username of the participant who made the winning submission.
    winner: &'a String,
}

/// The public state of this service that can safely be returned to clients.
#[derive(Serialize)]
pub struct PublicState<'a> {
    user_registry: &'a Address,
    train_dataset: &'a AuthenticatedData,
    evaluation_program: &'a AuthenticatedData,
    end_timestamp: u64,
}

fn main() {
    oasis_std::service!(Competition);
}
