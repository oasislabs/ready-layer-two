# ready-layer-two

This repo contains a confidential machine learning competition demo app.
Basically Kaggle, but such that competition creator's private test data is kept hidden from participants, and the participants' models are kept hidden from the competition creator.

This demo was created for presentation at the [Ready Layer One](https://readylayer.one/) blockchain expo.

## How it works

<img src="/docs/sequence.png" alt="application sequence diagram" width="500" />

The Oasis platform, provides confidential smart contracts (i.e. private state).
The private state of the [Competition service](/services/src/bin/competition.rs) stores
the keys to encrypted off-chain test data and submitted, trained models.
Additionally, the Competition service allows an [attested off-chain enclave](https://en.wikipedia.org/wiki/Trusted_Computing#Remote_attestation) to receive the secrets and evaluate the models on the test data.
The models and data are decrypted into (encrypted) enclave memory; the evaluation program is sandboxed (though not very completely in this demo) to make it difficult to extract secrets.
If everything works properly, nobody except the evaluation program has access to the raw data.
The evaluation enclave puts the _two_ in _ready-layer-two_.

Also, it's generally useful to have user registration so that a person can, y'know, actually be announced as the winner.
This functionality is in the [UserRegstry service](/services/src/bin/user_registry.rs), which amounts to passing around JWTs.

## Building the demo

To build the demo, you'll need

* the [Oasis SDK](https://docs.oasis.dev/quickstart.html#set-up-the-oasis-sdk) to build and test the platform services
* [Docker](https://www.docker.com/get-started) to train and evaluate the models

## Running the demo

Once you have all of the build tools, simply run `make`.
You should see the following output:

```
✔️     Create UserRegistry
✔️     Register participants
✔️     Upload data
✔️     Create Competition
✔️     Make submissions
✔️     Start evaluation program "enclave"
✔️  🔒 Fetch data encryption keys
✔️  🔒 Evaluate models on test data
✔️  🔒 Announce winner

🎉 PEGASOS has won the competition! 🎉
```
