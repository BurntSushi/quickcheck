RUST_CFG=

compile:
	rustc -O ./src/lib.rs

install:
	cargo-lite install

ctags:
	ctags --recurse --options=ctags.rust --languages=Rust

docs:
	rm -rf doc
	rustdoc src/lib.rs
	# WTF is rustdoc doing?
	chmod 755 doc
	in-dir doc fix-perms
	rscp ./doc/* gopher:~/www/burntsushi.net/rustdoc/

test: quickcheck-test
	RUST_TEST_TASKS=1 RUST_LOG=quickcheck=4 ./quickcheck-test

quickcheck-test: src/lib.rs src/arbitrary.rs
	rustc --test $(RUST_CFG) src/lib.rs -o quickcheck-test

test-examples:
	(cd ./examples && ./test)

test-clean:
	rm -rf ./quickcheck-test

clean: test-clean
	rm -f *.rlib

push:
	git push origin master
	git push github master

