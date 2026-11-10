# shellcheck shell=bash
# FilePath: docker/mysql/replica/01-start-replica.sh
#
# Init script for the mysql-cluster replica (sourced by the image entrypoint).
# Points the replica at lt-mysql-source with GTID auto-positioning; everything,
# including the seed data, arrives through replication. super_read_only is persisted
# last so even root cannot write to the replica once the real server starts.

docker_process_sql <<SQL
CHANGE REPLICATION SOURCE TO
    SOURCE_HOST='lt-mysql-source',
    SOURCE_PORT=3306,
    SOURCE_USER='repl',
    SOURCE_PASSWORD='${LT_REPLICATION_PASSWORD}',
    SOURCE_AUTO_POSITION=1,
    GET_SOURCE_PUBLIC_KEY=1,
    SOURCE_CONNECT_RETRY=2;
START REPLICA;
SET PERSIST super_read_only = ON;
SQL
