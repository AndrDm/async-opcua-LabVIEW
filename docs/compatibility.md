# OPC UA Feature Support  

## OPC UA Binary Transport Protocol

This implementation supports the `opc.tcp://` binary protocol. Binary over `https://` is not supported although it is conceivable that it could be supported.

The implement will **never** implement OPC UA over XML. XML hasn't see much adoption so this is no great impediment.

## Server

The server shall implement the OPC UA capabilities:

* http://opcfoundation.org/UA-Profile/Server/Behaviour - base server profile
* http://opcfoundation.org/UA-Profile/Server/EmbeddedUA - embedded UA profile

### Services

The following services are supported in the server:

* Discovery service set
  * GetEndpoints
  * FindServers - stub that only ever returns the current server URI.
  * FindServersOnNetwork - stub that returns BadServiceUnsupported
  * RegisterServer - stub that returns BadServiceUnsupported
  * RegisterServer2 - stub that returns BadServiceUnsupported

* SecureChannel service set
  * OpenSecureChannel
  * CloseSecureChannel

* Attribute service set
  * Read
  * Write
  * History Read
  * History Update

* Session service set
  * CreateSession
  * ActivateSession
  * CloseSession
  * Cancel
  
* Node Management service set
  * AddNodes
  * AddReferences
  * DeleteNodes
  * DeleteReferences
  
* Query service set
  * QueryFirst - not implemented in any node manager, but the framework exists.
  * QueryNext - not implemented in any node manager, but the framework exists.

* View service set
  * Browse
  * BrowseNext
  * TranslateBrowsePathsToNodeIds
  * RegisterNodes
  * UnregisterNodes

* MonitoredItem service set
  * CreateMonitoredItems 
    - Data change filter including dead band filtering.
    - Event filter
  * ModifyMonitoredItems
  * SetMonitoringMode
  * SetTriggering
  * DeleteMonitoredItems

* Subscription service set
  * CreateSubscription
  * ModifySubscription
  * DeleteSubscriptions
  * TransferSubscriptions
  * Publish
  * Republish
  * SetPublishingMode

* Method service set
  * Call

### Address Space / Nodeset

The standard OPC UA address space is exposed through the `CoreNodeManager` implementation. OPC UA for Rust uses a script to generate code to create and populate the standard address space. This functionality is controlled by a server build feature `generated-address-space` that defaults to on but can be disabled if the full address space is not required. When disabled, the address space will be empty apart from some root objects.

### Current limitations

Currently the following are not supported

* Multiple created sessions in a single transport.
  * This should now technically be supported, but without any client that support it is not tested at all.

## Client

The client API is asynchronous, but require you to "drive" the connection by polling an event loop. Convenience methods are provided for polling the event loop on a background thread.

The client exposes functions that correspond to the current server supported profile, i.e. look above at the server services and there will be client-side functions that are analogous to those services.

The client is only automatically tested against the server implementation, so primarily only services supported by the current server implementation are supported. The implementation aims to contain all services, tested against other servers where necessary.

## Configuration

Server and client can be configured programmatically via a builder or by configuration file. See 
the `samples/` folder for examples of client and server side configuration. 

The config files are specified in YAML but this is controlled via serde so the format is not hard-coded.

## Encryption modes

Server and client support endpoints with the standard message security modes:

* None
* Sign
* SignAndEncrypt

The following security policies are supported:

* None
* Basic128Rsa15
* Basic256
* Basic256Rsa256
* Aes128-Sha256-RsaOaep
* Aes256-Sha256-RsaPss

## User identities

The server and client support the following user identity tokens

1. Anonymous - i.e. no identity
2. UserName - encrypted and plaintext. User/pass identities are defined by configuration.
3. X509 certificates

## Crypto

OPC UA for Rust uses cryptographic algorithms for signing, verifying, encrypting and decrypting data. In addition it creates, loads and saves certificates and keys.

OpenSSL is used for encryption although it would be nice to go to a pure Rust implementation assuming a crate delivers everything required. The crypto+OpenSSL code is isolated in an `async-opcua-crypto` crate.

You must read the [setup](./setup.md) to configure OpenSSL for your environment.

### Certificate pki structure

The server / client uses the following directory structure to manage trusted/rejected certificates:

```
pki/
  own/
    cert.der - your server/client's public certificate
  private/
    key.pem  - your server/client's private key
  trusted/
    ...      - contains certs from client/servers you've connected with and you trust
  rejected/
    ...      - contains certs from client/servers you've connected with and you don't trust
```

For encrypted connections the following applies:

* The server will reject the first connection from an unrecognized client. It will create a file representing the cert in its the `pki/rejected/` folder and you, the administrator must move the cert to the `trusted/` folder to permit connections from that client in future.
    * NOTE: Signed certificates are not supported at this time. Potentially a cert signed with a trusted CA could be automatically moved to the `trusted/` folder.
* Likewise, the client shall reject unrecognized servers in the same fashion, and the cert must be moved from the `rejected/` to `trusted/` folder for connection to succeed.
* Servers that register with a discovery server may find the discovery server rejects their registration attempts if the cert is unrecognized. In that case you must move your server's cert from discovery server's  `rejected` to its ``trusted` folder, wherever that may be. e.g. on Windows it is under `C:\ProgramData\OPC Foundation\UA\Discovery\pki`

There are switches in config that can be used to change the folder that certs are stored and to modify
the trust model.

### Certificate creator tool

The `tools/certificate-creator` tool will create a demo public self-signed cert and private key. 
It can be built from source, or the crate:

```bash
$ cargo install --force async-opcua-certificate-creator
```

A minimal usage might be something like this inside samples/simple-client and/or samples/simple-server:

```bash
$ async-opcua-certificate-creator --pkipath ./pki
```

A full list of arguments can be obtained by ```--help``` and you are advised to set fields such
as expiration length, description, country code etc to your requirements.
