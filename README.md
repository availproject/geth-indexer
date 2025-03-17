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
| `{full}` | full transactions |

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

## You can also apply pagination and limits, default page_idx is 0 and limit is 10.

```
http://localhost:9090/transactions?latest=true&&chain_id=7890&&page_idx=0&&limit=10
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