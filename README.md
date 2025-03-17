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



## Blocks

### **GET** /blocks?[Identifier]&[Order]&[Limit]

Returns a maximum `Limit` number of blocks filtered by a combination of `Identifier` and order specified by `Order`. 

__Identifier__

| Name |
|:---  |
| `{block_hash}` <sup>\*One of</sup> | 
| `{block_number}` <sup>\*One of</sup> |
| `{latest}` <sup>\*One of |

__Order__

| Name | Description |
|:---  |:--- |
|`{order}` | "asc", "desc"|

__Limit__

| Name | Description |
|:---  |:--- |
| `{limit}` <sup>\*optional</sup> | Maximum number of records per query. Default value is 25. |

__Response__

| Code | Description | Body |
|:--- |:--- |:--- | 
|200  | OK. | A vector of blocks of length `limit`. |

## Transactions

### **GET** /transactions?[Identifier]&[Filter]&[Order]&[Limit]

Returns a maximum `Limit` number of transactions filtered by a combination of `Identifier` and `Filter`(s) and order specified by `Order`. 

__Identifier__

| Name |
|:---  |
| `{tx_hash}`  <sup>\*optional</sup> |
| `{latest}` <sup>\*optional</sup> | 

__Filter__

|Name |
|:--- |
|`{signer}` <sup>\*optional</sup> |
|`{type}` <sup>\*optional</sup> |
|`{chain_id}`  <sup>\*optional</sup> |

__Order__

| Name | Description |
|:---  |:--- |
| `{order}` | "asc", "desc". If `asc`, is passed, return Response ordered by ascending order of `tx_number`. If `desc`, is passed, return Response ordered by descending order of `tx_number`. |

__Limit__

| Name | Description |
|:---  |:--- |
|`{limit}`  <sup>\*optional</sup>  | Maximum number of records per query. Default value is 25. |

__Response__

| Code | Description | Body |
|:--- |:--- |:--- | 
|200  | OK. | A vector of transactions of length equal to `limit`. |


