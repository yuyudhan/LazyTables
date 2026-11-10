#!/usr/bin/env bash
# FilePath: docker/postgres/init/00-replication.sh
#
# Prepares a streaming-replication primary. Runs only when LT_REPLICATION_PASSWORD
# is set (the pg-cluster primary in docker/compose.yml); single instances skip it.
set -euo pipefail

if [ -z "${LT_REPLICATION_PASSWORD:-}" ]; then
    exit 0
fi

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
    -v repl_password="$LT_REPLICATION_PASSWORD" <<'SQL'
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD :'repl_password';
SQL

echo "host replication replicator all scram-sha-256" >> "$PGDATA/pg_hba.conf"
