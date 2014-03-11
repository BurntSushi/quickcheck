QuickCheck for Rust with shrinking.

It is not yet in a working state although a substantial amount of code has been 
written to implement shrinking on common types.


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

