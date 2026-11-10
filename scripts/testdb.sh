#!/usr/bin/env bash
# FilePath: scripts/testdb.sh
#
# Manage the LazyTables test databases defined in docker/compose.yml.
#
#   scripts/testdb.sh up <profile>      start a profile and wait until healthy
#   scripts/testdb.sh probe <profile>   run a SQL probe against every node of a profile
#   scripts/testdb.sh reset <profile>   down <profile> (drops its data) then up again
#   scripts/testdb.sh down [profile]    stop one profile (default: all) and remove its volumes
#   scripts/testdb.sh status            list running test-database containers
#
# Profiles: pg, pg-cluster, mysql, mysql-cluster, mariadb, mariadb-cluster, sqlite.
# The sqlite profile has no server: `up sqlite` builds target/lazytables-test.db.
# Docs and connection strings: docs/dev/test-databases.md.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/docker/compose.yml"
SQLITE_DB="$ROOT/target/lazytables-test.db"
PROFILES=(pg pg-cluster mysql mysql-cluster mariadb mariadb-cluster sqlite)
PASSWORD="${LT_DB_PASSWORD:-lazy}"

compose() {
    docker compose -f "$COMPOSE_FILE" "$@"
}

usage() {
    sed -n '4,15p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit "${1:-0}"
}

require_profile() {
    local profile="${1:-}"
    local known
    for known in "${PROFILES[@]}"; do
        if [ "$profile" = "$known" ]; then
            return 0
        fi
    done
    echo "unknown profile '${profile}'; expected one of: ${PROFILES[*]}" >&2
    exit 2
}

sqlite_build() {
    mkdir -p "$ROOT/target"
    rm -f "$SQLITE_DB"
    # keinos/sqlite3 runs as a non-root image user; root inside a rootless container maps to the host user.
    docker run --rm --user 0:0 -v "$ROOT:/w" -w /w keinos/sqlite3 \
        sqlite3 /w/target/lazytables-test.db ".read docker/sqlite/init.sql"
    echo "sqlite fixture: $SQLITE_DB"
}

up() {
    local profile="$1"
    if [ "$profile" = "sqlite" ]; then
        sqlite_build
        return
    fi
    compose --profile "$profile" up -d --wait --wait-timeout 600
    compose --profile "$profile" ps --format 'table {{.Name}}\t{{.Status}}\t{{.Ports}}'
}

down() {
    local args=()
    local profile
    if [ "$#" -gt 0 ]; then
        args=(--profile "$1")
    else
        for profile in "${PROFILES[@]}"; do
            args+=(--profile "$profile")
        done
    fi
    compose "${args[@]}" down -v
}

psql_in() {
    local container="$1" sql="$2"
    docker exec -e PGPASSWORD="$PASSWORD" "$container" \
        psql -h 127.0.0.1 -U lazy -d lazytables -v ON_ERROR_STOP=1 -tA -c "$sql"
}

mysql_in() {
    local container="$1" client="$2" sql="$3"
    docker exec -e MYSQL_PWD="$PASSWORD" "$container" "$client" -h 127.0.0.1 -uroot -N -B -e "$sql"
}

probe_pg() {
    local container="$1"
    echo "== $container"
    psql_in "$container" "SELECT 'version', current_setting('server_version')
        UNION ALL SELECT 'in_recovery', pg_is_in_recovery()::text
        UNION ALL SELECT 'schemas', string_agg(nspname, ',' ORDER BY nspname) FROM pg_namespace
            WHERE nspname NOT LIKE 'pg\\_%' AND nspname <> 'information_schema'
        UNION ALL SELECT 'tables', count(*)::text FROM pg_tables WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
        UNION ALL SELECT 'big_events', count(*)::text FROM public.big_events"
}

probe_mysql() {
    local container="$1" client="$2"
    echo "== $container"
    mysql_in "$container" "$client" "SELECT 'version', VERSION()
        UNION ALL SELECT 'read_only', @@read_only
        UNION ALL SELECT 'databases', GROUP_CONCAT(SCHEMA_NAME ORDER BY SCHEMA_NAME) FROM information_schema.SCHEMATA
            WHERE SCHEMA_NAME NOT IN ('mysql', 'information_schema', 'performance_schema', 'sys')
        UNION ALL SELECT 'tables', COUNT(*) FROM information_schema.TABLES
            WHERE TABLE_SCHEMA NOT IN ('mysql', 'information_schema', 'performance_schema', 'sys')
        UNION ALL SELECT 'big_events', COUNT(*) FROM test_db.big_events"
}

probe() {
    local profile="$1"
    case "$profile" in
        pg)
            probe_pg lt-pg16
            probe_pg lt-pg13
            ;;
        pg-cluster)
            probe_pg lt-pg-primary
            probe_pg lt-pg-replica
            echo "== replication (primary pg_stat_replication)"
            psql_in lt-pg-primary "SELECT client_addr, state, sync_state FROM pg_stat_replication"
            ;;
        mysql)
            probe_mysql lt-mysql8 mysql
            ;;
        mysql-cluster)
            probe_mysql lt-mysql-source mysql
            probe_mysql lt-mysql-replica mysql
            echo "== replication (replica status)"
            docker exec -e MYSQL_PWD="$PASSWORD" lt-mysql-replica mysql -h 127.0.0.1 -uroot -e 'SHOW REPLICA STATUS\G' |
                grep -E 'Running:|Seconds_Behind_Source|Last_(IO_|SQL_)?Error:'
            echo "== replica write attempt (must fail: read_only)"
            if mysql_in lt-mysql-replica mysql "CREATE DATABASE lt_write_probe" 2>&1; then
                echo "replica accepted a write" >&2
                return 1
            fi
            ;;
        mariadb)
            probe_mysql lt-mariadb11 mariadb
            ;;
        mariadb-cluster)
            local node
            for node in lt-galera-1 lt-galera-2 lt-galera-3; do
                probe_mysql "$node" mariadb
                mysql_in "$node" mariadb "SHOW STATUS WHERE Variable_name IN ('wsrep_cluster_size', 'wsrep_local_state_comment')"
            done
            ;;
        sqlite)
            echo "== $SQLITE_DB"
            docker run --rm --user 0:0 -v "$ROOT:/w" -w /w keinos/sqlite3 sqlite3 /w/target/lazytables-test.db \
                "SELECT 'tables', count(*) FROM sqlite_master WHERE type = 'table'; SELECT 'views', count(*) FROM sqlite_master WHERE type = 'view';"
            ;;
    esac
}

main() {
    local command="${1:-}"
    case "$command" in
        up | probe | reset)
            require_profile "${2:-}"
            case "$command" in
                up) up "$2" ;;
                probe) probe "$2" ;;
                reset)
                    down "$2"
                    up "$2"
                    ;;
            esac
            ;;
        down)
            if [ -n "${2:-}" ]; then
                require_profile "$2"
                down "$2"
            else
                down
            fi
            ;;
        status) docker ps --filter "label=com.docker.compose.project=lt-testdb" --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}' ;;
        -h | --help | help) usage 0 ;;
        *) usage 2 ;;
    esac
}

main "$@"
