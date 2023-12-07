# NEARHat

Tooling suite for local development in NEAR ecosystem. It allows you to run local development of dApps and create
automated end-to-end tests from smart contracts to indexers. 

Currently supports local versions of:
* nearcore sandbox
* NEAR Lake Indexer (+LocalStack S3)
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
* ![image](https://github.com/near/near-hat/assets/116191277/e20331ce-670f-43c2-b4aa-b152d490e328)

## Starting and stopping local environment
```
$ ./start.sh
```

