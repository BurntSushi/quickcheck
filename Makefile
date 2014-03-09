install:
	cargo-lite install

docs:
	rm -rf doc
	rustdoc src/lib.rs

test: quickcheck-test
	./quickcheck-test ; rm -f quickcheck-test

quickcheck-test:
	rustc --test src/lib.rs -o quickcheck-test

push:
	git push origin master
	git push github master

