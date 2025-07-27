import boto3
import ruamel.yaml

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
dts = set()
for bucket in client.list_buckets()["Buckets"]:
    name = bucket["Name"]
    if not name.startswith("antiraid.guild."):
        continue

    print(f"Searching in bucket: {name}")
    for file in client.list_objects_v2(Bucket=name).get("Contents", []):
        file = file["Key"]
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
            dts.add(datetime)

            num_encrypted += 1
        num_total += 1

print(f"Status: {num_encrypted}/{num_total}.")
print(f"Servers with encrypted files: {', '.join(sorted(servers))} [{len(servers)}].")
print(f"Backup datetimes: {'\n'.join(sorted(dts))} [{len(dts)}].")