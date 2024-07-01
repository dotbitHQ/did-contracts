# Deployted Scripts

> You should not rely on the outpoint of a contract if its hash_type is "type" because when the contract is upgraded, the outpoint will change. You should query the latest outpoint using the script's code_hash from explorer.

## Mainnet

### did-cluster

```json
{
    "cluster_id": "0xcff856f49d7a01d48c6a167b5f1bf974d31c375548eea3cf63145a233929f938",
    "outpoint": {
        "tx_hash": "0x3831cfcebe885d506b221a046803c6af6e09b76a9a70d1a6bdb57bf2e93a58f0",
        "index": 0
    }
}

```

### did-cell-type

```json
{
    "code_hash": "0xcfba73b58b6f30e70caed8a999748781b164ef9a1e218424a6fb55ebf641cb33",
    "hash_type": "type",
    "outpoint": {
        "tx_hash": "0xeda235b56aa422c497b9b9bcfde7af289376e7fe12449ade99176609005994ba",
        "index": 0 
    }
}

```

## Testnet

### did-cluster

```json
{
    "cluster_id": "0x38ab2c230a9f44b4ed7ebb4f7f15a7c9ecf79b3d723a2caf4a8e1b621f61dd71",
    "outpoint": {
        "tx_hash": "0x2066676e9c6cc0d7218b5fbbf721258999f91eb7fbfc43a4ae080a45b54efb27",
        "index": 0
    }
}
```


### did-cell-type

```json
{
    "code_hash": "0x295fa7f597160712f3c07942b544e05cbff45b9068f1158220b6266fbd70ac57",
    "hash_type": "type",
    "outpoint": {
        "tx_hash": "0xf3a12e97b70353b67c774bb10aa34baf00371f5d94150be2c7c39a7692c99b40",
        "index": 0
    }
}

```