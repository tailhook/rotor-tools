===========
Rotor Tools
===========

:Documentation: http://tailhook.github.io/rotor-tools/

This module contains various tools that are useful for wrinting applications
using rotor_ asynchronous framework for rust.

The tools here could be the core of rotor we are trying to make rotor
itself as small as possible, because all things in the same application
(actually the same main loop) should have same version of rotor core library.
But different versions of other libraries are possible, including rotor-tools.

Tools included:

* Simplified state machines:

    * A bare timer


.. _rotor: http://github.com/tailhook/rotor

=======
License
=======

Licensed under either of

* Apache License, Version 2.0,
  (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.

------------
Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
