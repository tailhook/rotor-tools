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

