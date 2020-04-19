# headless-haven-helper

This is an UNOFFICIAL headless version of [Gloomhaven helper][helper], implemented in Rust.

It currently supports listening to state updates from a 8.3.x server and printing them on stdout.

<hr />

## Purpose

The purpose of this project is to enable a headless server of the Gloomhaven helper to run in e.g. a container.
Since the protocol also seems to be esoteric (no pun intended), this project could provide another API in addition to
the current binary-focused one.


[helper]: http://esotericsoftware.com/gloomhaven-helper