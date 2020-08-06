# Oracle Core

![](images/oracle-core.png)

The oracle core is the off-chain component that oracles who are part of an oracle pool run. This oracle core provides a HTTP API interface for reading the current protocol state & another for submitting datapoints. Once a datapoint is submit the oracle core will automatically generate the required tx and post it as well as any other actions required for the protocol to run. This thereby allows the oracle to participate in the oracle pool protocol without any extra effort for the oracle operator.

The oracle core requires that the user has access to a full node wallet in order to create txs & perform UTXO-set scanning. Furthermore each oracle core is designed to work with only a single oracle pool. If an operator runs several oracles in several oracle pools then a single full node can be used, but several instances of oracle cores must be run (and set with different api ports).

A `connector` must also be used with the oracle core in order to acquire data to submit to the pool. Each connector sources data from the expected sources, potentially applies functions to said data, and then submits the data to the oracle core via HTTP API during the `Live Epoch` stage in the oracle pool protocol. All oracles for a given pool are expected to use the exact same connector, thereby making it simple to onboard and get started.

The current oracle core is built to run the protocol specified in the [Basic Oracle Pool Spec](https://github.com/ergoplatform/oracle-core/blob/master/docs/specs/Basic-Oracle-Pool-Spec.md). Future versions will also support stake slashing and governance, with the specs already available for reading in the [specs folder](docs/specs).

Many other documents can also be found explaining how various parts of the oracle core work in the [docs folder](docs).


# Building & Running An Oracle
The majority of oracle operators will only need to focus on setting up their own oracle core to work with an already bootstrapped oracle pool. This section will explain how to do so for oracle using the ERG-USD adapter. The steps are exactly the same for other adapters, but simply require using that adapter's prepare script.

It is assumed that you are running this oracle on Linux, and have the following prerequisites:
- Access to an [Ergo Node](https://github.com/ergoplatform/ergo) v3.3.0+ with an unlocked wallet.
- A recent stable version of the [Rust Compiler](https://www.rust-lang.org/tools/install) installed.
- The Linux CLI tool `screen` and the `libssl-dev` package on Ubuntu (aka `openssl-devel` on Fedora, and potentially slightly different on other distros) installed.

1. Clone this repository via:
```sh
git clone git@github.com:ergoplatform/oracle-core.git
```
2. Enter into the adapter's script folder:
```sh
cd oracle-core/scripts/erg-usd-oracle
```
3. Run the prepare script which will automatically compile the oracle core and the adapter for you:
```sh
sh prepare-erg-usd-oracle.sh
```
4. Enter into the newly created `oracle-core-deployed` folder:
```sh
cd ../../oracle-core-deployed & ls
```
5. Edit your `oracle-config.yaml` with your Ergo Node information, your oracle address (address that was used for bootstrapping the pool & is inside of your Ergo Node wallet), and other relevant pool information. Read more about the [config file here](docs/Oracle-Config.md). (Optionally acquire a pre-configured `oracle-config.yaml` from the user who bootstrapped the oracle pool and simply fill in your node info/oracle address)
6. Ensure your Ergo Node is running (and matches the info you input in the config) and has it's wallet unlocked (with some Ergs in the wallet to pay for tx fees).
7. Launch your oracle by running `run-oracle.sh`:
```sh
sh run-oracle.sh
```
8. A `screen` instance will be created which launches both the oracle core and the adapter. (Press `Ctrl+a - n` to go between the core & the adapter screens).
9. If your node is running and properly configured, the oracle core will inform you that it has successfully registered the required UTXO-set scans:
```sh
To Do...
```
10. Press enter to confirm that the scans have been registered. Your oracle core is now properly set up and waiting for the UTXO-set scans to be triggered in order to read the state of the oracle pool on-chain to then perform actions/txs.
11. Rescan the blockchain history by deleting `.ergo/wallet/registry` in your Ergo Node folder.
12. Once the node has finished rescanning, the oracle core & adapter will automatically issue transactions and move the protocol forward.
13. Congrats, you can now detach from the screen instance if you wish via `Ctrl+a d`. (And reattach via `screen -r`)


# Bootstrapping An Oracle Pool
In order for an oracle pool to run, it must be first created/bootstrapped on-chain. This is the bootstrap process that is required before oracle operators can run their oracle core and have the pool function on-chain.

1. Generate a new NFT for your new Oracle Pool.
2. Generate a new "Oracle Pool Participant Token" of total count equal to the number of oracles who will be a part of the pool.
3. Decide on and set the key parameters of the pool which will be hard-coded into the contracts. (ie. Epoch Prep Length, Live Epoch Length, Payout Amount, etc.)
4. Compile the smart contracts using the parameters/token ids to acquire the smart contract/stage addresses.
5. Fill out a `oracle-config.yaml` with all of the parameters of the oracle pool which were set, as well as with the newly generated smart contract addresses.
6. Send the `oracle-config.yaml` to all of the oracles part of the pool.
7. Wait until all of the oracles launch the oracle core so that their nodes can register the proper UTXO-set scans.
8. Bootstrap the oracle pool box at the "Epoch Preparation" stage/contract, and holding the "Oracle Pool NFT".
9. Acquire the addresses of all of the oracles.
10. Bootstrap a "Datapoint" box for every single one of the oracles with their corresponding addresses held in R4, and with a single "Oracle Pool Participant Token".