[![Travis Build Status](https://travis-ci.org/stbuehler/rust-xml-data.svg?branch=master)](https://travis-ci.org/stbuehler/rust-xml-data)
[![crates.io](https://img.shields.io/crates/v/xml-data.svg)](https://crates.io/crates/xml-data)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

This library provides a generic interface to parse XML data: a user might implement how to parse and serialize their data (possibly derived), while others will implement adaptors for generic XML parsers.

This is similar to what serde does; but serde assumes your data consists of "native data" (strings, integers, floats, ...) and nested data (lists and maps).  XML doesn't map to this very well; while there are some adaptors, they often accept lots of structually different input data: an element might be interpreted as map in serde. A subelement with text now can be interpreted as key (`<key>value</key>`), but an attribute is interpreted the same way `<... key="value">`.

This library focuses only on XML instead, and provides a more strict interface with clearly defined output.

For the following XML handling crates adaptors are included if enabled through the equally named features:
- [`quick-xml`](https://crates.io/crates/quick-xml)

If the `derive` feature is enabled the following traits can be derived:
- `Element`, `parser::Element`, `serializer::Element`)
- `Inner`, `parser::Inner`, `serializer::Inner`)

The documentation for `master` is located at [https://stbuehler.github.io/rustdocs/xml-data/xml-data/](https://stbuehler.github.io/rustdocs/xml-data/xml-data/); released versions are documented at [https://docs.rs/xml-data](https://docs.rs/xml-data).
