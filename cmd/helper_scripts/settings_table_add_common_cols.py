#!/bin/python3

import sys
import asyncio
import asyncpg

if len(sys.argv) != 2:
    print("Usage: python3 settings_table_add_common_cols.py <table_name>")
    print("settings_table_add_common_cols adds common columns such as created_at/created_by/last_updated_at/last_updated_by to tables")
    sys.exit(1)

table = sys.argv[1]

async def main():
    conn: asyncpg.Connection = await asyncpg.connect()

    await conn.execute(f"ALTER TABLE {table} ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()")
    await conn.execute(f"ALTER TABLE {table} ADD COLUMN IF NOT EXISTS created_by TEXT NOT NULL DEFAULT '0'")
    await conn.execute(f"ALTER TABLE {table} ALTER COLUMN created_by DROP DEFAULT")
    await conn.execute(f"ALTER TABLE {table} ADD COLUMN IF NOT EXISTS last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()")
    await conn.execute(f"ALTER TABLE {table} ADD COLUMN IF NOT EXISTS last_updated_by TEXT NOT NULL DEFAULT '0'")
    await conn.execute(f"ALTER TABLE {table} ALTER COLUMN last_updated_by DROP DEFAULT")
    
    await conn.close()

asyncio.run(main())