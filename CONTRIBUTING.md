## Contributing

Contributions to labrinth are welcome! However, we do have a couple dependencies that you need to get up and running. 

We reccomend using [Docker](https://www.docker.com/) for setting up your dev enviroment. If you have Docker and docker-compose installed, you may run:
```sh
docker-compose up
```
which will deploy a Meilisearch container on port 7700, a PostgreSQL container on port 5432 and a pgAdmin web UI on port 8070.
When prompted to input a server password in pgAdmin, simply enter nothing into the password field.

You will have to set up the database now. To do so, install the sqlx cli:
```sh
cargo install --git https://github.com/launchbadge/sqlx sqlx-cli --no-default-features --features postgres
```
then, run the following commands to create the database and install schemas:
```sh
sqlx database create
sqlx migrate run
```
