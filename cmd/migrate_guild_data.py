# Migrate guild_roles to kv
import asyncio, asyncpg, json
import secrets

async def main():
    conn: asyncpg.Connection = await asyncpg.connect()

    data = await conn.fetch(f"SELECT guild_id, role_id, perms, index, created_at, last_updated_at, created_by, last_updated_by FROM guild_roles")
    
    for entry in data:
        print(f"Inserting role {entry['role_id']} for guild {entry['guild_id']} with perms {entry['perms']} at index {entry['index']}")
        await conn.execute(
            "INSERT INTO guild_templates_kv (guild_id, key, value, scopes, created_at, last_updated_at, id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            entry["guild_id"],
            entry["role_id"],
            json.dumps({"index": entry["index"], "perms": entry["perms"]}),
            ["builtins.guild_permissions"],
            entry["created_at"],
            entry["last_updated_at"],
            f"automigrated.{entry['created_by']}.{entry['last_updated_by']}/{secrets.token_hex(24)}"
        )

    print(data)

    await conn.close()

asyncio.run(main())