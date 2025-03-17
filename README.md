# geth-indexer

## Metrics

### **GET** /metrics/[Field]?[Identifier]

Gets current performance metrics

__Field__

|Name |Parameter |
|:--- |:---  |
|`{current_tps}` <sup>\*One of</sup> |UTF-8|
|`{transaction_volume}` <sup>\*One of</sup> |UTF-8|
|`{total_transactions}` <sup>\*One of</sup> |UTF-8|
|`{successful_transfers}` <sup>\*One of</sup> |UTF-8|

__Identifier__

| Name |
|:---  |
| `{chain_id}`  <sup>\*optional</sup> |


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


