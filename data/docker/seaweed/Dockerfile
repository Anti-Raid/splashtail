FROM chrislusf/seaweedfs:3.85_large_disk_full

# Debugging
RUN apk add --no-cache curl

# Needed for custom entrypoint
COPY ./filer.toml /filer.toml
COPY ./replication.toml /replication.toml
COPY ./entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]