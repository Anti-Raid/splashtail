import boto3
import ruamel.yaml
import sys

guild_id = None
name = None
output_name = None
print(sys.argv)
if len(sys.argv) not in (3,4):
    print("Usage: python download_backups.py <guild_id> <name> <output_name>")
    print("Usage (list guild): python download_backups.py <guild_id> list")
    sys.exit(1)

with open("config.yaml") as f:
    yaml = ruamel.yaml.YAML(typ='safe', pure=True)
    config = yaml.load(f)

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

guild_id = sys.argv[1]
name = sys.argv[2]

try:
    int(guild_id)
except ValueError:
    print("guild_id must be an integer")
    sys.exit(1)

if name == "list":
    for file in client.list_objects_v2(Bucket=f"antiraid.guild.{guild_id}").get("Contents", []):
        file = file["Key"]
        print(file)
else:
    output_name = sys.argv[3]

    print(f"Searching in bucket: {name}")
    response = client.get_object(Bucket=f"antiraid.guild.{guild_id}", Key=name)
    content = response["Body"].read()

    print(f"Writing to file: {name} with file size {len(content)} bytes")

    with open(output_name, "wb") as f:
        f.write(content)