include .env

.PHONY: default $(MAKECMDGOALS)

TEST__USER_ID := 728871946456137770
CDN_PATH := /silverpelt/cdn/antiraid

default:
	$(error No target provided. Please see README.md for more information)

# This target builds all of Anti-Raid's components
buildall:
	# Core infra
	cd infra/nirn-proxy && make
	cd infra/Sandwich-Daemon && make

	# Other infra
	make buildanimuscli
	make buildwafflepaw
	make buildbot
	make buildmewldwebui
	make buildwebserver

# Alias for buildall
all:
	make buildall

buildanimuscli:
	cd infra/animuscli && make

buildwafflepaw:
	cd infra/wafflepaw && make

buildwebserver:
	CGO_ENABLED=0 go build -v 

reloadwebserver:
	systemctl restart splashtail-staging-webserver

restartwebserver:
	make buildwebserver
	make reloadwebserver

updatebot:
	make buildbot && cp -v target/release/botv2 botv2

updatebot_dbg:
	make buildbot_dbg && cp -v target/debug/botv2 botv2
	
format_rust:
	# For every project in core/rust.*, run cargo sqlx prepare
	for d in core/rust.*; do \
		cd $$d && cargo fmt && cd ../..; \
	done

	# For every project in services/rust.*, run cargo sqlx prepare
	for d in services/rust.*; do \
		cd $$d && cargo fmt && cd ../..; \
	done


restartbot:
	make buildbot
	make restartbot_nobuild

restartbot_nobuild:
	systemctl stop splashtail-staging-bot
	sleep 3 # Give time for the webserver to stop
	cp -v target/release/bot botv2
	systemctl start splashtail-staging-bot

reloadjobserver:
	systemctl restart splashtail-staging-jobs

sqlx:
ifndef CI_BUILD
	# For every project in core/rust.*, run cargo sqlx prepare
	for d in core/rust.*; do \
		cd $$d && cargo sqlx prepare && cd ../..; \
	done

	# For every project in services/rust.*, run cargo sqlx prepare
	for d in services/rust.*; do \
		cd $$d && cargo sqlx prepare && cd ../..; \
	done
endif

buildbot: sqlx
	cd services/rust.bot && SQLX_OFFLINE=true cargo build --release

ifndef CI_BUILD
	# For every project in core/* and services/*, copy .generated/* to data/generated/{project_name} and to the website (services/website/lib/generated)
	mkdir -p data/generated/build_assets
	for d in core/* services/*; do \
		[ -d $$d/.generated ] || continue; \
		mkdir -p data/generated/build_assets/$$(basename $$d); \
		mkdir -p services/website/src/lib/generated/build_assets; \
		cp -rf $$d/.generated/* data/generated/build_assets/$$(basename $$d); \
		cp -rf $$d/.generated/* services/website/src/lib/generated/build_assets; \
	done
endif

buildbot_dbg: sqlx
	cd services/rust.bot && SQLX_OFFLINE=true cargo build --timings

buildmewldwebui:
	cd services/go.api/mewld_web/ui && npm i && npm run build && cd ../../

tests:
	CGO_ENABLED=0 go test -v -coverprofile=coverage.out ./...

ts:
	rm -rvf $(CDN_PATH)/dev/bindings/splashtail
	~/go/bin/tygo generate

	# Copy over go types
	mkdir -p $(CDN_PATH)/dev/bindings/splashtail/go
	cp -rf core/go.std/types $(CDN_PATH)/dev/bindings/splashtail/go

	# Patch to change package name to 'splashtail_types'
	#sed -i 's:package types:package splashtail_types:g' $(CDN_PATH)/dev/bindings/splashtail/go/types/{*.go,*.ts}

	cp -rf $(CDN_PATH)/dev/bindings/splashtail/* services/website/src/lib/generated
	rm -rf services/website/src/lib/generated/go	

promoteprod:
	rm -rf ../prod2
	cd .. && cp -rf staging prod2
	echo "prod" > ../prod2/config/current-env
	cd ../prod2 && make && rm -rf ../prod && mv -vf ../prod2 ../prod && systemctl restart splashtail-prod
	cd ../prod && make ts

	# Git push to "current-prod" branch
	cd ../prod && git branch current-prod && git add -v . && git commit -m "Promote staging to prod" && git push -u origin HEAD:current-prod --force
