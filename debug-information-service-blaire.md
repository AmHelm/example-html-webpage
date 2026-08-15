# Blaire - A debug information service for Foreign chain configurations

Status: Draft

## Purpose

This document defines goals and outlines the design of the Foreign chain configurations debug information webservice named Blaire. Blaire will enable the MPC team to easily inspect Foreign chain configuration information.

## Background

Nodes have Foreign chain RPC configurations which are not visible in debug endpoints due to them being potential attack vectors. We would still like to easily access and inspect this information to spot potential configuration bugs. 

## Proposed solution

Blaire will work as a standalone web application that serves Foreign chain RPC configuration debug information (e.g. foreign chain configuration, certain logs etc.) to authenticated users. The current workflow is to ping each node operator manually and separately for the configurations. Through Blaire, the MPC team members will save time and effort by simply requesting the webservice for information relevant to debugging. 

## High level design

The webservice will be accessible to authenticated MPC team members. 

Work flow:
1. MPC nodes will publish their configurations to Blaire at startup and on reconfigurations.
2. Blaire will authenticate, validate and then store the configuration information in a database.
1. Users authenticate themselves to access the webpage.
2. The user will be able to request Blaire for node configurations.
3. Blaire reads its database to serve the information to users.
4. The requests are recorded in an audit log.
5. (Potentially) The user will be able to save/copy/compare the information. 

```mermaid
---
title: Blaire - System Context
---
flowchart TD
    DEV["**MPC Team Member**
      _Selects nodes, compares configurations, copies or downloads results_"]

    AUTH["**Authentication**
      _Verifies session and MPC team membership_"]

    BL["**Blaire**
    _Foreign Chain Debug Information Service_"]

    LOG["**Log**
      _Who requested which nodes, and when_"]

    MPC["**MPC nodes**"]

    DB["**Blaire database**
    _Contains MPC nodes configuration information_"]

    DEV -->|"1. Request configurations for selected nodes"| AUTH
    AUTH -->|"2. Verified request"| BL
    BL -->|"3. Records requests"| LOG
    BL -->|"4. Forwards user request"| DB
    DB -->|"5. Returns requested nodes' configuration information"|BL
    BL -->|"6. Returns requested nodes' configuration information"| DEV
    MPC -->|"Provides configuration information, secrets redacted"| DB

    DEV@{ shape: manual-input}
    AUTH@{ shape: proc}
    BL@{ shape: proc}
    LOG@{ shape: db}
    MPC@{ shape: proc}
    DB@{ shape: db}
```

See [the Foreign chain configurations documentation](https://github.com/near/mpc/blob/0185bf46611aece50a9e876ed8ec0ef96133e421/docs/foreign-chain-transactions.md?plain=1#L631) for a configuration example snippet. 

API keys for authentication will still need to be redacted for security reasons. The nodes will redact these secrets before they are published to Blaire, ensuring that no keys can be leaked in case of a breach.

### Requirements

Required functions:
- Nodes publish redacted Foreign chain configurations   
- Authentication of users (only a select group of team members) before site can be accessed
- Store MPC nodes' Foreign chain configurations
- Users able to request the database for configurations
- Users can see their own request history 

Potential functionalities:
- Download the information as a file/JSON
- The ability to easily copy the information to clip board (button)
- Hand-select several nodes of interest and get all of their configuration information at the same time
- Compare different nodes' configurations
- The MPC node operators having access to Blaire 

## Wire formats/service API

Server endpoint/API root path: 
https://URL (TBD)

### MPC nodes --> Blaire

POST /api/v1/reports     publish config info     config:write   

The reports will be posted through the Blaire API, where the configs are recorded at a node's startup or reconfiguration. The configurations will have a historic record, so that previous configurations could be compared to newer ones.

```rust
async fn publish_node_config_report(
    State(state): State<AppState>,
    node: AuthenticatedNode,
    Json(report): ForeignChainConfig
) -> Result<StatusCode,ApiError> {}
```

### Blaire <--> Users

POST /api/login                             log in authenticated users
POST /api/logout                            log out authenticated users
GET /api/v1/nodes                           list currently participating nodes          nodes:read
GET /api/v1/nodes/{node_id}/config          fetch latest reported config from a node    config:read
GET /api/v1/nodes/{node_id}/history         fetch a node's config history               config:read
GET /api/v1/node?id={node_id}&id={node_id}  compare different node configs              config:read
GET /api/v1/user/audit                      list user actions/requests                  audit:read

```rust
async fn get_node_config(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(node_id): Path<NodeId>
) -> Result<Json<ForeignChainConfig>,ApiError> {}
```

```rust
async fn list_user_audit_log(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<AuditEvent>,ApiError> {}
```


## Data model


### Structs

```rust
pub struct AuthenticatedUser {
    pub username: String,       //username or email, depending on future authentication
}
```

[The NodeId struct will be based on the existing type of the same name in the MPC repository](https://github.com/near/mpc/blob/fb32ae3787e0e445168260591e3e00213b786adc/crates/near-mpc-contract-interface/src/types/tee.rs#L24)
(fix so that other parts in the document use the same names, ie node_id/account_id)
```rust
pub struct NodeId {
    /// Operator account.
    pub account_id: AccountId,
    /// TLS public key used by the node for peer-to-peer communication.
    pub tls_public_key: Ed25519PublicKey,
    /// Full-access Ed25519 public key of the operator account.
    pub account_public_key: Ed25519PublicKey,
}
```

TODO: Link to existing Foreign chain configin the MPC repository
[Foreign chain config struct](https://github.com/near/mpc/blob/b647bcd117ee8fcd09e17ad3a963dbf6078403fa/crates/node-config/src/foreign_chains.rs#L46)
```rust
pub struct ForeignChainConfig {
    pub id: i64,
    pub node_id: String,
    pub config: String, //JSON?
}
```

### Database

The back-end will connect to a database containing some of the following tables. The foreign chain table will contain the configurations of the individual MPC nodes. There will also be a Node and operator mapping table, which connects which operator controls which node. Each node will also have an access key to Blaire, stored in a separate table. Another table will be an audit log, which will record all user events. The audit log is essential for visibility, error handling and security.
(Need to add more motivation of why each table is needed and how they connect with each other and the system)

#### Foreign chain configuration table

| Node ID       | Created at    | Foreign chain config |
| ------------- | ------------- | -------------        |
| Near #1       | date, time    | JSON(config)         |
| Everstake     | date, time    | JSON(config)         |
| ....          | date, time    | JSON(config)         |

```sql
CREATE TABLE node_config_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    fc_config TEXT NOT NULL
);
```

#### Node - operator mapping

| Node ID       | Operator ID   |
| ------------- | ------------- |
| Near #1       | .....         | 
| Everstake     | .....         |
| ....          | .....         |

```sql
CREATE TABLE node_operator_mapping (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL UNIQUE,
    operator_id TEXT NOT NULL 
);
```

#### Audit log

| User ID       | Timestamp     | Event                 |
| ------------- | ------------- | -------------         |
| User #1       | date, time    | Logged in             |
| User #1       | date, time    | Request Node #1 config|
| ....          | date, time    | .....                 |

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  
    user_id TEXT NOT NULL,
    event_timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    event_type TEXT NOT NULL
);
```

#### Node credentials

| Node ID       | Token hash     |
| ------------- | -------------  |
| Near #1       | .......        |
| Everstake     | .......        |
| ....          | .......        |


```sql
CREATE TABLE node_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL    -- for authentication/access
);
```

## Authentication/security

Initially, while in development, the webpage will have an authentications system where there will only be one single user, with a username and password configured in environment variables. Once the webpage is ready for deployment, there should be a stronger authentication system in place (details TBD).

Possible authentication methods:
- Some sort of SSO service, using NEAR One org emails, with the ability to revoke access
- GitHub OAth/integration, where access could build on group permissions 

### Risks

The configuration information used to be public but was withdrawn as an extra precaution. If the debug service were to be hacked and this information is leaked, we heighten the risk to our node system. Therefore, security should still be strong and accessibility limited to only MPC team members. 