# Add a new perm to kv
import asyncio, asyncpg, json
import secrets

async def main():
    conn: asyncpg.Connection = await asyncpg.connect()

    guild_id = input("Enter the guild ID: ")
    if not guild_id.strip():
        print("Guild ID cannot be empty.")
        return

    role_id = input("Enter the role ID: ")
    if not role_id.strip():
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

    index = input("Enter the index: ")
    if not index.strip():
        print("No index provided")
        return
    
    try:
        index = int(index)
    except ValueError:
        print("Invalid index value. It must be an integer.")
        return

    await conn.execute(
        "INSERT INTO guild_templates_kv (guild_id, key, value, scopes, id) VALUES ($1, $2, $3, $4, $5)",
        guild_id,
        role_id,
        json.dumps({"index": index, "perms": perms}),
        ["builtins.guild_permissions"],
        f"{secrets.token_hex(32)}"
    )

    await conn.close()

asyncio.run(main())