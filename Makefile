RUST_CFG ?=
RUST_PATH ?= ./build/
RUSTC ?= rustc
LIB = build/.timestamp_quickcheck

compile: $(LIB)

$(LIB): src/lib.rs src/arbitrary.rs src/macro.rs
	@mkdir -p ./build
	$(RUSTC) -O ./src/lib.rs --out-dir build
	$(RUSTC) -O ./src/macro.rs --out-dir build
	@touch build/.timestamp_quickcheck

ctags:
	ctags --recurse --options=ctags.rust --languages=Rust

docs:
	rm -rf doc
	rustdoc src/lib.rs
	# WTF is rustdoc doing?
	chmod 755 doc
	in-dir doc fix-perms
	rscp ./doc/* gopher:~/www/burntsushi.net/rustdoc/

test: build/test
	RUST_TEST_TASKS=1 RUST_LOG=quickcheck=4 ./build/test

build/test: src/lib.rs src/arbitrary.rs $(LIB)
	$(RUSTC) -L $(RUST_PATH) --test $(RUST_CFG) src/lib.rs -o build/test

test-examples:
	(cd ./examples && ./test)

test-clean:
	rm -rf ./quickcheck-test

clean: test-clean
	rm -f ./build/* $(LIB)

push:
	git push origin master
	git push github master
