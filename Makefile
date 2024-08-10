ifndef CI_BUILD
include .env
endif

.PHONY: default $(MAKECMDGOALS)

TEST__USER_ID := 728871946456137770
CDN_PATH := /silverpelt/cdn/antiraid
PWD := $(shell pwd)

default:
	$(error No target provided. Please see README.md for more information)

# This target builds all of Anti-Raid's components
buildall:
	# Core infra
	cd infra/nirn-proxy && make
	cd infra/Sandwich-Daemon && make
	cd infra/animuscli && make
	cd infra/wafflepaw && make

	# Other infra
	make buildanimuscli
	make buildwafflepaw
	make buildmewldwebui
	make build

all:
	make buildall
	
format:
	# For every project in core/rust.*
	for d in core/rust.* services/rust.*; do \
		cd $$d && cargo fmt && cd ../..; \
	done

	# For every project in services/go.*
	for d in core/go.* services/go.*; do \
		cd $$d && go fmt && cd ../..; \
	done

build:
	mkdir -p out
	make build_go
	make build_rust
	make copyassets

build_go:
	for d in services/go.*; do \
		echo $$d && cd ${PWD}/$$d && go build -v -o ${PWD}/out && cd ${PWD}; \
	done

build_rust:
	for d in services/rust.*; do \
		PROJECT_NAME=$$(basename $$d) && \
		OUTPUT_FILE=$$(echo $$PROJECT_NAME | tr . _) && \
		echo $$d && cd ${PWD}/$$d && cargo build --release && \
		mv ${PWD}/target/release/$$OUTPUT_FILE ${PWD}/out/$$PROJECT_NAME && \
		go build -v -o ${PWD}/out/$$PROJECT_NAME.loader && cd ${PWD} \
		cd ${PWD}; \
	done

copyassets:
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

buildmewldwebui:
	cd core/go.std/mewld_web/ui && npm i && npm run build && cd ../../

tests:
	CGO_ENABLED=0 go test -v -coverprofile=coverage.out ./...

ts:
	rm -rvf $(CDN_PATH)/dev/bindings/splashtail
	~/go/bin/tygo generate

	# Copy over go types
	mkdir -p $(CDN_PATH)/dev/bindings/splashtail/go
	cp -rf services/go.api/types $(CDN_PATH)/dev/bindings/splashtail/go

	# Patch to change package name to 'splashtail_types'
	#sed -i 's:package types:package splashtail_types:g' $(CDN_PATH)/dev/bindings/splashtail/go/types/{*.go,*.ts}

	# Patch to change all "SelectMenu = any;" to "SelectMenu = undefined /*tygo workaround*/;" to work around tygo issue
	sed -i 's:SelectMenu = any;:SelectMenu = undefined /*tygo workaround*/;:g' $(CDN_PATH)/dev/bindings/splashtail/discordgo.ts

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
