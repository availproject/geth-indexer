# geth-indexer

## Metrics

### **GET** /metrics/[Field]?[Identifier]

Gets current performance metrics

__Field__

|Name |Parameter |
|:--- |:---  |
|`{current_tps}` <sup>\*One of</sup> |UTF-8|
|`{transaction_volume}` <sup>\*One of</sup> |UTF-8|
|`{total_transfers}` <sup>\*One of</sup> |UTF-8|
|`{successful_transfers}` <sup>\*One of</sup> |UTF-8|

__Identifier__

| Name |
|:---  |
| `{chain_id}`  <sup>\*optional</sup> |


__Request_Response_Examples__
* Current TPS

    * Request

      `GET /metrics/current_tps`

      ```bash
      # Without chain_id
      curl http://localhost:9090/metrics/current_tps
      ```

       ```bash
      # With chain_id
      curl http://localhost:9090/metrics/current_tps?chain_id=7890
      ```

    * Response

      ```json
      {
        "150"
      }
      ```

* Transaction Volume
    * Request

      `GET /metrics/transaction_volume`

      ```bash
      # Without chain_id
      curl http://localhost:9090/metrics/transaction_volume
      ```

       ```bash
      # With chain_id
      curl http://localhost:9090/metrics/transaction_volume?chain_id=7890
      ```

    * Response

      ```json
        [
            {"successful_txns":1353,"total_txns":1353,"timestamp":"2025-03-16 21:23:59.000 IST"},
            {"successful_txns":2068,"total_txns":2068,"timestamp":"2025-03-16 21:23:59.000 IST"}
        ]
      ```

* Daily Successful transfer

    * Request

      `GET /metrics/successful_transfers`

      ```bash
      # Without chain_id
      curl http://localhost:9090/metrics/successful_transfers
      ```

       ```bash
      # With chain_id
      curl http://localhost:9090/metrics/successful_transfers?chain_id=7890
      ```

    * Response

      ```json
      {
        "200"
      }
      ```

* Daily Total Transfers

    * Request

      `GET /metrics/total_transfers`

      ```bash
      # Without chain_id
      curl http://localhost:9090/metrics/total_transfers
      ```

       ```bash
      # With chain_id
      curl http://localhost:9090/metrics/total_transfers?chain_id=7890
      ```

    * Response

      ```json
      {
        "300"
      }
      ```

## Transactions

### **GET** /transactions?[Identifier]&[Filter]&[Parts]&[Limit]

Returns a maximum `Limit` number of transactions filtered by a combination of `Identifier` and `Filter`(s) and response type specified by `Parts`. 

__Identifier__

| Name |
|:---  |
| `{tx_hash}`  <sup>\*optional</sup> |
| `{latest}` <sup>\*optional</sup> | 

__Filter__

|Name |
|:--- |
|`{chain_id}`  <sup>\*optional</sup> |

__Parts__

| Name | Description |
|:---  |:--- |
| `{summary_only}` | txn summary |
| `{all}` | full transactions |

__Limit__

| Name | Description |
|:---  |:--- |
|`{limit}`  <sup>\*optional</sup>  | Maximum number of records per query. Default value is 25. |

__Response__

| Code | Description | Body |
|:--- |:--- |:--- | 
|200  | OK. | A vector of transactions of length equal to `limit`. |


## Example Usage 

## For querying latest txns

```
http://localhost:9090/transactions?latest=true&&chain_id=7890

```

```
Response: 

[
    {
        "TxnSummary": {
            "hash": "0xfa856fec5709ba379071d75c0756bd05226a25d708d5cbdfd19633559ad40174",
            "signer": "0x9035412c90420b81af3fa4a90a619b7664f954e4",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },
    {
        "TxnSummary": {
            "hash": "0x922b9937e014816967060ecb124b54e47d7e0854c5d288cc6bd57b0fbdfd871e",
            "signer": "0x9a2998f1a8624babd3d885d4f67c90e56639df78",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },...
]
```

## You can also apply limits, default limit is 10.

```
http://localhost:9090/transactions?latest=true&&chain_id=7890&&limit=10
```

```
Response: 

[
    {
        "TxnSummary": {
            "hash": "0xb436bd8ee248cf5fcb0913e3ad30414e9a97ed464257bcac867eb153b1559eff",
            "signer": "0x1b9acf69d6f1f4c6e48070d1fc2f736a4df91555",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },
    {
        "TxnSummary": {
            "hash": "0x59ef3366d9ba579b2df7a93dab1f5ebb1e498b0b69cb6250611684ef15dbcad8",
            "signer": "0xc4fe947d42ef02e7fa674139723dbac2f9d28e84",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },
    {
        "TxnSummary": {
            "hash": "0x93a0e657c81503e9e3851e1dd42c76daae9671d1cb0e0e0ebea161c27b9acdd3",
            "signer": "0xb7004d323788ef312af726a00060d2fa90cfe11c",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },
    {
        "TxnSummary": {
            "hash": "0xfbca51e254f3594b284baf9176502f41529ddb7cd371f69b79dad07a5050e0cd",
            "signer": "0x3daec1f829372c40458612de110a8ae53768eb50",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    },
    {
        "TxnSummary": {
            "hash": "0x1424dbb00cd119d3eb084f0d09d5ae319ed38198bcd32bbbf29eadce0bada696",
            "signer": "0xba415ddf22c54f9014da0d3de1d63f4381a1bea4",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438410
        }
    }
]

```

## For querying a certain txn by hash

```
http://localhost:9090/transactions?tx_hash=0x4d977c608d65a0f622870bb5bc23e269fcaee5e6c0ac31e0f49f023e8faf35a3&&chain_id=7890

```
```
Response: 

[
    {
        "TxnSummary": {
            "hash": "0x4d977c608d65a0f622870bb5bc23e269fcaee5e6c0ac31e0f49f023e8faf35a3",
            "signer": "0xc469d6a293b6004ab48b903c9ab274cc2c5f156c",
            "status": 1,
            "value": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "block_height": 438378
        }
    }
]

```

## In case you are looking to query full transactions 

```
http://localhost:9090/transactions?latest=true&&chain_id=7890&&all=true&&limit=2
```

```
Response 

[
    {
        "Transaction": {
            "hash": "0x2895ec1eea1ffaa39364f0070cade1eb5626717124bb4304c31d82193ffb45c6",
            "nonce": "0xf3",
            "blockHash": "0x9c0bc9ab9b9fb7f226b7bbbf828bdca07ea6c0c9643eecb26175805caf7587ab",
            "blockNumber": "0x6b08c",
            "transactionIndex": "0x0",
            "from": "0xb7004d323788ef312af726a00060d2fa90cfe11c",
            "to": "0x853b73e7e3ccb992d67d3809a6fb3bae4361aff1",
            "value": "0x1",
            "gasPrice": "0x0",
            "gas": "0x1c9c380",
            "input": "0x",
            "r": "0x766ca754a88427fe57f7d1524377de0a8318c9e6990a806da815099cd92bf2d8",
            "s": "0x1201163b060b5dec83e617304729f2bc3a86662dde66723082220b136132ec1d",
            "v": "0x3dc8",
            "chainId": "0x1ed2",
            "accessList": []
        }
    },
    {
        "Transaction": {
            "hash": "0x927ed96d6a88bc7814fd1ed402fa87b8c1d787a37c4a242abbd3ffd99f757f6b",
            "nonce": "0xeb",
            "blockHash": "0x9c0bc9ab9b9fb7f226b7bbbf828bdca07ea6c0c9643eecb26175805caf7587ab",
            "blockNumber": "0x6b08c",
            "transactionIndex": "0x1",
            "from": "0x9035412c90420b81af3fa4a90a619b7664f954e4",
            "to": "0x853b73e7e3ccb992d67d3809a6fb3bae4361aff1",
            "value": "0x1",
            "gasPrice": "0x0",
            "gas": "0x1c9c380",
            "input": "0x",
            "r": "0x27aab4a803a4ca60f59e235d2abc5f02e6b030386a0cf008c545a78c51368005",
            "s": "0x6e52cc1e92100c2f01a9b9ab9eaf345fb40d9bbc4acc017ca7e2d298d60e289a",
            "v": "0x3dc8",
            "chainId": "0x1ed2",
            "accessList": []
        }
    }
]
```
Commit-by:- @RSH
