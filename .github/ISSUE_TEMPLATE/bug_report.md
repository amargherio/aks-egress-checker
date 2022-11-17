---
name: Bug report
about: Problems and issues with code or documentation
title: ''
labels: bug
assignees: ''
---

<!--

Thank you for wanting to open a bug report for this egress checker! I'm sorry 
we had to meet this way, but with the details you provide here I'm confident
I can help turn this experience around!

Some of the fields and information below may not be relevant to your situation
or you may not have all of the information I'm asking for - that's OK! Every
little bit helps make fixing bugs easier.

If you need to include code snippets or logs, please put them in fenced code
blocks.  If they're super-long, please use the details tag like
<details><summary>super-long log</summary> lots of stuff </details>

-->

**What happened**:

<!-- (please include exact error messages if you can) -->

**What you expected to happen**:

<!-- What do you think went wrong? -->


**Egress Checker Image Version** (Describe the pod in the cluster and find the image URL in the output):
<!--
kubectl describe po -n <pod-namespace> egress-checker
-->

**AKS version** (use `kubectl version`):

**Environment**:

- **Node image version**:
- **Basic cluster related info**:
  - `kubectl version`
  - `kubectl get nodes -o wide`
  - UDR or load balancer outbound type?
  - Kubenet or Azure CNI?

- **Others**:
  - Any other related information that may help


**How to reproduce this issue**:
<!---

As minimally and precisely as possible. Keep in mind we do not have access to your cluster or application.

-->

**Anything else we need to know**: