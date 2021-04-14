# A detailed guide on how to set everything up

## Database

1. Install `PostgreSQL`.
2. Create a postgres database, and grant yourself permissions on it.
    You'll need either database owner or superuser privileges (We install extensions).
3. Install [timescaledb](https://timescale.com)
4. You probably need to preload the library.
    * For this, go to your postgresql config (e.g. `/etc/postgres/` or `/var/lib/postgres/data/`).
    * Update the `shared_preload_libraries` line in the config, to look like this `shared_preload_libraries = 'timescaledb'`.

The enabling of the extension will happen automatically in SqlX's migration scripts.

## Project Setup

1. Install the `sqlx-cli` app with `cargo install sqlx-cli`.
    This helper is used for migration and database management.
