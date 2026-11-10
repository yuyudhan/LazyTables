# shellcheck shell=bash
# FilePath: docker/mariadb/init/00-replication.sh
# (identical copy in docker/mysql/init; keep both in sync)
#
# Sourced (not executed) by the image entrypoint, so docker_process_sql is available.
# Creates the replication user on a replication source. Runs only when
# LT_REPLICATION_PASSWORD is set (the *-cluster sources in docker/compose.yml).
# Re-issuing CREATE DATABASE for MYSQL_DATABASE writes it to the binlog so replicas
# that replay from the start get the database before the seed scripts use it.

if [ -n "${LT_REPLICATION_PASSWORD:-}" ]; then
    docker_process_sql <<SQL
CREATE DATABASE IF NOT EXISTS \`${MYSQL_DATABASE:-${MARIADB_DATABASE}}\`;
CREATE USER IF NOT EXISTS 'repl'@'%' IDENTIFIED BY '${LT_REPLICATION_PASSWORD}';
GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';
SQL
fi
