QuickCheck for Rust with shrinking.

This is pretty close to a working state. Most (or all) of the plumbing has been
done. All that's really left is to actually run the tests given a property.
(And of course, craft a public API.)

The "plumbing" includes `Arbitrary`, `Shrink` and `Testable` traits, among a
few others.


### Documentation

Documentation is a work in progress:
[http://burntsushi.net/rustdoc/quickcheck/](http://burntsushi.net/rustdoc/quickcheck/).


### Laziness

A key aspect for writing good shrinkers is a good lazy abstraction. For this,
I chose iterators. My insistence on this point has resulted in the use of an
existential type, which I think I'm willing to live with.

Note though that the shrinkers for lists and integers are not lazy. Their
algorithms are more complex, so it will take a bit more work to get them to
use iterators like the rest of the shrinking strategies.


### Request for review

This is my first Rust project, so I've undoubtedly written unidiomatic code. In 
fact, it would be fair to say that the code in this project just happened to be 
what I could manage to get by the compiler.

I think my primary concern is whether I'm using the region types correctly.

Also, I would like to avoid using macros for building abstractions. (I'm not 
opposed to using them to generating trait implementations---as is done in the 
standard library---but I haven't learned them yet.)

