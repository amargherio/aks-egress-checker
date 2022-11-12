# AKS network egress checker

## What it does


## Egress support
| Egress Group                  | Network/Application?  | Required or optional? | Check status | All egress checked? |
|-------------------------------|-----------------------|-----------------------|--------------|---------------------|
| Azure Global                  | Network               | Required              | Enabled      | Partial             |
| Azure Global                  | Application           | Required              | Enabled      | Partial             |
| Azure Global                  | Application           | Optional              | Enabled      | Full coverage       |
| Azure China 21Vianet          | Network               | Optional              | Disabled     | Partial             |
| Azure China 21Vianet          | Application           | Optional              | Disabled     | Partial             |
| Azure US Government           | Network               | Optional              | Enabled      | Partial             |
| Azure US Government           | Application           | Optional              | Enabled      | Partial             |
| AKS Node OS updates           | Application           | Optional              | Enabled      | Partial             |
| GPU-enabled clusters          | Application           | Optional              | Enabled      | Full coverage       |
| Windows Server                | Application           | Optional              | Enabled      | Partial             |
| Microsoft Defender            | Application           | Optional              | Enabled      | Partial             |
| Azure Monitor                 | Network               | Optional              | Enabled      | Partial             |
| Azure Monitor                 | Application           | Optional              | Enabled      | Partial             |
| Azure Policy                  | Application           | Optional              | Enabled      | Partial             |
| Azure Policy 21Vianet         | Application           | Optional              | Enabled      | Partial             |
| Azure Policy US Gov           | Application           | Optional              | Enabled      | Partial             |
| AKS Cluster Extensions        | Application           | Optional              | Enabled      | Partial             |
| AKS Cluster Extensions US Gov | Application           | Optional              | Enabled      | Partial             |