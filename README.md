# NEARHat

NEARHat is a NEAR Protocol local development environment.
It allows you to run local development of dApps and create
automated end-to-end tests from smart contracts to indexers.

![NEARHat-Logo](https://github.com/near/near-hat/assets/116191277/68326fa2-f9d9-45b4-a332-078b4733d376)

Built by the Pagoda Engineers for the NEAR Ecosystem as part of the December 2023 hackathon.

Currently supports local versions of:
* nearcore sandbox
* NEAR Lake Indexer (+ LocalStack NEAR Lake S3 bucket)
* NEAR Relayer
* Local NEAR Explorer
* Query API

Potential future support:
* Local MyNearWallet
* BOS dependency chain and near.org gateway
* FastAuth


## One line installation:
```
$ ./install.sh
```
This will install dependencies via Homebrew and setup local `.nearhat` domain.

You need to be logged into Github Container Registry (until all docker containers are published to DockerHub): https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry

![image](https://github.com/near/near-hat/assets/116191277/e20331ce-670f-43c2-b4aa-b152d490e328)

## Starting and stopping local environment
```
$ ./start.sh
```

## Forking mainnet smart contracts
NEARHat allows to fork mainnet contracts and refer to them through `http://rpc.nearhat`.
To fork the USDC contract (with account id `17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2`) start NEARHat with the following command:
```bash
RUST_BACKTRACE=1 RUST_LOG=info cargo run -p near-hat-cli -- start --contracts-to-spoon 17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2
```
