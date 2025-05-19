#!/usr/bin/env bash

if[[ -z "${SKIP_DOCKER}" ]]

set -x
set -eo pipefail
if ! [ -x "$(command -v psql)" ]; then
echo >&2 "Error: psql is not installed."
exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
echo >&2 "Error: sqlx is not installed."
echo >&2 "Use:"
echo >&2 " cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
echo >&2 "to install it."
exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"

if [[ -z "${SKIP_DOCKER}" ]]
then
docker run \
-e POSTGRES_USER=${DB_USER} \
-e POSTGRES_PASSWORD=${DB_PASSWORD} \
-e POSTGRES_DB=${DB_NAME} \
-p "${DB_PORT}":5432 \
-d postgres \
postgres -N 1000
fi
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
>&2 echo "Postgres is still unavailable - sleeping"
sleep 1
done
>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

OS_TYPE="$(uname -s)"

if [[ "$OS_TYPE" == MINGW* || "$OS_TYPE" == MSYS* ]]; then
  PS_SCRIPT_PATH="$(pwd)/scripts/set_database_url.ps1"
    echo "Windows detected via uname=$OS_TYPE"
    powershell.exe -ExecutionPolicy Bypass -File "$PS_SCRIPT_PATH" "$DATABASE_URL"
else
    echo "Non-Windows OS detected ($OS_TYPE), skipping PowerShell step"
fi

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
>&2 echo "Postgres has been migrated, ready to go!"