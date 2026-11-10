# Test databases

`docker/compose.yml` defines seeded test databases for every engine LazyTables supports,
as single instances and as clusters. `scripts/testdb.sh` (or the `make db-*` targets)
drives it. Requires Docker with the compose plugin; no local database clients are needed.

```sh
scripts/testdb.sh up pg          # start a profile, wait until every node is healthy
scripts/testdb.sh probe pg       # SQL probe: version, recovery/read-only state, schemas, table count
scripts/testdb.sh reset pg       # drop the profile's data and start it again from the seeds
scripts/testdb.sh down pg        # stop one profile and drop its data
scripts/testdb.sh down           # stop every profile
scripts/testdb.sh status         # running test containers

make db-up PROFILE=pg-cluster    # same commands via make: db-up, db-probe, db-reset, db-down, db-status
```

Start one profile at a time when memory is tight (measured: each Galera node uses about
115-140 MB after seeding). Ports bind to `127.0.0.1` only. Data is not
kept: `down` removes the volumes, so the next `up` re-runs the seed scripts.
Credentials are throwaway test values; `LT_DB_PASSWORD` and `LT_REPL_PASSWORD`
override `lazy` and `replpass`.

## Profiles and connection strings

| Profile | Container | Role | Connection string |
| --- | --- | --- | --- |
| `pg` | lt-pg16 | Postgres 16 | `postgresql://lazy:lazy@127.0.0.1:15432/lazytables` |
| `pg` | lt-pg13 | Postgres 13 | `postgresql://lazy:lazy@127.0.0.1:15433/lazytables` |
| `pg-cluster` | lt-pg-primary | Postgres 16 primary | `postgresql://lazy:lazy@127.0.0.1:15442/lazytables` |
| `pg-cluster` | lt-pg-replica | streaming hot standby (read-only) | `postgresql://lazy:lazy@127.0.0.1:15443/lazytables` |
| `mysql` | lt-mysql8 | MySQL 8.4 | `mysql://root:lazy@127.0.0.1:13306/test_db` |
| `mysql-cluster` | lt-mysql-source | MySQL 8.4 GTID source | `mysql://root:lazy@127.0.0.1:13316/test_db` |
| `mysql-cluster` | lt-mysql-replica | GTID replica, `super_read_only` | `mysql://root:lazy@127.0.0.1:13317/test_db` |
| `mariadb` | lt-mariadb11 | MariaDB 11.4 | `mysql://root:lazy@127.0.0.1:13307/test_db` |
| `mariadb-cluster` | lt-galera-1..3 | 3-node Galera, every node writable | `mysql://root:lazy@127.0.0.1:13321/test_db` (13322, 13323) |
| `sqlite` | none | file built from `docker/sqlite/init.sql` | `target/lazytables-test.db` |

Other logins on every server:

- Postgres `lt_reader` / `lt_reader`: `SELECT` on all schemas except `restricted`
  (no `USAGE`), e.g. `postgresql://lt_reader:lt_reader@127.0.0.1:15432/lazytables`.
- MySQL/MariaDB `lazytables` / `lazy`: full rights on `test_db` only.
- MySQL/MariaDB `lt_reader` / `lt_reader`: `SELECT` on `test_db`, `lt_sales`, `lt_inventory`,
  `lt_empty`; no access to `lt_restricted`.

In LazyTables: pane 1, `a`, type a name, Tab twice to the connection string field, paste
the string, Tab, `s` to save, Enter on the connection to connect.

## Fixtures

Seed scripts run in file order from `docker/<engine>/init` (replicas receive the data
through replication; Galera nodes 2 and 3 through SST).

Postgres (`lazytables` database):

- Schemas `public`, `analytics`, `admin` (01-03 seeds) plus `sales`, `inventory`,
  `"CamelCase Schema"` (needs identifier quoting) and `restricted` (04).
- Same table name in three schemas: `public.products`, `sales.products`, `inventory.products`;
  `orders` in `public` and `sales`.
- Foreign keys across schemas (`sales.products -> public.products`,
  `inventory.products -> sales.products`, `sales.orders -> public.users`,
  `"CamelCase Schema"."OrderLines" -> sales.orders`).
- View `sales.v_open_orders`, materialized view `inventory.mv_stock_by_warehouse`,
  enums `sales.order_status` and `public.mood`, `TEXT[]`/`VARCHAR[]` arrays, `json` and `jsonb`.
- `public.all_types`: one column per common type (numeric, money, bytea, time zones,
  interval, uuid, inet/cidr/macaddr, point, tsvector, xml, bit/varbit, ranges, enum) with a
  typical row, an edge-value row (unicode, 64 KB text, NaN/Infinity, empty arrays) and an
  all-NULL row.
- `public.big_events`: 100,000 rows for paging, with NULLs and some 5,000-character
  unicode notes.

MySQL and MariaDB (identical `04-feature-fixtures.sql` in both init directories):

- Databases `test_db` (01-03 seeds), `lt_sales`, `lt_inventory`, `lt_empty`, `lt_restricted`.
- `products` in `test_db`, `lt_sales` and `lt_inventory` with cross-database foreign keys;
  view `lt_sales.v_active_products`.
- `test_db.all_types`: every integer width, unsigned, decimal, float/double, bit, bool,
  char/varchar/text widths, binary/varbinary/blob, date/time/datetime(6)/timestamp/year,
  enum, set, json; typical, edge-value and all-NULL rows.
- `test_db.big_events`: 100,000 rows for paging, with NULLs and long unicode notes.

## Cluster notes

- `pg-cluster`: the replica is created with `pg_basebackup -R` from the primary
  (role `replicator`, created by `docker/postgres/init/00-replication.sh` only when
  `LT_REPLICATION_PASSWORD` is set) and runs as a hot standby; writes fail with
  `cannot execute ... in a read-only transaction`.
- `mysql-cluster`: GTID auto-position replication. The replica's only init script,
  `docker/mysql/replica/01-start-replica.sh`, points it at the source and persists
  `super_read_only`, so even root cannot write to it.
- `mariadb-cluster`: Galera 4 with `mariabackup` SST, config in
  `docker/mariadb/galera/galera.cnf`. Node 1 always bootstraps (`--wsrep-new-cluster`),
  which is safe because `down` drops the data; nodes 2 and 3 join in order.
