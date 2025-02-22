ifndef CI_BUILD
include .env
endif

.PHONY: default $(MAKECMDGOALS)

TEST__USER_ID := 728871946456137770
CDN_PATH := /silverpelt/cdn/antiraid
PWD := $(shell pwd)

default:
	$(error No target provided. Please see README.md for more information)

infra:
	# Core infra
	cd infra/nirn-proxy && make
	cd infra/Sandwich-Daemon && make
	
format:
	# For every project in core/rust.*
	for d in core/rust.* services/rust.*; do \
		cd $$d && cargo fmt && cd ../..; \
	done

	# For every project in services/go.*
	for d in core/go.* services/go.*; do \
		cd $$d && go fmt && cd ../..; \
	done

prepare:
	cd services/bot && cargo sqlx prepare
	cd services/template-worker && cargo sqlx prepare

# Builds AntiRaid services
build:
	mkdir -p out
	make build_go
	make build_rust
	make copyassets

build_go:
	for d in services/api services/jobserver; do \
		echo $$d && cd ${PWD}/$$d && go build -v -o ${PWD}/out && cd ${PWD}; \
	done

build_rust:
	mkdir -p ${PWD}/out
	cd services/bot && cargo build --release && mv ${PWD}/target/release/bot ${PWD}/out/bot && cd ${PWD}
	cd services/template-worker && cargo build --release && mv ${PWD}/target/release/template-worker ${PWD}/out/template-worker && cd ${PWD}

build_rust_dbg:
	mkdir -p ${PWD}/out/debug
	cd services/bot && cargo build && mv ${PWD}/target/debug/bot ${PWD}/out/debug/bot && cd ${PWD}
	cd services/template-worker && cargo build && mv ${PWD}/target/debug/template-worker ${PWD}/out/debug/template-worker && cd ${PWD}

clean: 
	rm -rf out target

cleanassets:
	rm -rf services/website/src/lib/generated services/badgerfang/src/types/splashtail

deepclean:
	make clean
	make cleanassets

copyassets:
	# For every project in core/* and services/*, copy .generated/* to data/generated/{project_name} and to the website (services/website/lib/generated)
	rm -rf data/generated/build_assets
	mkdir -p data/generated/build_assets
	for d in core/* services/*; do \
		[ -d $$d/.generated ] || continue; \
		mkdir -p data/generated/build_assets/$$(basename $$d); \
		cp -rf $$d/.generated/* data/generated/build_assets/$$(basename $$d); \
	done

	rm -rf services/website/src/lib/generated/build_assets
	mkdir -p services/website/src/lib/generated/build_assets
	
	for d in core/* services/*; do \
		[ -d $$d/.generated ] || continue; \
		mkdir -p services/website/src/lib/generated/build_assets/$$(basename $$d); \
		cp -rf $$d/.generated/* services/website/src/lib/generated/build_assets/$$(basename $$d); \
	done

	# Build rust assets too
	cd data/generated/build_assets && ../../../out/bot genassets && cd ../../..
	cd services/website/src/lib/generated/build_assets && ../../../../../../out/bot genassets && cd ../../../../../..

	# Generate typings for website
	~/go/bin/tygo generate

	# Patch to change all "SelectMenu = any;" to "SelectMenu = undefined /*tygo workaround*/;" to work around tygo issue
	sed -i 's:SelectMenu = any;:SelectMenu = undefined /*tygo workaround*/;:g' services/website/src/lib/generated/discordgo.ts

	# Copy typings to badgerfang
	rm -rf services/badgerfang/src/types/splashtail
	cp -rf services/website/src/lib/generated services/badgerfang/src/types/splashtail

docs:
	python3 docs/gen_khronos_docs.py ~/khronos docs/src/dev/templating/2-plugins.md

tests:
	./out/bot test

lint_go:
	for d in core/go.* services/go.*; do \
		~/go/bin/golangci-lint run ./$$d/...; \
	done

lintfull_go:
	go work edit -json | jq -r '.Use[].DiskPath'  | xargs -I{} ~/go/bin/golangci-lint run {}/... 

update_go:
	PWD=$(shell pwd)
	for d in core/go.* services/go.*; do \
		echo $$d; \
		cd $$d && go get -u ./... && cd ${PWD}; \
	done

gomodtidy:
	PWD=$(shell pwd)
	for d in core/go.* services/go.*; do \
		echo $$d; \
		cd $$d && go mod tidy && cd ${PWD}; \
	done
