# Anti-cyber squatting mechanism



## The main issues

- The transaction data itself is transparent and cannot be encrypted because the nodes processing registration requests are open-source decentralized nodes.
- Before transactions are confirmed on the blockchain, there are various roles that can preview, relay, or intercept transaction content.
- Malicious actors may attempt to register user-desired accounts preemptively upon learning about a user's registration intent.


## Solution

- Conceal the actual account name desired by the user through hashing.
- Require users to provide the hash plaintext, thus addressing the issue of being unable to reverse the plaintext from the hash.
- Set a time gap between providing the hash and providing the plaintext, and mandate that both actions must be completed sequentially. This ensures that even if a third party intercepts the plaintext, they cannot promptly complete both steps in the correct order.
