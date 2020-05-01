.PHONY: demo clean

demo: dist/demo demo/models | .oasis
	@oasis chain >/dev/null 2>&1 &
	@yarn -s start
	@pkill oasis

dist/demo:
	oasis build

demo/models:
	$(MAKE) -C demo

.oasis:
	@mkdir -p $@

clean:
	$(MAKE) -C demo clean
	rm -rf dist node_modules target
