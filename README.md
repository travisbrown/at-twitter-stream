## Twitter stream user info extractor

[![Build status](https://img.shields.io/github/workflow/status/travisbrown/at-twitter-stream/ci.svg)](https://github.com/travisbrown/at-twitter-stream/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/at-twitter-stream/main.svg)](https://codecov.io/github/travisbrown/at-twitter-stream)

This project lets you parse JSON data from the Twitter API or other sources to extract some basic user information,
and then to query that data more or less efficiently, even if you've extracted information about a lot of users.

My local instance has processed a small fraction of the Archive Team Twitter Stream data sets this week, and
currently contains 115 million historical screen names for 106 million Twitter accounts (many of which have been deactivated or suspended).

### Why would you do this?

I have a lot of one-off Rust projects like this, and I decided to publish this one to show how Rust can
be used for little data exploration tasks that might not traditionally be considered its domain.

I'm not a Rust expert and the code is probably mostly uninteresting and not a demonstration of best practices,
but that's kind of the point: you can just hack together something fun pretty quickly and the language doesn't
really get in the way.

As for the task, there's just a lot of bizarre history in archived Twitter metadata, and it can be fun to explore.
Like Rudy Giuliani briefly changing his Twitter handle to `@xxxxxxx37583982` in November 2016, for example:

```bash
$ target/release/ts-db query-screen-name RudyGiuliani
770781940341288960
$ target/release/ts-db query-user-id 770781940341288960
RudyGiuliani
xxxxxxx37583982
```

Why did this ghoul-ass motherfucker do this? I have no idea.

It can also actually be useful. Consider for example de-anonymizations like
[this one](https://exposedeznat.noblogs.org/mormonger-exposed-as-cole-noorda-of-salt-lake-city-utah-deznat-mormon/),
where "Mormonger", a popular homophobic and transphobic figure in the far-right Deseret nationalist movement,
was revealed to be a man named Cole Noorda from Salt Lake City, Utah. The published de-anonymization involves a lot of
careful research and presentation of evidence, but Mormonger's screen name history gives away the punch line:

```bash
$ target/release/ts-db query-screen-name Mormonger
1408886100
$ target/release/ts-db query-user-id 1408886100
Mormonger
colenoorda
```

He seems to have changed the account's screen name from `@Mormonger` to `@colenoorda` very briefly in June 2016,
but the Internet Archive remembers.

The other reason I did this is just that I wanted to play around with [this Rust library](https://github.com/rust-rocksdb/rust-rocksdb)
for working with [RocksDB](http://rocksdb.org), and this data seemed like a good excuse.

## License

This project is licensed under the Mozilla Public License, version 2.0. See the LICENSE file for details.
