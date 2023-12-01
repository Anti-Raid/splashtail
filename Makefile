TEST__USER_ID := 728871946456137770
CDN_PATH := /failuremgmt/cdn/antiraid


stcore:
	CGO_ENABLED=0 go build -v 
	systemctl reload splashtail-staging
all:
	make buildbot && make stcore 
buildbot:
	cd bot && npm i && npm run build
tests:
	CGO_ENABLED=0 go test -v -coverprofile=coverage.out ./...
ts:
	rm -rvf $(CDN_PATH)/dev/bindings/splashtail
	~/go/bin/tygo generate

	# Copy over go types
	mkdir $(CDN_PATH)/dev/bindings/splashtail/go
	cp -rf types $(CDN_PATH)/dev/bindings/splashtail/go

	# Patch to change package name to 'splashtail_types'
	sed -i 's:package types:package splashtail_types:g' $(CDN_PATH)/dev/bindings/splashtail/go/types/*

	# Add enums
	STAGING_API=true ibl genenums

promoteprod:
	rm -rf ../prod2
	cd .. && cp -rf staging prod2
	echo "prod" > ../prod2/config/current-env
	cd ../prod2 && make && rm -rf ../prod && mv -vf ../prod2 ../prod && systemctl restart splashtail-prod
	cd ../prod && make ts

	# Git push to "current-prod" branch
	cd ../prod && git branch current-prod && git add -v . && git commit -m "Promote staging to prod" && git push -u origin HEAD:current-prod --force
