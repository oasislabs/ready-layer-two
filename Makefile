.PHONY: demo demo_deps clean

demo: demo_deps dist/demo | .oasis
	@pkill oasis || true
	@oasis chain >/dev/null 2>&1 &
	@yarn -s start
	@pkill oasis

dist/demo:
	oasis build

demo_deps:
	@$(MAKE) -s -C demo

clean:
	$(MAKE) -C demo clean
	rm -rf dist node_modules target
