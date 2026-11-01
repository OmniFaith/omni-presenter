# Bible data

Bible data comes from [thiagobodruk/bible](https://github.com/thiagobodruk/bible) by Thiago Bodruk and contributors. It is pinned as the `assets/bible-source` Git submodule. Initialize it after cloning:

```sh
git submodule update --init --recursive
```

`build.rs` reads the submodule's `json/index.json` and embeds translation JSON files at build time; an initialized submodule is required. The application uses the pinned checkout, not a live network request at runtime. To update the data, check out the desired upstream commit in the submodule, test, and commit the new submodule pointer.

The upstream project's code and data are distributed under its [MIT license](../bible-source/LICENSE). Its README states that the Bible versions are the property of their respective owners, with all rights reserved to those owners. See the [upstream license and copyright notice](https://github.com/thiagobodruk/bible#what-about-the-license-and-the-copyrights) and version list for details.

Additional translations can also be imported in the application. `BibleTranslation::from_json` in `src/core/bible.rs` documents the accepted formats and validation. Run `cargo test` after updating the pinned revision.

Some upstream editions are incomplete or use different book and chapter structures. In particular, the upstream Brenton Septuagint contains an empty chapter, which the current importer rejects. A version appearing in the upstream index does not guarantee that it can be loaded correctly by the current importer; check the edition before relying on it for a presentation.
