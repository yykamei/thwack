# Changelog

## [v0.2.2](https://github.com/yykamei/thwack/tree/v0.2.2) (2021-07-10)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.2.1...v0.2.2)

**Fixed bugs:**

- Fix the selection disappearance at KeyCode::Down [\#118](https://github.com/yykamei/thwack/pull/118) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Bump libc from 0.2.97 to 0.2.98 [\#116](https://github.com/yykamei/thwack/pull/116) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump unicode-segmentation from 1.7.1 to 1.8.0 [\#114](https://github.com/yykamei/thwack/pull/114) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.2.1](https://github.com/yykamei/thwack/tree/v0.2.1) (2021-06-29)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.2.0...v0.2.1)

**Fixed bugs:**

- Pass absolute path to the specified command [\#110](https://github.com/yykamei/thwack/pull/110) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Run `cargo update` [\#105](https://github.com/yykamei/thwack/pull/105) ([yykamei](https://github.com/yykamei))
- Bump crossterm from 0.19.0 to 0.20.0 [\#102](https://github.com/yykamei/thwack/pull/102) ([yykamei](https://github.com/yykamei))
- Bump libc from 0.2.96 to 0.2.97 [\#98](https://github.com/yykamei/thwack/pull/98) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.95 to 0.2.96 [\#93](https://github.com/yykamei/thwack/pull/93) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.2.0](https://github.com/yykamei/thwack/tree/v0.2.0) (2021-06-09)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.1.0...v0.2.0)

**Breaking changes:**

- Make `MatchedPath` and `Finder` private [\#38](https://github.com/yykamei/thwack/issues/38)

**Implemented enhancements:**

- Emphasize the matched characters of `MatchedPath`s [\#32](https://github.com/yykamei/thwack/issues/32)
- Implement Ord for MatchedPath to make its iterator sortable [\#30](https://github.com/yykamei/thwack/issues/30)
- Replace `Command` with `libc::execvp` [\#55](https://github.com/yykamei/thwack/pull/55) ([yykamei](https://github.com/yykamei))

**Fixed bugs:**

- Handle wrapped paths when a terminal size is narrow [\#57](https://github.com/yykamei/thwack/issues/57)
- Add `--version` option [\#44](https://github.com/yykamei/thwack/issues/44)
- Let a user input uppercase characters on Search query [\#31](https://github.com/yykamei/thwack/issues/31)
- Use grapheme cluster to detect correct indices [\#66](https://github.com/yykamei/thwack/pull/66) ([yykamei](https://github.com/yykamei))
- Handle option like value as option [\#61](https://github.com/yykamei/thwack/pull/61) ([yykamei](https://github.com/yykamei))
- Fix the calculation of `positions` [\#41](https://github.com/yykamei/thwack/pull/41) ([yykamei](https://github.com/yykamei))

**Closed issues:**

- Update README.md to describe more information [\#34](https://github.com/yykamei/thwack/issues/34)

**Merged pull requests:**

- Add a new struct: StartingPoint [\#68](https://github.com/yykamei/thwack/pull/68) ([yykamei](https://github.com/yykamei))
- Revert "Change the type of `positions` and add `positions()`" [\#49](https://github.com/yykamei/thwack/pull/49) ([yykamei](https://github.com/yykamei))
- Change the type of `positions` and add `positions()` [\#47](https://github.com/yykamei/thwack/pull/47) ([yykamei](https://github.com/yykamei))

## [v0.1.0](https://github.com/yykamei/thwack/tree/v0.1.0) (2021-05-24)

[Full Changelog](https://github.com/yykamei/thwack/compare/f9958d1dd1afb82a8fe70ca24e3753bd56d43562...v0.1.0)

**Implemented enhancements:**

- Add a new option --exec \<COMMAND\> [\#14](https://github.com/yykamei/thwack/pull/14) ([yykamei](https://github.com/yykamei))
- Add Finder to traverse files [\#6](https://github.com/yykamei/thwack/pull/6) ([yykamei](https://github.com/yykamei))

**Fixed bugs:**

- Consume ReadDir iterator strictly [\#16](https://github.com/yykamei/thwack/pull/16) ([yykamei](https://github.com/yykamei))
- Stop recursion to avoid stack overflow [\#15](https://github.com/yykamei/thwack/pull/15) ([yykamei](https://github.com/yykamei))
- Keep the path characters as is; do not call to\_lowercase\(\) [\#13](https://github.com/yykamei/thwack/pull/13) ([yykamei](https://github.com/yykamei))
- Handle invalid unicode in Finder [\#8](https://github.com/yykamei/thwack/pull/8) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Run a command for the selected path [\#17](https://github.com/yykamei/thwack/pull/17) ([yykamei](https://github.com/yykamei))
- Define our own struct: Args [\#10](https://github.com/yykamei/thwack/pull/10) ([yykamei](https://github.com/yykamei))
- Add more tests for filtering [\#9](https://github.com/yykamei/thwack/pull/9) ([yykamei](https://github.com/yykamei))
- Add MatchedPath [\#7](https://github.com/yykamei/thwack/pull/7) ([yykamei](https://github.com/yykamei))
- Add list\(\) in finder [\#5](https://github.com/yykamei/thwack/pull/5) ([yykamei](https://github.com/yykamei))
- Add ci.yml [\#1](https://github.com/yykamei/thwack/pull/1) ([yykamei](https://github.com/yykamei))



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
