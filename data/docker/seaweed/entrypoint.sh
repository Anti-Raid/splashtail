#!/bin/sh

# Copy filer.toml to /etc/seaweedfs/filer.toml if /etc/seaweedfs/filer.toml does not exist
ls /etc/seaweedfs
if [ ! -f /etc/seaweedfs/filer.toml ]; then \
    cp /filer.toml /etc/seaweedfs/filer.toml; \
    else \
    echo "/etc/seaweedfs/filer.toml already exists, not overwriting"; \
    fi

# Same with replication.toml
if [ ! -f /etc/seaweedfs/replication.toml ]; then \
    cp /replication.toml /etc/seaweedfs/replication.toml; \
    else \
    echo "/etc/seaweedfs/replication.toml already exists, not overwriting"; \
    fi

# Same with s3.json
if [ ! -f /etc/seaweedfs/s3.json ]; then \
    cp /s3.json /etc/seaweedfs/s3.json; \
    else \
    echo "/etc/seaweedfs/s3.json already exists, not overwriting"; \
    fi

# We only care about server
case "$1" in
  'server')
  	ARGS="-dir=/data -volume.max=0 -master.volumePreallocate -master.volumeSizeLimitMB=4096 -filer.encryptVolumeData -s3.config=/etc/seaweedfs/s3.json"
 	shift
  	exec /usr/bin/weed -logtostderr=true server $ARGS $@
  	;;

  'shell')
  	ARGS="-cluster=$SHELL_CLUSTER -filer=$SHELL_FILER -filerGroup=$SHELL_FILER_GROUP -master=$SHELL_MASTER -options=$SHELL_OPTIONS"
  	shift
  	exec echo "$@" | /usr/bin/weed -logtostderr=true shell $ARGS
  ;;

  *)
  	exec /usr/bin/weed $@
	;;
esac