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

# Make a new bucket to store all data
merged_bucket_name = "antiraid.guilds"
try:
    client.create_bucket(Bucket=merged_bucket_name)
except client.exceptions.BucketAlreadyOwnedByYou:
    pass

file_count = 0
bucket_paginator = client.get_paginator('list_buckets')
for bucket_page in bucket_paginator.paginate():
    for bucket in bucket_page.get("Buckets", []):
        name: str = bucket["Name"]
        if not name.startswith("antiraid.guild."):
            continue

        print(f"Searching in bucket: {name}")
        object_paginator = client.get_paginator('list_objects_v2')
        for object_page in object_paginator.paginate(Bucket=name):
            for fileobj in object_page.get("Contents", []):
                file: str = fileobj["Key"]
                assert ".." not in file, "Invalid file name"
                merged_file_name = f"{name.replace('antiraid.guild.', '')}/{file}"
                print("Copying file:", file, "to", merged_file_name)

                # Copy the file to the merged bucket
                copy_source = {'Bucket': name, 'Key': file}
                client.copy_object(CopySource=copy_source, Bucket=merged_bucket_name, Key=merged_file_name)
                file_count += 1

    print(f"Copied {file_count} files to merged bucket {merged_bucket_name}")