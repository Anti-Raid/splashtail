# mewld

Fork of ``https://github.com/cheesycod/mewld`` with Anti-Raid specific changes. Note that while changes will be documented on a best-effor basis. It may not be complete especially in webui

Anti-Raid clusterer. Will be open source to help other large bot developers.

## What is a cluster

A cluster is a way to scale large discord bots by breaking them into seperate processes, each with a set of shards. ``mewld`` is an implementation of this system.

## How it works

- Shard count is retrieved using ``Get Gateway Bot``. This is then used to create a ``ClusterMap`` assigning each cluster a set of shards based on ``per_cluster`` in config.yaml. These ``ClusterMap``'s are then used to make a ``InstanceList`` of ``Instance`` structs.

- First instance is then started, once this instance sends ``launch_next``, the next instance is then launched until every cluster is launched, the last cluster should also send a ``launch_next`` once fully ready

- ``mewld`` also handles other tasks such as cluster management via its webui which also comes with (*upcoming*) support for application command permission configs (for private commands that should only be visible to specific roles in a support or staff server)

## Redis

Mewld uses redis for communication with clusters.

**Data Format:**

JSON with the following structure (golang syntax to make updating docs simpler)

```go
Scope     string         `json:"scope"`
Action    string         `json:"action"`
Args      map[string]any `json:"args,omitempty"`
CommandId string         `json:"command_id,omitempty"`
Output    any            `json:"output,omitempty"`
Data      map[string]any `json:"data,omitempty"` // Used in action logs
```

**Operations**

| Syntax      	   | Description 									  | Args                    |
| ------           | ----------- 									  | ----                    |
| launch_next      | Ready to launch next cluster, if available       | id -> cluster ID        |
| rollingrestart   | Rolling restart all clusters, one by one         |                         |
| start            | Starts a cluster with a specific ID              | id -> cluster ID        |
| stop             | Starts a cluster with a specific ID              | id -> cluster ID        |
| statuses         | Gets the statuses of all clusters.               |                         |
| shutdown         | Shuts down the entire bot and lets systemctl start it up again if needed | |
| restartproc      | Shuts down bot with error code so systemctl restarts it automatically |    |
| diag             | The cluster must respond with a ``proc.DiagResponse`` payload. This is used as a diagnostic by the clusterer and may be used in the future for more important actions.      |    |

## Diagnostics

Whenever a diagnostic is sent over the ``$CHANNEL`` channel, the cluster must respond with a ``diag`` (see Operations table) within 10 seconds. **A diagnostic can be identified based on the existence of a ``diag`` key**

## FAQ

- Why are there so many ``PingCheckStop <- i.ClusterID``?

**Answer:** To ensure that no erroneous ping checks are still running in background thus leading to random cluster death.