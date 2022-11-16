# AKS network egress checker

## What it does
This application loads in a set of [required egress definitions for AKS](https://learn.microsoft.com/en-us/azure/aks/limit-egress-traffic) and tests egress connectivity to them from
within your cluster!

Egress connectivity is a common support topic for AKS - especially when the network path has route tables, default routes,
network security groups, and firewalls or NVAs in play. It's easy to have an oversight where rules are changed in
one place but the team responsible for managing AKS clusters isn't aware of the change.

By setting up the YAML with the correct parameters and a list of egress groups to test for, you can continually 
validate that your cluster is in compliance with the required egress for a given configuration.

## Building from source
I provide some pre-built container images that can be used for deployments. If you'd rather build the image with less
dependencies or a different base (Ubuntu Jammy is where it's at here), you can clone this repo, change the images used
in the Dockerfile (including the Rust builder image if you're adventurous) and build new images to push as you please!

The current supported architecture for the Rust binary is linux_amd64 but building multi-arch binaries is on the roadmap
at some point (along with the correct container images for non-x86_64 architectures).

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

## Interested in contributing?
See the [contributor's guide](./CONTRIBUTING.md) for information!