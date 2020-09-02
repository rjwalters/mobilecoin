## mobilecoind

The MobileCoin Daemon, or `mobilecoind`, is a standalone executable which provides blockchain synchronization and wallet services.

It creates encrypted, attested connections to validator nodes to download blockchain data and to submit new transactions. It accepts connections from client software like [`mobilecoin-wallet`](todo-link) and [`mobilecoind-json`](todo-link).

To keep the blockchain in sync, `mobilecoind` downloads new blocks from cloud storage and checks with a quorum of validator nodes that the block headers are correct. If this succeeds, the new blocks are added to a local copy of the blockchain called the Ledger Database (*ledger-db*). This is done periodically to ensure the transaction outputs (*txos*) recorded in *ledger-db* can be used to calculate accurate balances and to generate valid new transactions.

Clients interact with `mobilecoind` using a gRPC service [API](./api/proto/mobilecoind_api.proto) defined with protocol buffers. Client software that maintains a wallet can use the API to create a *monitor* within `mobilecoind` that is provisioned with private keys. The *monitor* automatically maintains a list of unspent transaction outputs (*utxos*) controlled by its private keys in the MobileCoin Daemon Database (*mobilecoind-db*).

### Table of Contents

  - [Compiling `mobilecoind`](#compiling-mobilecoind)
  - [Running `mobilecoind`](#running-mobilecoind)
  - [Service API](#mobilecoind-api)
  - [Offline Use](#offline-use)

### Compiling `mobilecoind`

The MobileCoin Daemon can be compiled from source using `cargo`. A typical command to compile `mobilecoind` for production use, with reduced log output and release optimizations turned on, is:

```
SGX_MODE=HW \
IAS_MODE=PROD \
CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css \
cargo build --release -p mc-mobilecoind
```

To help protect user privacy, `mobilecoind` evaluates SGX remote attestation evidence before submitting a new transaction proposal to a remote server. Two environment variables are required at compile time (`SGX_MODE` and `IAS_MODE`) to select between simulated remote attestation and production attestation code.

A third environment variable, `CONSENSUS_ENCLAVE_CSS`, specifies the location of a binary file that provides signature artifacts for remote attestation. This information is used to whitelist the consensus enclaves that `mobilecoind` will recognize as valid.

A file containing the location information for the most recent MobileCoin TestNet signature artifact file is available via

```
curl -O https://enclave-distribution.test.mobilecoin.com/production.json
```

This retrieves a json record of:

```json
{
    "enclave": "pool/<git revision>/<signing hash>/<filename>",
    "sigstruct": "pool/<git revision>/<signing hash>/<filename>",
}
```

The `sigstruct` field provides the relative path to the binary signature artifacts css file.

As an example, the signature artifact file for an early release of MobileCoin's TestNet is available via:

```
curl -O https://enclave-distribution.test.mobilecoin.com/pool/bceca6256b2ad9a6ccc1b88c109687365677f0c9/bf7fa957a6a94acb588851bc8767eca5776c79f4fc2aa6bcb99312c3c386c/consensus-enclave.css
```

You can collect the most recent signature artifacts css file directly by nesting calls to `curl` and parsing the downloaded json object with `jq`:

```
curl -O https://enclave-distribution.test.mobilecoin.com/$(curl https://enclave-distribution.test.mobilecoin.com/production.json | jq -r '.sigstruct')
```

Once you fetch the desired binary signature artifacts file, you must provide the file path when compiling `mobilecoind` via an environment variable:

```
CONSENSUS_ENCLAVE_CSS=path/to/consensus-enclave.css
```

### Running `mobilecoind`

The MobileCoin Daemon can be launched using `cargo`. A typical command to start `mobilecoind` in the background, while capturing log output to a file, is:

```
MC_LOG=debug,rustls=warn,hyper=warn,tokio_reactor=warn,mio=warn,want=warn,rusoto_core=error,h2=error,reqwest=error \
cargo run --release -p mc-mobilecoind -- \
  --ledger-db /path/to/ledger-db \
  --mobilecoind-db /path/to/mobilecoind-db \
  --watcher-db /path/to/watcher-db \
  --peer mc://node1.test.mobilecoin.com/ \
  --peer mc://node2.test.mobilecoin.com/ \
  --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
  --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/ \
  --service-port 4444 \
  --poll-interval 1 \
  2>&1 > /path/to/mobilecoind.log &
```

The `MC_LOG` environment variable controls the verbosity of the messages that `mobilecoind` sends to `stdout` while running. These messages might be combined with output to `stderr` and redirected to a file on disk while `mobilecoind` runs in the background, as shown in the example above.

The runtime arguments are discussed below. Some additional details are also available by passing the `--help` argument to `mobilecoind` via `cargo`:

```
cargo run --release -p mc-mobilecoind -- --help
```

##### Local Storage

You must provide file paths for the local databases used by `mobilecoind`, such as:

```
  --ledger-db /path/to/ledger-db \
  --mobilecoind-db /path/to/mobilecoind-db \
  --watcher-db /path/to/watcher-db \
```

The Ledger Database (*ledger-db*) stores a local copy of the MobileCoin blockchain. Rather than confirm old block data, the MobileCoin Daemon always assumes that *ledger-db* contains a reliable and contiguous copy of the MobileCoin blockchain. After loading the *ledger-db* from disk, `mobilecoind` will start trying to download, confirm, and append new blocks.

The MobileCoin Daemon Database (*mobilecoind-db*) stores ledger entries that have been discovered by a `mobilecoind` monitor. This database is updated each time a new block is added to the Ledger Database.

A Watcher Database (*watcher-db*) is optional. When `mobilecoind` is launched with a Watcher Database, the host server performs an essential role in the MobileCoin Network as a "watcher node". In addition to the block data, watcher nodes download block signatures for each block, check that the signatures are correct, and help verify that the signatures overlap. This calculation allows watcher nodes to flag any blocks in which a hard fork could have occurred, to exclude the possibility of ledger extension attacks by a colluding set of malicious node operators who control quorum. See the [watcher](../watcher/README.md) crate for more information.

##### Trusted Validator Nodes

You must specify a list of the validator nodes that you will trust to provide correct ledger data (each called a *peer*). All of the validator nodes you choose must agree on new block content (using hashes recorded in block headers) before the local copy of the blockchain in *ledger-db* can be extended. New transactions prepared by a *monitor* are submitted to one of the trusted peers.

We use URIs to specify peers in the arguments passed to `mobilecoind` at launch, such as:

```
--peer mc://node1.test.mobilecoin.com/
```

##### Cloud Storage

In order to synchronize *ledger-db* with the MobileCoin Network blockchain, you must also provide a list of cloud storage locations (each called a "transaction source") where full validator nodes have published their externalized blocks.

We use URLs to specify the transaction sources in the arguments passed to `mobilecoind` at launch, such as:

```
--tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/
```

When `mobilecoind` is provided with the path to a Watcher Database (`--watcher-db`), block signatures are also downloaded from each cloud storage location. These block signatures are created within the secure enclaves running on each full validator node. The private key used to create block signatures is unique to each enclave invocation. By raising an alarm if a new block does not share any signing enclaves with its immediate predecessor block, watcher nodes help the MobileCoin community identify places where hard forks in the blockchain are possible.

##### Other Arguments

Clients may connect to the `mobilecoind` gRPC service API on the port provided by the `--service-port` argument. The default service port value is 4444.

The MobileCoin Daemon polls the provided peer URIs and cloud storage URLs for new blockchain data with a period provided by the `--polling-interval` argument in seconds. New block data is expected to arrive approximately once every 5 seconds. When `mobilecoind` detects that it is behind, it will download data as fast as possible to catch up with the MobileCoin Network.


### Service API

The MobileCoin Daemon provides a gRPC service API using protocol buffers. An equivalent REST API is exposed when additionally running [`mobilecoind-json`](todo-link) as a proxy service.

The functions available in the `mobilecoind` service API include:

##### Monitors

|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Add a new monitor|`AddMonitor`|`POST /monitors `|
|Remove a monitor and its data|`RemoveMonitor`|`DELETE /monitors/{monitor}`|
|Get a list of all known monitors|`GetMonitorList`|`GET /monitors `|
|Get the status of a monitor|`GetMonitorStatus`|`GET /monitors/{monitor} `|
|Get the public address for a subadddress|`GetPublicAddress`|`GET /monitors/{monitor}/subaddresses/{subaddress}/public-address `|
|Get the UnspentTxOuts for a subadddress|`GetUnspentTxOutList`|`GET /monitors/{monitor}/subaddresses/{subaddress}/utxos `|
|Get the balance for a subadddress|`GetBalance`|`GET /monitors/{monitor}/subaddresses/{subaddress}/balance `|
|Get the processed block content for a monitor|`GetProcessedBlock`|`GET /monitors/{monitor}/processed-blocks/{block} `|


##### Entropy and Keys
|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Generate a new random account entropy value|`GenerateEntropy`|`POST /entropy `|
|Calculate an account key for an entropy|`GetAccountKey`|`GET /entropy/{entropy} `|


##### Transcribed Base-58 Codes
|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Decode a payment Request code|`ReadRequestCode`|`GET /codes/request/{code} `|
|Encode a payment Request code|`GetRequestCode`|`POST /codes/request `|
|Decode a self-payment Transfer code|`ReadTransferCode`|`GET /codes/transfer/{code} `|
|Encode a self-payment Transfer code|`GetTransferCode`|`POST /codes/transfer `|
|Decode an Address Request code|`ReadAddressRequestCode`|`GET /codes/address-request/{code} `|
|Encode an Address Request code|`GetAddressRequestCode`|`POST /codes/address-request `|
|Decode an Address Response code|`ReadAddressResponseCode`|`GET /codes/address-response/{code} `|
|Encode an Address Response code|`GetAddressResponseCode`|`POST /codes/address-response `|


##### Transactions
|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Generate a transaction proposal|`GenerateTx`|`POST /tx/build `|
|Generate a tx proposal used to optimize a wallet |`GenerateOptimizationTx`|`POST /tx/build-for-optimization `|
|Generate a tx proposal used to fund a MobileCoin Transfer code|`GenerateTransferCodeTx`|`POST /tx/build-for-transfer-code `|
|Get the status for a submitted tx using the key image|`GetTxStatusAsSender`|`POST /tx/status-as-sender `|
|Get the status for a submitted tx using the tx pubkey|`GetTxStatusAsReceiver`|`POST /tx/status-as-receiver `|


##### Sending Payments
|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Build and submit a simple payment|`SendPayment`|`POST /send-payment `|
|Submit a tx proposal to the network|`SubmitTx`|`POST /consensus/submit-tx `|


##### Ledger
|Description | gRPC API | REST API |
| ---- | ---- | ---- |
|Get information for the local ledger|`GetLedgerInfo`|`GET /ledger/local `|
|Get information for the network ledger|`GetNetworkStatus`|`GET /ledger/network `|
|Get header content for a block|`GetBlockInfo`|`GET /ledger/blocks/{block}/header `|
|Get all raw content for a block|`GetBlock`|`GET /ledger/blocks/{block} `|


### Offline Use

Offline transactions are a way of constructing a transaction on a machine that is not connected to the Internet, allowing for increased safety around the storage of sensitive key material. The requirements for doing that are:

1. A machine that is connected to the internet, running `mobilecoind` as usual.
1. A second machine, not connected to the internet, that has a recent copy of ledger. The ledger must contain some spendable TxOuts by the key that will be used.

The steps for constructing and submitting an offline transaction are:

1. Copy a recent copy of the ledger database into the airgapped machine. The copied ledger should include TxOuts that are spendable by the user.
1. Run `mobilecoind` on the airgapped machine: `MC_LOG=trace CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css SGX_MODE=SW IAS_MODE=DEV cargo run -p mc-mobilecoind --release -- --offline --listen-uri insecure-mobilecoind://127.0.0.1:4444/ --mobilecoind-db /tmp/wallet-db`. Note that a CSS file is still needed since its impossible to build mobilecoind without one, unless you are able to compile the enclave.
1. Connect to this `mobilecoind`, add a monitor with your keys, let it scan the ledger, and construct a transaction using the `GenerateTx` API call.
1. `GenerateTx` will return a `TxProposal` message, which you can then protobuf-encode into a blob of bytes.
1. Copy this blob of bytes into a machine that has internet access and `mobilecoind` running.
1. Decode the blob of bytes back into a `TxProposaland` submit it using the `SubmitTx` API call. Even if the `mobilecoind` instance you are submitting to has no monitors defined at all, this would still work.
