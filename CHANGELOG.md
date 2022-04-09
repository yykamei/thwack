# Changelog

## [Unreleased](https://github.com/yykamei/thwack/tree/HEAD)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.7.0...HEAD)

**Merged pull requests:**

- Remove resolved TODO comment [\#256](https://github.com/yykamei/thwack/pull/256) ([yykamei](https://github.com/yykamei))
- Bump libc from 0.2.121 to 0.2.122 [\#254](https://github.com/yykamei/thwack/pull/254) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump crossterm from 0.23.1 to 0.23.2 [\#252](https://github.com/yykamei/thwack/pull/252) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.7.0](https://github.com/yykamei/thwack/tree/v0.7.0) (2022-04-01)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.6.0...v0.7.0)

**Breaking changes:**

- Drop the support of aarch64-unknown-linux-musl and arm-unknown-linux-musleabihf [\#247](https://github.com/yykamei/thwack/pull/247) ([yykamei](https://github.com/yykamei))
- Change Ord for MatchedPath [\#243](https://github.com/yykamei/thwack/pull/243) ([yykamei](https://github.com/yykamei))
- Make command-line arguments take precedence over envs [\#227](https://github.com/yykamei/thwack/pull/227) ([yykamei](https://github.com/yykamei))

**Implemented enhancements:**

- Let a user copy the selected path [\#58](https://github.com/yykamei/thwack/issues/58)

**Merged pull requests:**

- Bump log from 0.4.14 to 0.4.16 [\#241](https://github.com/yykamei/thwack/pull/241) ([dependabot[bot]](https://github.com/apps/dependabot))
- Refactor cli.rs to handle mutable variables within a struct [\#239](https://github.com/yykamei/thwack/pull/239) ([yykamei](https://github.com/yykamei))
- Bump libc from 0.2.120 to 0.2.121 [\#237](https://github.com/yykamei/thwack/pull/237) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump crossterm from 0.23.0 to 0.23.1 [\#234](https://github.com/yykamei/thwack/pull/234) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.119 to 0.2.120 [\#231](https://github.com/yykamei/thwack/pull/231) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump git2 from 0.14.1 to 0.14.2 [\#229](https://github.com/yykamei/thwack/pull/229) ([dependabot[bot]](https://github.com/apps/dependabot))
- Migrate the edition from 2018 to 2021 [\#225](https://github.com/yykamei/thwack/pull/225) ([yykamei](https://github.com/yykamei))
- Bump git2 from 0.14.0 to 0.14.1 [\#221](https://github.com/yykamei/thwack/pull/221) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.6.0](https://github.com/yykamei/thwack/tree/v0.6.0) (2022-02-27)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.5.0...v0.6.0)

**Implemented enhancements:**

- Read the environment variables: THWACK\_LOG\_FILE and THWACK\_EXEC [\#218](https://github.com/yykamei/thwack/pull/218) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Separate Preferences and StatusLine [\#216](https://github.com/yykamei/thwack/pull/216) ([yykamei](https://github.com/yykamei))
- Bump git2 from 0.13.25 to 0.14.0 [\#211](https://github.com/yykamei/thwack/pull/211) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.5.0](https://github.com/yykamei/thwack/tree/v0.5.0) (2022-02-24)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.11...v0.5.0)

**Implemented enhancements:**

- Respect .gitignore [\#208](https://github.com/yykamei/thwack/pull/208) ([yykamei](https://github.com/yykamei))
- Support --log-file option to write logs [\#204](https://github.com/yykamei/thwack/pull/204) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Bump libc from 0.2.118 to 0.2.119 [\#206](https://github.com/yykamei/thwack/pull/206) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.11](https://github.com/yykamei/thwack/tree/v0.4.11) (2022-02-16)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.10...v0.4.11)

**Fixed bugs:**

- thwack cannot run on directories that contain inaccessible files [\#198](https://github.com/yykamei/thwack/issues/198)

**Merged pull requests:**

- Bump libc from 0.2.117 to 0.2.118 [\#200](https://github.com/yykamei/thwack/pull/200) ([dependabot[bot]](https://github.com/apps/dependabot))
- Run `cargo update` [\#194](https://github.com/yykamei/thwack/pull/194) ([yykamei](https://github.com/yykamei))
- Bump unicode-segmentation from 1.8.0 to 1.9.0 [\#190](https://github.com/yykamei/thwack/pull/190) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump crossterm from 0.22.1 to 0.23.0 [\#189](https://github.com/yykamei/thwack/pull/189) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.113 to 0.2.117 [\#188](https://github.com/yykamei/thwack/pull/188) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.10](https://github.com/yykamei/thwack/tree/v0.4.10) (2022-01-21)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.9...v0.4.10)

**Merged pull requests:**

- Bump libc from 0.2.111 to 0.2.113 [\#181](https://github.com/yykamei/thwack/pull/181) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump tempfile from 3.2.0 to 3.3.0 [\#179](https://github.com/yykamei/thwack/pull/179) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.9](https://github.com/yykamei/thwack/tree/v0.4.9) (2021-12-13)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.8...v0.4.9)

**Merged pull requests:**

- Bump libc from 0.2.108 to 0.2.111 [\#176](https://github.com/yykamei/thwack/pull/176) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.8](https://github.com/yykamei/thwack/tree/v0.4.8) (2021-12-03)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.7...v0.4.8)

**Merged pull requests:**

- Bump libc from 0.2.107 to 0.2.108 [\#172](https://github.com/yykamei/thwack/pull/172) ([dependabot[bot]](https://github.com/apps/dependabot))
- Make `buf!` receive multiple expressions [\#170](https://github.com/yykamei/thwack/pull/170) ([yykamei](https://github.com/yykamei))

## [v0.4.7](https://github.com/yykamei/thwack/tree/v0.4.7) (2021-11-08)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.6...v0.4.7)

**Fixed bugs:**

- Fix move down behavior [\#167](https://github.com/yykamei/thwack/pull/167) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Bump libc from 0.2.106 to 0.2.107 [\#166](https://github.com/yykamei/thwack/pull/166) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.6](https://github.com/yykamei/thwack/tree/v0.4.6) (2021-11-05)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.5...v0.4.6)

**Merged pull requests:**

- Bump libc from 0.2.105 to 0.2.106 [\#163](https://github.com/yykamei/thwack/pull/163) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.104 to 0.2.105 [\#161](https://github.com/yykamei/thwack/pull/161) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump crossterm from 0.21.0 to 0.22.1 [\#160](https://github.com/yykamei/thwack/pull/160) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.5](https://github.com/yykamei/thwack/tree/v0.4.5) (2021-10-19)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.4...v0.4.5)

**Merged pull requests:**

- Bump libc from 0.2.103 to 0.2.104 [\#158](https://github.com/yykamei/thwack/pull/158) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.4](https://github.com/yykamei/thwack/tree/v0.4.4) (2021-09-28)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.3...v0.4.4)

**Merged pull requests:**

- Bump libc from 0.2.102 to 0.2.103 [\#156](https://github.com/yykamei/thwack/pull/156) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.3](https://github.com/yykamei/thwack/tree/v0.4.3) (2021-09-18)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.2...v0.4.3)

**Merged pull requests:**

- Bump unicode-width from 0.1.8 to 0.1.9 [\#153](https://github.com/yykamei/thwack/pull/153) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.101 to 0.2.102 [\#152](https://github.com/yykamei/thwack/pull/152) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.100 to 0.2.101 [\#150](https://github.com/yykamei/thwack/pull/150) ([dependabot[bot]](https://github.com/apps/dependabot))

## [v0.4.2](https://github.com/yykamei/thwack/tree/v0.4.2) (2021-08-24)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.1...v0.4.2)

**Merged pull requests:**

- Bump crossterm from 0.20.0 to 0.21.0 [\#147](https://github.com/yykamei/thwack/pull/147) ([dependabot[bot]](https://github.com/apps/dependabot))
- Bump libc from 0.2.99 to 0.2.100 [\#146](https://github.com/yykamei/thwack/pull/146) ([dependabot[bot]](https://github.com/apps/dependabot))
- Add Buffer::normalize\_path to ignore the difference between Windows and Unix [\#144](https://github.com/yykamei/thwack/pull/144) ([yykamei](https://github.com/yykamei))

## [v0.4.1](https://github.com/yykamei/thwack/tree/v0.4.1) (2021-08-11)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.4.0...v0.4.1)

**Merged pull requests:**

- Bump libc from 0.2.98 to 0.2.99 [\#142](https://github.com/yykamei/thwack/pull/142) ([dependabot[bot]](https://github.com/apps/dependabot))
- \[refactor\] Add tests for query [\#139](https://github.com/yykamei/thwack/pull/139) ([yykamei](https://github.com/yykamei))

## [v0.4.0](https://github.com/yykamei/thwack/tree/v0.4.0) (2021-08-08)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.3.0...v0.4.0)

**Merged pull requests:**

- Add --status-line=\<absolute|relative|none\> [\#137](https://github.com/yykamei/thwack/pull/137) ([yykamei](https://github.com/yykamei))
- \[refafctor\] Add Terminal trait for testing [\#135](https://github.com/yykamei/thwack/pull/135) ([yykamei](https://github.com/yykamei))
- \[refactor\] Add show\_version.rs to test --version [\#133](https://github.com/yykamei/thwack/pull/133) ([yykamei](https://github.com/yykamei))
- \[refactor\] Add show\_help.rs for integration tests [\#131](https://github.com/yykamei/thwack/pull/131) ([yykamei](https://github.com/yykamei))

## [v0.3.0](https://github.com/yykamei/thwack/tree/v0.3.0) (2021-07-12)

[Full Changelog](https://github.com/yykamei/thwack/compare/v0.2.2...v0.3.0)

**Implemented enhancements:**

- Show help screen like nano\(1\) and let users enable/disable it [\#20](https://github.com/yykamei/thwack/issues/20)
- Allow to move selections with Ctrl-p and Ctrl-n [\#120](https://github.com/yykamei/thwack/pull/120) ([yykamei](https://github.com/yykamei))

**Fixed bugs:**

- Change selection when the resize event happens [\#128](https://github.com/yykamei/thwack/pull/128) ([yykamei](https://github.com/yykamei))
- Fix Resize event handling to update paths [\#123](https://github.com/yykamei/thwack/pull/123) ([yykamei](https://github.com/yykamei))

**Merged pull requests:**

- Refactor MatchedPath to return chunks of absolute [\#125](https://github.com/yykamei/thwack/pull/125) ([yykamei](https://github.com/yykamei))

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
