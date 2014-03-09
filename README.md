QuickCheck for Rust with shrinking.

It is not yet in a working state although a substantial amount of code has been 
written to implement shrinking on common types.


### Laziness

A key aspect for writing good shrinkers is a good lazy abstraction. For this,
I chose iterators. My insistence on this point has greatly complicated type 
signatures for implementing the `Shrink` trait. More than that, the code to 
produce these iterators is maddeningly complex, since the state for iteration 
must be tracked explicitly. In fact, it was so complex that I gave up on 
writing a lazy iterator for shrinking vectors, which is arguable the most 
important place for laziness.

I want laziness, but if I can't find a way to reduce the complexity of writing 
shrinkers in Rust, I'm likely to trash the idea and just use vectors. Comments 
are welcome.


### Request for review

This is my first Rust project, so I've undoubtedly written unidiomatic code. In 
fact, it would be fair to say that the code in this project just happened to be 
what I could manage to get by the compiler.

So far, I think I've failed at two things that I'm not quite sure how to 
correct:

* I'm using owned pointers everywhere and have probably used the `Clone` trait 
  as a crutch. This is particularly bad in the vector and number shrinking 
  code.
* The types one is required to write down to implement a shrinker are 
  horrendous. I consider it a bug that an implementation of the `Shrink` trait 
  exposes implementation details (like the kind of iterator used).
  I'd very much like to use an `~Iterator` trait object instead, but was
  [unable to get this to 
  work](http://www.reddit.com/r/rust/comments/1zuwdj/help_designing_a_shrinker_for_quickcheck_or_how/).

I would like to avoid using macros for building abstractions. (I'm not opposed 
to using them to generating trait implementations---as is done in the standard 
library---but I haven't learned them yet.)

