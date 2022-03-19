# QSV Sniffer

> NOTE: This fork of [csv-sniffer](https://github.com/jblondin/csv-sniffer) addresses an [uninitialized memory security advisory](https://github.com/jblondin/csv-sniffer/issues/1),
updates dependencies, and modernizes the code base, eliminating some deprecated features since the crate was last updated.
This is being published on crates.io, so that qsv can be published as well. Once this [pull request](https://github.com/jblondin/csv-sniffer/pull/2) upstream is merged,
this fork will be removed from crates.io.

This `qsv-sniffer` crate provides methods to infer CSV file details (delimiter choice, quote
character, number of fields, field data types, etc.).
