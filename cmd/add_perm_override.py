# Add a new perm to kv
import asyncio, asyncpg, json
import secrets

async def main():
    conn: asyncpg.Connection = await asyncpg.connect()

    guild_id = input("Enter the guild ID: ")
    if not guild_id.strip():
        print("Guild ID cannot be empty.")
        return

    member_id = input("Enter the member ID: ")
    if not member_id.strip():
        print("Role ID cannot be empty.")
        return

    perms_input = input("Enter the permissions (as a JSON string): ")
    try:
        perms = json.loads(perms_input)
    except json.JSONDecodeError:
        print("Invalid JSON format for permissions.")
        return

    if not isinstance(perms, list):
        print("Permissions must be a list of strings.")
        return

    for v in perms:
        if not isinstance(v, str):
            print(f"Invalid permission value: {v}. All permissions must be strings.")
            return

    await conn.execute(
        "INSERT INTO guild_templates_kv (guild_id, key, value, scopes, id) VALUES ($1, $2, $3, $4, $5)",
        guild_id,
        member_id,
        json.dumps(perms),
        ["builtins.guild_permissions.overrides"],
        f"{secrets.token_hex(32)}"
    )

    await conn.close()

asyncio.run(main())