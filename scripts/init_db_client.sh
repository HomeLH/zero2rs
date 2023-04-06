DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
HOST="127.0.0.1"

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done
>&2 echo "Postgres is up and running on port ${DB_PORT}!"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${HOST}:${DB_PORT}/${DB_NAME}
echo ${DATABASE_URL}
sqlx database create
sqlx migrate run

