import boto3
import ruamel.yaml

with open("config.yaml") as f:
    yaml = ruamel.yaml.YAML(typ='safe', pure=True)
    config = yaml.load(f)

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

file_count = 0

object_paginator = client.get_paginator('list_objects_v2')
for object_page in object_paginator.paginate(Bucket="antiraid.guilds"):
    for fileobj in object_page.get("Contents", []):
        file: str = fileobj["Key"]
        print("file: ", file)
        file_count += 1
print(f"Total files in single bucket: {file_count}")