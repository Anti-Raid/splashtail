.PHONY: default $(MAKECMDGOALS)

PWD := $(shell pwd)

default:
	$(error No target provided. Please see README.md for more information)
	
format:
	# For every project in core/rust.*
	for d in services/template-worker; do \
		cd $$d && cargo fmt && cd ../..; \
	done

# Generates ts bindings
ts:
	cd services/template-worker && cargo test export_bindings

# Builds AntiRaid services
build:
	mkdir -p out
	make build_go
	make build_rust

build_go:
	for d in services/api; do \
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