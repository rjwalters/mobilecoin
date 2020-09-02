## mobilecoind-json

This is a standalone executable which provides a simple HTTP JSON API wrapping the [mobilecoind](../mobilecoind) gRPC API.

It can be run alongside `mobilecoind` to provide HTTP access to all `mobilecoind` functionality.

### Launching

Since `mobilecoind-json` can simply be launched with:

```
cargo run --release
```

Available options are:

- `--listen_host` - hostname for webserver, default `127.0.0.1`
- `--listen_port` - port for webserver, default `9090`
- `--mobilecoind_host` - hostname:port for mobilecoind gRPC, default `127.0.0.1:4444`
- `--use_ssl` - connect to mobilecoind using SSL, default is false

### HTTP API

# TODO: add links from table to examples curl section

| gRPC | HTTP | Description |
| **** | **** | **** |
|`AddMonitor`            | POST /monitors | Add a new monitor.  |
|`RemoveMonitor`         |  | Remove a monitor and all associated data.  |
|`GetMonitorList`        | GET /monitors | List all known monitor ids.  |
|`GetMonitorStatus`      | GET /monitors/{monitor} | Get the status of a specific monitor.  |
|`GetUnspentTxOutList`   |  | Get a list of UnspentTxOuts for a given monitor and subadddress index.  |
|`GenerateEntropy`       | GET /entropy | Generate a new random root entropy value.  |
|`GetAccountKey`         |  | Generate an AccountKey from a 32 byte root entropy value.  |
|`GetPublicAddress`      |  | Get the public address for a given monitor and subadddress index.  |
|`ReadRequestCode`       | GET /request-code/{payload} | Decode a base-58 encoded "MobileCoin Request Code" into receiver/value/memo.  |
|`GetRequestCode`        | POST /monitors/{monitor}/{subaddress}/request-code | Encode receiver/value/memo into a base-58 "MobileCoin Request Code".  |
|`ReadTransferCode`      |  | Decode a base-58 encoded "MobileCoin Transfer Code" into entropy/tx_public_key/memo.  |
|`GetTransferCode`       |  | Encode entropy/tx_public_key/memo into a base-58 "MobileCoin Transfer Code".  |
|`GetAddressRequestCode` | POST /address-request | Encode a URL into a base-58 "MobileCoin Address Request Code".  |
|`GenerateTx`            |  | Generate a transaction proposal object.  |
|`GenerateOptimizationTx`|  | Generate a transaction that merges a few UnspentTxOuts into one, in order to reduce wallet fragmentation.  |
|`GenerateTransferCodeTx`|  | Generate a transaction that can be used for a "MobileCoin Transfer Code" QR.  |
|`SubmitTx`              |  | Submits a transaction to the network.  |
|`GetLedgerInfo`         | GET /ledger-info | Get information about the downloaded ledger.  |
|`GetBlockInfo`          | GET /block-info/{block} | Get information about a downloaded block.  |
|`GetBlock`              | GET /block-details/{block} | Get more detailed information about a downloaded block  |
|`GetTxStatusAsSender`   | POST /check-transfer-status | Get the status of a submitted transaction as the Sender (using the key image).  |
|`GetTxStatusAsReceiver` | POST /check-receiver-transfer-status | Get the status of a submitted transaction as the Recipient (using the tx public key).  |
|`GetProcessedBlock`     | GET /processed-block/{monitor}/{block} | Get the contents of a processed block.  |
|`GetBalance`            | GET /monitors/{monitor}/{subaddress}/balance | Get the balance for a given monitor and subadddress index, in picoMOB.  |
|`SendPayment`           |  | Build and submit a simple payment and return any change to the Sender's subaddress.  |
|`GetNetworkStatus`      |  | Get information about the network.  |
|                        | POST /monitors/{monitor}/{subaddress}/transfer | Transfer funds between two subaddresses. |


add api for mirror, add binance specific utility functions
/// Retreive a single block. #[get("/block/<block_num>")]
/// Retreive processed block information. #[get("/processed-block/<block_num>")]

### Using with `curl`


#### Generate a new master key
```
$ curl localhost:9090/entropy -X POST
{"entropy":"706db549844bc7b5c8328368d4b8276e9aa03a26ac02474d54aa99b7c3369e2e"}
```

#### Generate an account key from entropy
```
$ curl localhost:9090/entropy/706db549844bc7b5c8328368d4b8276e9aa03a26ac02474d54aa99b7c3369e2e

{"view_private_key":"e0d42caf6edd0dc8a762c665ad5682a87e0a7159e60653827be3911af49d2b01","spend_private_key":"e90849e9dcbbb7aa425cfb34ae3978c14e3dfffd18652e7a6a4821cb1557b703"}
```

#### Add a monitor for a key over a range of subaddress indices
```
$ curl localhost:9090/monitors -d '{"account_key": {"view_private_key":"e0d42caf6edd0dc8a762c665ad5682a87e0a7159e60653827be3911af49d2b01","spend_private_key":"e90849e9dcbbb7aa425cfb34ae3978c14e3dfffd18652e7a6a4821cb1557b703"}, "first_subaddress": 0, "num_subaddresses": 10}' -X POST -H 'Content-Type: application/json'

{"monitor_id":"a0cf8b79c9f8d74eb935ab4eeeb771f3809a408ad47246be47cf40315be9876e"}
```

#### Get the status of an existing monitor
```
$ curl localhost:9090/monitors/a0cf8b79c9f8d74eb935ab4eeeb771f3809a408ad47246be47cf40315be9876e

{"first_subaddress":0,"num_subaddresses":10,"first_block":0,"next_block":2068}
```

#### Check the balance for a monitor and subaddress index
```
$ curl localhost:9090/monitors/a0cf8b79c9f8d74eb935ab4eeeb771f3809a408ad47246be47cf40315be9876e/subaddresses/0/balance

{"balance":199999999999990}
```
#### Get the public address for a monitor and subaddress
```
$ curl localhost:9090/monitors/a0cf8b79c9f8d74eb935ab4eeeb771f3809a408ad47246be47cf40315be9876e/subaddresses/0/public-address

{"view_public_key":"543b376e9d5b949dd8694f065d95a98a89e6f17a20c621621a808605d1904324","spend_public_key":"58dba855a885dd535dc5180af443abae67c790b860d5adadb4d6a2ecb71abd28","fog_report_url":"","fog_authority_fingerprint_sig":"","fog_report_id":""}
```

#### Generate a request code from a public address and optional other information
```
$ curl localhost:9090/codes/request -d '{"public_address": {"view_public_key":"543b376e9d5b949dd8694f065d95a98a89e6f17a20c621621a808605d1904324","spend_public_key":"58dba855a885dd535dc5180af443abae67c790b860d5adadb4d6a2ecb71abd28","fog_report_url":"","fog_authority_fingerprint_sig":"","fog_report_id":""}, "amount": 10, "memo": "Please pay me"}'  -X POST -H 'Content-Type: application/json'

{"request_code":"ufTwqVqF2rXmFVBZ1CWWS3ntdajVZGfZ5A2YZqAwhVnaVYrFpS9Z8iAg44CBGDeyjFDX8Hj4W7ZzArBn1xSp9wu8NriqQAogN8fUybKmoWgaz92kT4M7fbjRYKZmoY8"}
```

#### Read all the information in a request code
```
$ curl localhost:9090/codes/request/HUGpTreNKe4ziGAwDNYeW1iayWJgZ4DgiYRk9fw8E7f21PXQRUt4kbFsWBxzcJj12K6atUMuAyRNnwCybw5oJcm6xYXazdZzx4Tc5QuKdFdH2XSuUYM8pgQ1jq2ZBBi

{"receiver":{"view_public_key":"40f884563ff10fb1b37b589036db9abbf1ab7afcf88f17a4ea6ec0077e883263","spend_public_key":"ecf9f2fdb8714afd16446d530cf27f2775d9e356e17a6bba8ad395d16d1bbd45","fog_url":""},"value":"10","memo":"Please pay me"}
```
This JSON can be passed directly to `build-and-submit` or you can change the amount if desired.

#### Build and submit a payment from a monitor/subaddress to a request code
Using the information in the `read-request`, make creates and submits a transaction. If this succeeds, funds will be transferred.
```
$ curl localhost:9090/monitors/fca4ffa1a1b1faf8ad775d0cf020426ba7f161720403a76126bc8e40550d9872/subaddresses/0/build-tx-and-submit -d '{"receiver":{"view_public_key":"40f884563ff10fb1b37b589036db9abbf1ab7afcf88f17a4ea6ec0077e883263","spend_public_key":"ecf9f2fdb8714afd16446d530cf27f2775d9e356e17a6bba8ad395d16d1bbd45","fog_url":""},"value":"10","memo":"Please pay me"}' -X POST -H 'Content-Type: application/json'

{"sender_tx_receipt":{"key_images":["dc8a91dbacad97b59e9709379c279a28b3c35262f6744226d15ee87be6bbf132","7e22679d8e3c14ba9c6c45256902e7af8e82644618e65a4589bab268bfde4b61"],"tombstone":2121}, ,"receiver_tx_receipt_list":[{"recipient":{"view_public_key":"f460626a6cefb0bdfc73bb0c3a9c1a303a858f0b1b4ea59b154a1aa8d927af71","spend_public_key":"6a74da2dc6ff116d9278a30a4f8584e9edf165a22faf04a3ac210f219641a92d","fog_report_url":"","fog_authority_fingerprint_sig":"","fog_report_id":""},"tx_public_key":"7060ad50195686ebba591ccfed18ff9536b729d07a00022a21eb21db7e9a266b","tx_out_hash":"190ec89253bf47a05385b24e5b289a3a31127462aad613da9484f77d03986112","tombstone":2329,"confirmation_number":"190ec89253bf47a05385b24e5b289a3a31127462aad613da9484f77d03986112"}]}
```

This returns receipt information that can be used by the sender to verify their transaction went through and also receipts to give to the receivers
proving that you initiated the transaction

#### Check the status of a transfer with a key image and tombstone block
The return value from `build-tx-and-submit` can be passed directly directly to `status-as-sender`
```
$ curl localhost:9090/tx/status-as-sender -d '{"sender_tx_receipt":{"key_images":["dc8a91dbacad97b59e9709379c279a28b3c35262f6744226d15ee87be6bbf132","7e22679d8e3c14ba9c6c45256902e7af8e82644618e65a4589bab268bfde4b61"],"tombstone":2121}}'  -X POST -H 'Content-Type: application/json'

{"status":"verified"}
```

#### Check the status of a transaction from the receiving side and verify confirmation number
The return value from `transfer` includes a list called `receiver_tx_receipt_list`. The appropriate item in the list can be send to the recipient over
a separate channel (e.g. a secure chat application) and they can use it to verify that they were paid by the sender.
```
$ curl localhost:9090/tx/status-as-receiver -d '{"recipient":{"view_public_key":"f460626a6cefb0bdfc73bb0c3a9c1a303a858f0b1b4ea59b154a1aa8d927af71","spend_public_key":"6a74da2dc6ff116d9278a30a4f8584e9edf165a22faf04a3ac210f219641a92d","fog_report_url":"","fog_authority_fingerprint_sig":"","fog_report_id":""},"tx_public_key":"7060ad50195686ebba591ccfed18ff9536b729d07a00022a21eb21db7e9a266b","tx_out_hash":"190ec89253bf47a05385b24e5b289a3a31127462aad613da9484f77d03986112","tombstone":2329,"confirmation_number":"190ec89253bf47a05385b24e5b289a3a31127462aad613da9484f77d03986112"}' -X POST -H 'Content-Type: application/json'

{"status":"verified"}
```

### Ledger status endpoints

#### Ledger totals
```
$ curl localhost:9090/ledger/local

{"block_count":"2280","txo_count":"16809"}
```

#### Counts for a specific block
```
$ curl localhost:9090/ledger/blocks/1/header

{"key_image_count":"1","txo_count":"3"}
```

#### Details about a specific block
```
$ curl localhost:9090/ledger/blocks/1

{"block_id":"7b06f8d069f7c169a5a2be51b24331394af832b1453d679e0cca502d3b131bf1","version":0,"parent_id":"e498010ee6a19b4ac9313af43d8274c53d54a1bbc275c06374dbe0095872a6ee","index":"1","cumulative_txo_count":"10003","contents_hash":"c0486e70c50055ecb54ca1f2e8b02fabd1b2322dcd2c133710c3e3149359adec"}
```

## Using MobileCoin at an Exchange

Exchanges may be more familiar with HTTP/JSON APIs based on past experience with other popular cryptocurrencies. Running `mobilecoind` and `mobilecoind-json` can provide this interface for exchanges that are not ready to migrate internal software to gRPC.

An exchange will typically run a *full validator node* and use it to submit new transactions to the MobileCoin network. For improved operational security, this *full validator node* should be distinct from the *watcher node* that is provisioned with private key material. The exchange *watcher node* will have spending authority and should not accept any incoming network connections. Instead, the `GetProcessedBlock` function should be used to collect decoded ledger data on the *watcher node*. The processed transaction data may be forwarded to a third server using the *MobileCoin Mirror Wallet* software. The third server can supply "read-only" ledger access to safely manage incoming client requests for transaction information by block.

## Step-by-step example

#### Preparing to service user accounts

1. Start the *watcher node*

The exchange *watcher node* can be configured to connect to any *validator nodes* to submit new transactions, but exchanges will likely run their own *full validator nodes* for best performance.

1. Assign subaddress index values to users

Subaddresses allow an exchange to efficiently manage a large number of user accounts. Each user account should be assigned to a unique subaddress index. The subaddress public address should be treated as a shared secret with the assigned user, so that any deposits made to a particular subaddress can be assumed to come from the assigned user.

1. Add a fully-permissioned monitor for a range of subaddresses to `mobilecoind`

1. Allow the ledger and monitor to synchronize with the MobileCoin Network

1. Setup the *MobileCoin Mirror Wallet*

#### Deposits: Accepting funds from a user

1. Generate a *MobileCoin Payment Request Code*

1. Display the code to the User as a qr code or b58 string

1. Confirm the incoming transaction at the *watcher node*

1. Collect the processed incoming transaction at the *watcher node*

1. Forwarding the processed incoming transaction using *MobileCoin Mirror Wallet*

1. Collect the processed incoming transaction at the read-only mirror server

#### Withdrawals: Sending funds to an external Public Address

1. Collect the user's external public address (TODO: this intersects with the public address b58 & address request qr code)

2. Create a payment from the user's assigned subaddress to the user's registered public address

1. Confirm the outgoing transaction at the *watcher node*

1. Collect the processed outgoing transaction at the *watcher node*

1. Forwarding the processed outgoing transaction using *MobileCoin Mirror Wallet*

1. Collect the processed outgoing transaction at the read-only mirror server



# TODO: match curl commands from below to exchange examples above

Start mobilecoind: <instructions to run mobilecoind>
Start mobilecoind-json: cargo run -p mobilecoind-json
This connects to mobilecoind on the default host/port localhost:4444 and listens on the default port 9090
Add a monitor for your root entropy:
$ curl localhost:9090/monitors -d '{"entropy": "706db549844bc7b5c8328368d4b8276e9aa03a26ac02474d54aa99b7c3369e2e", "first_subaddress": 0, "num_subaddresses": 1000}' -X POST -H 'Content-Type: application/json'
The response would contain your monitor id:
{"monitor_id":"08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3"}
Wait for the monitor to finish syncing. You can check the status of how many blocks were synced:
$ curl localhost:9090/monitors/08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3

{"first_subaddress":0,"num_subaddresses":10,"first_block":0,"next_block":20902}
The monitor is done syncing when next block gets close or is equal to the number of blocks in the ledger. You can check the ledger size:
$ curl localhost:9090/ledger-info

{"block_count":"24512","txo_count":"73639"}
You can check the balance of a given subaddress:
$ curl localhost:9090/monitors/08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3/0/balance

{"balance":"70010000999941"}
You can get the public address of a specific subaddress (in this case address number 3):
$ curl localhost:9090/monitors/08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3/3/request-code -X POST -d '{}' -H 'Content-Type: application/json'

{"request_code":"3CyCrJg39TbWtA6wNW3mquUc9hu3LU3RfJT2tCSZASfzBPJJ6zDyLcbVn3xqqs7zYxNcNBtxWY7G1am7LbVv15zzQMmGbhGgKUFLpFUrXt98AB"}
You can send money to it in 2 steps:
Read the keys from the address:
$ curl localhost:9090/read-request/3CyCrJg39TbWtA6wNW3mquUc9hu3LU3RfJT2tCSZASfzBPJJ6zDyLcbVn3xqqs7zYxNcNBtxWY7G1am7LbVv15zzQMmGbhGgKUFLpFUrXt98AB

{"receiver":{"view_public_key":"3eb7cc019a35e00fbee7e3a27f5c2f2427a3b5b7255268e7243f7deb34504436","spend_public_key":"228329fced7672526f364d58f40109006b33a7ac53cc357cc79f171ed4073d5c","fog_report_url":"","fog_authority_sig":"","fog_report_id":""},"value":"0","memo":""}
Send money (from subaddress 0, to subaddress 3):
$ curl localhost:9090/monitors/08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3/0/transfer -d '{"receiver":{"view_public_key":"3eb7cc019a35e00fbee7e3a27f5c2f2427a3b5b7255268e7243f7deb34504436","spend_public_key":"228329fced7672526f364d58f40109006b33a7ac53cc357cc79f171ed4073d5c","fog_report_url":"","fog_authority_sig":"","fog_report_id":""},"value":"100","memo":""}' -X POST -H 'Content-Type: application/json'

{"sender_tx_receipt":{"key_images":["7e7e16d4e71e1d0b83bd556d3acf3d7ad5021f843d666b16fa17c55a0a10ee1e"],"tombstone":30310},"receiver_tx_receipt_list":[{"recipient":{"view_public_key":"3eb7cc019a35e00fbee7e3a27f5c2f2427a3b5b7255268e7243f7deb34504436","spend_public_key":"228329fced7672526f364d58f40109006b33a7ac53cc357cc79f171ed4073d5c","fog_report_url":"","fog_authority_sig":"","fog_report_id":""},"tx_public_key":"6867fb29cd281a71fceb2730f2251e2a02ef60aee01b66390da176f9180f032e","tx_out_hash":"b5845494c4a90450e0887a1c5bf6b074c910093e4594de1fbb2e7b2529d2f093","tombstone":30310,"confirmation_number":"b5845494c4a90450e0887a1c5bf6b074c910093e4594de1fbb2e7b2529d2f093"}]}
Now that you sent money, you can look at the mobilecoind logs and see in which block the transaction went in:
2020-07-21 23:54:07.649229763 UTC INFO Processed 2 utxos and 1 key images in block 30261 for monitor id 08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3, mc.app: mobilecoind, mc.module: mc_mobilecoind::database, mc.src: mobilecoind/src/database.rs:232
You can now use the new processed block API to get information about this block:
$ curl http://127.0.0.1:9090/processed-block/08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3/30261 | python -mjson.tool
 {
	"tx_outs": [
    	{
        	"direction": "received",
        	"key_image": "40e0432ba9c5de24becf58c04b855213300d95fad3c48c2d75de7a3512153d5a",
        	"monitor_id": "08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3",
        	"public_key": "6867fb29cd281a71fceb2730f2251e2a02ef60aee01b66390da176f9180f032e",
        	"subaddress_index": 3,
        	"value": "100"
    	},
    	{
        	"direction": "received",
        	"key_image": "d6716d7c4f038a847b2f106eed62c0ce59c2e0eecfcf1d1da473bd26e9864d58",
        	"monitor_id": "08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3",
        	"public_key": "58292cdd7f2d7c3caf885d9bbeca69f17d2e15fe781fc31eafbdb9506433560d",
        	"subaddress_index": 0,
        	"value": "999999999890"
    	},
    	{
        	"direction": "spent",
        	"key_image": "7e7e16d4e71e1d0b83bd556d3acf3d7ad5021f843d666b16fa17c55a0a10ee1e",
        	"monitor_id": "08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3",
        	"public_key": "72c6efe83c43a742092cbc296740b6c07527804b72959fde135a6310651ec819",
        	"subaddress_index": 0,
        	"value": "1000000000000"
    	}
	]
}
There are 3 transactions in the block that belong to monitor 08b4e048afc793213fae60d6ad69a5cb73e43a0d1ebba1cdaaf008a912acf1c3:
The one that went to subaddress 3 with value 100
The change transaction that went to subaddress 0 (since the UTXO spent was higher than what was being sent)
The spent transaction that was sent

From this information, as well as the information we had when we sent the transaction, we can construct the following table:





Sender
Recipient
Amount
TXO Public Key (unique identifier)
Direction


User assigned to Subaddress 3
Self
100
6867fb29
Received


User assigned to Subaddress 0 (master account, only assigned for Self. This is a change txo)
Self
999999999890
58292cd
Received


Self, from subaddress 0
Public Address: (View: 3eb7cc0
Spend: 228329)
100
72c6efe
Sent



