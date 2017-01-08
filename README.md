# wdsapi
Rust API for the IBM Watson Discovery Service
[![Build Status](https://travis-ci.org/bruceadams/wdsapi.svg?branch=master)](https://travis-ci.org/bruceadams/wdsapi)

This project is platform for learning for me.

I hope to get it published into crates.io, once I figure out how to do that.
Similarly, I hope to get the API documentation properly published.

This project uses the serde's code generation features,
which currently requires the _nightly_ toolchain.

I have chosen to be strict in what I expect the Watson Discovery Service
(WDS) to return. This is likely to break things as WDS makes changes and
additions in the future.
