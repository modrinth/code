## Contributing

Contributions to labrinth are welcome! However, we do have a couple dependencies that you need to get up and running. 

We reccomend using [Docker](https://www.docker.com/) for setting up your dev enviroment. If you have Docker and docker-compose installed, you may run:
```sh
docker-compose up
```
which will deploy a Meilisearch container on port 7700, a MongoDB instance on port 27017 and a MongoDB web UI on port 8081

Alternatively, follow theese steps:
1. Install and run a [MeiliSearch](https://docs.meilisearch.com/guides/introduction/quick_start_guide.html) instance
2. Install [A local MongoDB server](https://www.mongodb.com/try/download/community)
3. Run `mongod --dbpath path/to/db`
4. Everything should be setup and you should be ready to contribute.