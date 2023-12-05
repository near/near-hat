# NEAR Hat

Prequirements 
* Docker: `brew install --cask docker` OR `https://docs.docker.com/desktop/install/mac-install/` 
* Rust: `brew install rust`
* You need to be logged into Github Container Registry first: https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry
* ![image](https://github.com/near/near-hat/assets/116191277/e20331ce-670f-43c2-b4aa-b152d490e328)


Tooling suite for local development in NEAR ecosystem.

To start:
```
$ RUST_LOG=info cargo run -p near-hat-cli -- start
```

Currently supports local versions of:
* nearcore sandbox
* NEAR Lake Indexer (+LocalStack S3)
* NEAR Relayer

Potential future support:
* Local MyNearWallet
* Local nearblocks.io (is there an opensource version?)
* BOS dependency chain and near.org gateway
* FastAuth
* Web Push Notifications
* Query API
