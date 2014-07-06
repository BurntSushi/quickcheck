RUST_CFG ?=
RUST_PATH ?= ./build/
RUSTC ?= rustc

LIB = build/.timestamp_quickcheck
MACRO_LIB = build/.timestamp_quickcheck_macro

DIR=src
MACRO_DIR=quickcheck_macros/src

compile: $(LIB) $(MACRO_LIB)

$(LIB): $(DIR)/lib.rs $(DIR)/arbitrary.rs
	@mkdir -p ./build
	$(RUSTC) -O $(DIR)/lib.rs --out-dir build
	@touch $(LIB)

$(MACRO_LIB): $(MACRO_DIR)/lib.rs
	@mkdir -p ./build
	$(RUSTC) -O $(MACRO_DIR)/lib.rs --out-dir build
	@touch $(MACRO_LIB)

ctags:
	ctags --recurse --options=ctags.rust --languages=Rust

docs:
	rm -rf doc
	rustdoc $(DIR)/lib.rs
	# WTF is rustdoc doing?
	chmod 755 doc
	in-dir doc fix-perms
	rscp ./doc/* gopher:~/www/burntsushi.net/rustdoc/

test: build/test
	RUST_TEST_TASKS=1 RUST_LOG=quickcheck=4 ./build/test

build/test: build $(DIR)/lib.rs $(DIR)/arbitrary.rs $(MACRO_DIR)/lib.rs $(LIB) $(MACRO_LIB)
	$(RUSTC) -L $(RUST_PATH) --test $(RUST_CFG) $(DIR)/lib.rs -o build/test

test-examples:
	(cd ./examples && ./test)

test-clean:
	rm -rf ./quickcheck-test

clean: test-clean
	rm -f ./build/* $(LIB)

push:
	git push origin master
	git push github master
