# MongoDB
[MongoDB](https://www.mongodb.com/) is a felxible document NoSQL database.


## Supported versions
| Agent Version | MongoDB Version | MongoDB Mode            |
| ------------- | --------------- | ----------------------- |
| 0.2.0+        | 3.0+ / 3.2+     | Replica Set / Clustered |
| 0.1.0+        | 3.2+            | Replica Set             |


## Installation from code
The following instructions where executed on a clean Fedora 28 install
but should work for any Linux system:
```bash
# Install needed tools and dependencies.
dnf install cmake gcc git make openssl-devel

# Install rust and cargo with rustup.
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Get the code and compile replicante.
git clone --recursive https://github.com/replicante-io/agents.git
cd agents
cargo build --release

# Ensure the built binaries work.
target/release/replicante-agent-mongodb
```


## Configuration
[import, lang:"yaml"](../../agent-mongodb.example.yaml)


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/mongodb/CHANGELOG.md)
for all details.

### Upgrading to 0.2.0
- The configuration format was changed and existing files may not work.
