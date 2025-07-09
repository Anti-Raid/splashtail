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
	for d in core/rust.* services/template-worker; do \
		cd $$d && cargo fmt && cd ../..; \
	done

	# For every project in services/go.*
	for d in core/go.* services/go.*; do \
		cd $$d && go fmt && cd ../..; \
	done

prepare:
	cd services/template-worker && cargo sqlx prepare
	for d in core/rust/rust.*; do \
		echo $$d && cd $$d && cargo sqlx prepare && cd $(PWD); \
	done

# Builds AntiRaid services
build:
	mkdir -p out
	make build_go
	make build_rust

build_go:
	for d in services/api services/jobserver; do \
		echo $$d && cd ${PWD}/$$d && go build -v -o ${PWD}/out && cd ${PWD}; \
	done

build_rust:
	mkdir -p ${PWD}/out
	cd services/template-worker && cargo build --release && mv ${PWD}/target/release/template-worker ${PWD}/out/template-worker && cd ${PWD}

build_rust_dbg:
	mkdir -p ${PWD}/out/debug
	cd services/template-worker && cargo build && mv ${PWD}/target/debug/template-worker ${PWD}/out/debug/template-worker && cd ${PWD}

clean: 
	rm -rf out target

deepclean:
	make clean

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
	for d in core/go/* services/api services/jobserver; do \
		echo $$d; \
		cd $$d && GOPROXY=direct go get -u ./... && cd ${PWD}; \
	done

gomodtidy:
	PWD=$(shell pwd)
	for d in core/go.* services/go.*; do \
		echo $$d; \
		cd $$d && go mod tidy && cd ${PWD}; \
	done
