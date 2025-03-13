# geth-indexer

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
|`{order}` | "asc", "desc". If `asc`, is passed, return Response ordered by ascending order of `block_number`. If `desc`, is passed, return Response ordered by descending order of `block_number`. |

__Limit__

| Name | Description |
|:---  |:--- |
| `{limit}` <sup>\*optional</sup> | Maximum number of records per query. For `block_number`, the query returns `limit` number of blocks, going back from the current height while for an identifier like `block_hash` or `tx_hash` or `latest`, the result will always be unique irrespective of what limit is set.  `limit` accepts any number less than 25. Default value is 25. |

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
| `{tx_number}`  <sup>\*optional</sup> |
| `{latest}` <sup>\*optional</sup> | 

__Filter__

|Name |
|:--- |
|`{signer}` <sup>\*optional</sup> |
|`{recipient}` <sup>\*optional</sup> |
|`{type}` <sup>\*optional</sup> |
|`{chain_id}`  <sup>\*optional</sup> |

__Order__

| Name | Description |
|:---  |:--- |
| `{order}` | "asc", "desc". If `asc`, is passed, return Response ordered by ascending order of `tx_number`. If `desc`, is passed, return Response ordered by descending order of `tx_number`. |

__Limit__

| Name | Description |
|:---  |:--- |
|`{limit}`  <sup>\*optional</sup>  | Maximum number of records per query. For `tx_number`, the query returns `limit` number of transactions, going back from the current record while for an identifier like `tx_hash` or `latest` or a set of `filters`, or a combination of `identifiers` and `filters` defined above, the result will always return a unique number of transactions identified. `limit` accepts any number less than 25. Default value is 25. |

__Response__

| Code | Description | Body |
|:--- |:--- |:--- | 
|200  | OK. | A vector of transactions of length equal to `limit`. |

## Metrics

### **GET** /metrics/[Field]?[Identifier]

Gets current performance metrics

__Field__

|Name |Parameter |
|:--- |:---  |
|`{average_tps}` <sup>\*One of</sup> |UTF-8|

__Identifier__

| Name |
|:---  |
| `{chain_id}`  <sup>\*optional</sup> |
