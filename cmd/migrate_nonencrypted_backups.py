import boto3
import ruamel.yaml
import subprocess
import asyncio, asyncpg, json, secrets

with open("config.yaml") as f:
    yaml = ruamel.yaml.YAML(typ='safe', pure=True)
    config = yaml.load(f)

# iblfile constants
AUTO_ENCRYPTED_FILE_MAGIC = b"iblaef";
AUTO_ENCRYPTED_FILE_CHECKSUM_SIZE = 32
AUTO_ENCRYPTED_FILE_ID_SIZE = 16

TAG = "aes256"

endpoint = config["object_storage"]["endpoint"]
access_key = config["object_storage"]["access_key"]
secret_key = config["object_storage"]["secret_key"]
secure = config["object_storage"]["secure"]
client = boto3.client(
    "s3",
    endpoint_url=f"http://{endpoint}" if not secure else f"https://{endpoint}",
    aws_access_key_id=access_key,
    aws_secret_access_key=secret_key,
)

num_encrypted = 0
num_total = 0
servers = set()

non_encrypted = set()

for bucket in client.list_buckets()["Buckets"]:
    name: str = bucket["Name"]
    if not name.startswith("antiraid.guild."):
        continue

    print(f"Searching in bucket: {name}")
    for fileobj in client.list_objects_v2(Bucket=name).get("Contents", []):
        file: str = fileobj["Key"]
        print(file)

        # Download the first 100 bytes of the file
        response = client.get_object(Bucket=name, Key=file, Range="bytes=0-100")
        content = response["Body"].read()
        
        magic = content[0:len(AUTO_ENCRYPTED_FILE_MAGIC)]
        checksum = content[len(AUTO_ENCRYPTED_FILE_MAGIC):len(AUTO_ENCRYPTED_FILE_MAGIC) + AUTO_ENCRYPTED_FILE_CHECKSUM_SIZE]
        encryptor = content[len(AUTO_ENCRYPTED_FILE_MAGIC) + AUTO_ENCRYPTED_FILE_CHECKSUM_SIZE:len(AUTO_ENCRYPTED_FILE_MAGIC) + AUTO_ENCRYPTED_FILE_CHECKSUM_SIZE + AUTO_ENCRYPTED_FILE_ID_SIZE]
        print(encryptor)
        if encryptor == b"aes256$$$$$$$$$$":
            print(f"Found encrypted file: {file}")
            servers.add(name.split(".")[2])

            # Get the time the backup was created
            datetime = file.replace("antiraid-backup-", "").replace(".iblfile", "")

            num_encrypted += 1
        else:
            non_encrypted.add((name, file))
            print(f"Found non-encrypted file: {file}")
        num_total += 1

print(f"Status: {num_encrypted}/{num_total}.")
print(f"Servers with encrypted files: {', '.join(sorted(servers))} [{len(servers)}].")

async def migrate_all():
    conn: asyncpg.Connection = await asyncpg.connect()

    for non_encrypted_file in non_encrypted:
        if not non_encrypted_file[1].endswith(".iblfile"):
            print(f"Skipping non-backup file: bucket={non_encrypted_file[0]} file={non_encrypted_file[1]}")
            continue

        # Calculate output directory
        #print(f"Non-encrypted file: bucket={non_encrypted_file[0]} file={non_encrypted_file[1]}")
        guild_id = non_encrypted_file[0].split(".")[2]
        backupid = non_encrypted_file[1].replace("antiraid-backup-", "").replace(".iblfile", "").split("/")[1]
        dt = non_encrypted_file[1].replace("antiraid-backup-", "").replace(".iblfile", "").split("/")[2]
        #print(f"Backup ID: {backupid}, datetime: {dt}")
        output_path = f"antiraidbackup-{dt}.arb2"

        code = subprocess.run(
            ["legacybackupconverter", "s3", non_encrypted_file[0], non_encrypted_file[1], output_path],
            env={
                "LBC_S3_ACCESS_KEY": access_key,
                "LBC_S3_SECRET_KEY": secret_key,
                "LBC_S3_ENDPOINT": endpoint,
                "LBC_S3_SSL": "true" if secure else "false",
            }
        )

        if code.returncode != 0:
            raise RuntimeError(f"Failed to convert {non_encrypted_file[1]} (exit code {code.returncode})")

        print(f"Converted {non_encrypted_file[1]} to {output_path}")

        # Save metadata for the bot
        await conn.execute("DELETE FROM guild_templates_kv WHERE guild_id = $1 AND key = $2", guild_id, f"migrated.{backupid}")
        await conn.execute(
            "INSERT INTO guild_templates_kv (guild_id, key, value, scopes, id) VALUES ($1, $2, $3, $4, $5)",
            guild_id,
            f"migrated.{backupid}",
            json.dumps({"filename": output_path, "created_by": "0", "num_updates": 0}),
            ["builtins.backups.metadata"],
            f"{secrets.token_hex(32)}"
        )


asyncio.run(migrate_all())