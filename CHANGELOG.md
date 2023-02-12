# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - update README ([`88be3cb`](https://github.com/cmars/onionpipe-rs/commit/88be3cb690686fdb5c1d1d76f6b5e06e1431e3d7))
</details>

<csr-unknown>
Fix import remote addr parsingImprove wrapped error textLeaving room for error backtracesstderr output, needs to be replaced with proper logging though<csr-unknown/>

