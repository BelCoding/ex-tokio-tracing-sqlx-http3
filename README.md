# An example for building with the crates tokio, tracing, sqlx and reqwest for http3

## Description

A simplified user backend for email and phone numbers.

## Installation

To run the server in your local machine PostgreSQL is needed. You can follow [this guide](https://ubuntu.com/server/docs/install-and-configure-postgresql) for installing Postgres in Ubuntu.

Apart from the crates in Cargo.toml I did install the sqlx-cli:
```cargo search sqlx-cli```

Notice that for the macros ```query!(...)``` to work the db table must be present at compile time. You can create it manually or easier with the sqlx-cli, after cloning the repo the scripts are in the migrations folder already so you just need to run:
```sqlx migrate run```