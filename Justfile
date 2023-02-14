init-db:
    #!/usr/bin/env bash
    set -o allexport; source .env || echo "no .env file to read"; set +o allexport
    sqlx database create
    sqlx migrate run
