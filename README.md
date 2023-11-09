# zola_chrono
Designed to set dates on zola pages. 
It set's `date` and `updated` in zola [front matter](https://www.getzola.org/documentation/content/page/#front-matter) according to the rules listed [here](https://c-git.github.io/misc/documentation-update/#rules-for-setting-date-and-updated).
Rules can also be found in the long help output of the executable `--help`.

# Install

```sh
cargo install zola_chrono
```

# Usage

After installing run the following to see the available options

```sh
zola_chrono --help
```
<!-- TODO find way to automate having the help output show up here. Needs to be automatic because doing it manually is not sustainable. -->

See [here](https://c-git.github.io/misc/documentation-update/#rules-for-setting-date-and-updated) for a summary of the rules and a link to the test cases which best document how it works. <!-- Best to link from there to be able to update it without releasing a new version -->

To see instructions on setting it up as a pre-push hook see [my notes](https://c-git.github.io/misc/documentation-update/#using-zola-chrono) for how I did it for my use case. 
Used to be a pre-commit but that was more often that I cared for.

## License

All code in this repository is dual-licensed under either:

- Apache License, Version 2.0
- MIT license

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both as noted in
this [issue](https://github.com/bevyengine/bevy/issues/2373) on [Bevy](https://bevyengine.org)'s repo.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.