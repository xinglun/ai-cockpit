# Versioning

Runtime version and Repository Protocol version are independent.

```text
ai-cockpit 2.3.5
supports repositoryProtocol = 1

repository:
protocol_version = 1
```

A Runtime upgrade may add capabilities while continuing to support Protocol 1.
Only a Protocol 1 → Protocol 2 change is a repository migration. Runtime startup
must report both versions and the Runtime digest.

Protocol compatibility is explicit: an unsupported major protocol is Red, while
an optional capability absent from the current Runtime is Yellow with a safe
action. Historical Work Items retain the Project Profile digest and protocol
version that were used at their decision boundary.

