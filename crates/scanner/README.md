# Scanner

## Token Balance

Sometime we need to get balance of giving block number. so if we call contracts mothed is means the states on least block number.

How to solve this problem? We can use eth_call for it.

### **Eth_call**

is an Ethereum API method that executes a new message call immediately without creating a transaction on the blockchain.
There are two main types of eth_call: read contract calls and write contract calls.

- Parameters:
  - object — the transaction call object with:
  - from — (optional) the string of the address, the transaction is sent from
  - to — the string of the address, the transaction is directed to
  - gas — (optional) the integer of the gas provided for the transaction execution
  - gasPrice — (optional) the integer of the gas price used for each paid gas, encoded as hexadecimal
  - value — (optional) the integer of the value sent with this transaction, encoded as hexadecimal
  - data — (optional) the string of the hash of the method signature and encoded parameters, see the Ethereum Contract ABI (opens new window).
  - quantity or tag — the integer block number, or the string with:
    - latest — the latest block that is to be validated. The Beacon Chain may reorg and the latest block can become orphaned
    - safe — the block that is equal to the tip of the chain and is very unlikely to be orphaned
    - finalized — the block that is accepted by two-thirds of the Ethereum validators
    - earliest — the genesis block
    - pending — the pending state and transactions block
- Returns:
  - data – the return value of the executed contract
