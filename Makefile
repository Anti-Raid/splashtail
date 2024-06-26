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
	make buildbot && cp -v botv2/target/release/botv2 botv2

updatebot_dbg:
	make buildbot_dbg && cp -v botv2/target/debug/botv2 botv2
formatbot:
	cd botv2 && cargo fmt

restartbot: sqlx ts
	make buildbot
	make restartbot_nobuild

restartbot_nobuild:
	systemctl stop splashtail-staging-bot
	sleep 3 # Give time for the webserver to stop
	cp -v botv2/target/release/botv2 botv2
	systemctl start splashtail-staging-bot

reloadjobserver:
	systemctl restart splashtail-staging-jobs

sqlx:
ifndef CI_BUILD
	cd botv2/jobserver && cargo sqlx prepare
	cd botv2 && cargo sqlx prepare
endif

buildbot: sqlx
	cd botv2 && SQLX_OFFLINE=true cargo build --release

buildbot_dbg: sqlx
	cd botv2 && SQLX_OFFLINE=true cargo build --timings

buildmewldwebui:
	cd webserver/mewld_web/ui && npm i && npm run build && cd ../../

tests:
	CGO_ENABLED=0 go test -v -coverprofile=coverage.out ./...

ts:
	rm -rvf $(CDN_PATH)/dev/bindings/splashtail
	~/go/bin/tygo generate

	# Copy over go types
	mkdir -p $(CDN_PATH)/dev/bindings/splashtail/go
	cp -rf splashcore/types $(CDN_PATH)/dev/bindings/splashtail/go

	# Patch to change package name to 'splashtail_types'
	sed -i 's:package types:package splashtail_types:g' $(CDN_PATH)/dev/bindings/splashtail/go/types/{*.go,*.ts}
	
	cd botv2 && cargo test
	cp -rf botv2/.generated $(CDN_PATH)/dev/bindings/splashtail/rust
	cp -rf botv2/.generated/serenity_perms.json splashcore/data/serenity_perms.json

	cp -rf $(CDN_PATH)/dev/bindings/splashtail/* website/src/lib/generated
	rm -rf website/src/lib/generated/go	

promoteprod:
	rm -rf ../prod2
	cd .. && cp -rf staging prod2
	echo "prod" > ../prod2/config/current-env
	cd ../prod2 && make && rm -rf ../prod && mv -vf ../prod2 ../prod && systemctl restart splashtail-prod
	cd ../prod && make ts

	# Git push to "current-prod" branch
	cd ../prod && git branch current-prod && git add -v . && git commit -m "Promote staging to prod" && git push -u origin HEAD:current-prod --force
