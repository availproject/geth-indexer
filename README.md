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


