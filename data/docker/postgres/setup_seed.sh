#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	--CREATE ROLE antiraid WITH SUPERUSER LOGIN PASSWORD 'antiraid';
    CREATE ROLE root WITH SUPERUSER LOGIN PASSWORD 'root';
    CREATE ROLE frostpaw WITH SUPERUSER LOGIN PASSWORD 'frostpaw';
    --CREATE ROLE antiraid WITH SUPERUSER LOGIN PASSWORD 'ibl';
    CREATE DATABASE frostpaw WITH OWNER frostpaw;
	GRANT ALL PRIVILEGES ON DATABASE antiraid TO antiraid;
    GRANT ALL PRIVILEGES ON DATABASE frostpaw TO frostpaw;
EOSQL

# Download latest iblcli
cd ~/
echo "Downloading latest iblcli..."
cd /ibl

# Keep rerunning `ibl db load /seed.iblcli-seed` until exit code 0
while true; do
    echo "Running ibl db load data/seed.iblcli-seed..."
    PGUSER=antiraid ./ibl db load /seed.iblcli-seed
    if [ $? -eq 0 ]; then
        echo "Seed loaded successfully."
        break
    else
        echo "Seed loading failed, retrying..."
        sleep 1
    fi
done