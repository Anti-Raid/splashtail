#!/bin/python3

import sys
import asyncio
import asyncpg

async def main():
    conn: asyncpg.Connection = await asyncpg.connect()

    jobs = await conn.fetch("SELECT id, owner FROM jobs")

    for job in jobs:
        if job["owner"].startswith("u/"):
            raise ValueError(f"Job {job['id']} has an owner that starts with 'u/'")

        guild_id = job["owner"].split("/")[1]

        await conn.execute("UPDATE jobs SET owner = $1 WHERE id = $2", guild_id, job["id"])

    # Rename owner to guild id and add foreign key constraint
    await conn.execute("ALTER TABLE jobs RENAME COLUMN owner TO guild_id")
    await conn.execute("ALTER TABLE jobs ADD CONSTRAINT fk_guild_id FOREIGN KEY (guild_id) REFERENCES guilds(id) ON UPDATE CASCADE ON DELETE CASCADE")

    await conn.close()

asyncio.run(main())