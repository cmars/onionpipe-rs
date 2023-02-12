# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.1 (2023-02-12)

### Chore

 - <csr-id-88be3cb690686fdb5c1d1d76f6b5e06e1431e3d7/> update README

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release onionpipe v0.2.0 ([`6c505c9`](https://github.com/cmars/onionpipe-rs/commit/6c505c942cc9890a417f7775caff6a156ba19142))
    - update README ([`88be3cb`](https://github.com/cmars/onionpipe-rs/commit/88be3cb690686fdb5c1d1d76f6b5e06e1431e3d7))
</details>

## v0.2.0 (2023-02-12)

<csr-id-d096a5da2184ec04b1bbc1e02daf5bbc7c92250a/>
<csr-id-33b49d8f51496150fffae29f8d4ed746c0007f6e/>
<csr-id-f87b6b5a8b306f374ae9c4ea9a9c93abafb4e7f4/>
<csr-id-34b147be23f53e1b62bfa2f57301e0e9b86ac48f/>
<csr-id-dc64dbe89246a1a356a2a33d1fd29cecb9aff314/>
<csr-id-c9306cb34ecdd39393f65e20b83f13e8f671b66b/>
<csr-id-09760db9d14cd693b4e0f7f5784f48711ac1849b/>
<csr-id-ce286536a3833182ac650868d2263f426ef7fcf0/>
<csr-id-f85ff121415d0e482447d398b70a86fcac7b7f8a/>
<csr-id-88be3cb690686fdb5c1d1d76f6b5e06e1431e3d7/>

### Chore

 - <csr-id-d096a5da2184ec04b1bbc1e02daf5bbc7c92250a/> add local cargo bin to $PATH
 - <csr-id-33b49d8f51496150fffae29f8d4ed746c0007f6e/> github action
 - <csr-id-f87b6b5a8b306f374ae9c4ea9a9c93abafb4e7f4/> rename onion secret key field
   Make it clear this is a sensitive field.
 - <csr-id-34b147be23f53e1b62bfa2f57301e0e9b86ac48f/> separate library and CLI binary
 - <csr-id-dc64dbe89246a1a356a2a33d1fd29cecb9aff314/> add readme
 - <csr-id-c9306cb34ecdd39393f65e20b83f13e8f671b66b/> forward given exports, cleanup
   Create onions from the given exports rather than a hard-coded one.
   More graceful shutdown on interrupt.
   Organizing module usage.
 - <csr-id-09760db9d14cd693b4e0f7f5784f48711ac1849b/> structuring into types, annotate errors
 - <csr-id-ce286536a3833182ac650868d2263f426ef7fcf0/> initial commit
   Early proof of concept

### Chore

 - <csr-id-88be3cb690686fdb5c1d1d76f6b5e06e1431e3d7/> update README

### Chore

 - <csr-id-f85ff121415d0e482447d398b70a86fcac7b7f8a/> add changelog

### New Features

<csr-id-47c9f702b93b6a582bfbd9cb15190b13c86a71f0/>
<csr-id-2062d9a439e45d7ca8cf7e4c38ede9215a794059/>

 - <csr-id-52b4077cf2a4532d3eeadfcd32ac2e97f14c5872/> cli
   Drive-by fixes:
   - Fix import forwarding loop, should continue, not return on connection
   error

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 148 calendar days.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release onionpipe v0.2.0 ([`4b24e2e`](https://github.com/cmars/onionpipe-rs/commit/4b24e2e1081f69fb4e0e28efce259f3b0b979951))
    - add changelog ([`f85ff12`](https://github.com/cmars/onionpipe-rs/commit/f85ff121415d0e482447d398b70a86fcac7b7f8a))
    - Release onionpipe v0.2.0 ([`2b677d3`](https://github.com/cmars/onionpipe-rs/commit/2b677d3c7d00143c83a55f8e5c1328562e6667ae))
    - add local cargo bin to $PATH ([`d096a5d`](https://github.com/cmars/onionpipe-rs/commit/d096a5da2184ec04b1bbc1e02daf5bbc7c92250a))
    - Merge pull request #3 from cmars/feat/cli ([`5dc3f04`](https://github.com/cmars/onionpipe-rs/commit/5dc3f04522c952826a08b5045241b3d20cbc8230))
    - cli ([`52b4077`](https://github.com/cmars/onionpipe-rs/commit/52b4077cf2a4532d3eeadfcd32ac2e97f14c5872))
    - github action ([`33b49d8`](https://github.com/cmars/onionpipe-rs/commit/33b49d8f51496150fffae29f8d4ed746c0007f6e))
    - rename onion secret key field ([`f87b6b5`](https://github.com/cmars/onionpipe-rs/commit/f87b6b5a8b306f374ae9c4ea9a9c93abafb4e7f4))
    - Merge pull request #2 from cmars/feat/config ([`3ccfa64`](https://github.com/cmars/onionpipe-rs/commit/3ccfa64ac746757bc05beff1723d8f6ead367a6f))
    - config file ([`47c9f70`](https://github.com/cmars/onionpipe-rs/commit/47c9f702b93b6a582bfbd9cb15190b13c86a71f0))
    - separate library and CLI binary ([`34b147b`](https://github.com/cmars/onionpipe-rs/commit/34b147be23f53e1b62bfa2f57301e0e9b86ac48f))
    - implement imports ([`2062d9a`](https://github.com/cmars/onionpipe-rs/commit/2062d9a439e45d7ca8cf7e4c38ede9215a794059))
    - add readme ([`dc64dbe`](https://github.com/cmars/onionpipe-rs/commit/dc64dbe89246a1a356a2a33d1fd29cecb9aff314))
    - forward given exports, cleanup ([`c9306cb`](https://github.com/cmars/onionpipe-rs/commit/c9306cb34ecdd39393f65e20b83f13e8f671b66b))
    - structuring into types, annotate errors ([`09760db`](https://github.com/cmars/onionpipe-rs/commit/09760db9d14cd693b4e0f7f5784f48711ac1849b))
    - initial commit ([`ce28653`](https://github.com/cmars/onionpipe-rs/commit/ce286536a3833182ac650868d2263f426ef7fcf0))
</details>

