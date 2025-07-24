# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v1.5.1 (2025-07-24)

### Bug Fixes

 - <csr-id-0000096e79d9b0e99da9bbac4b3844dde167f335/> add tracing
 - <csr-id-0000095cb5dcc1e4924576e69a456a51f3bbba26/> repair all other commands
 - <csr-id-00000945e170cc7288581aad74ed288b70afff93/> rework subcommands

### Style

 - <csr-id-00000933ceddd6506b398e4e3aa9e5b9fe85f466/> cleanup
 - <csr-id-0000092f419a98d38c917ad480390da365aa4bad/> lint
 - <csr-id-000009146c5e1f2fe4f7997e7adc855c1af6ec78/> ignore a warning

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add tracing (0000096)
    - Repair all other commands (0000095)
    - Rework subcommands (0000094)
    - Cleanup (0000093)
    - Lint (0000092)
    - Ignore a warning (0000091)
</details>

## v1.5.0 (2025-07-23)

<csr-id-00000851eda9e06d87dfcfceb80f76d3a00dbbd7/>
<csr-id-0000083225da1f66fa4b61057fd0340bd00abcf5/>

### Chore

 - <csr-id-00000851eda9e06d87dfcfceb80f76d3a00dbbd7/> update lock file
 - <csr-id-0000083225da1f66fa4b61057fd0340bd00abcf5/> remove dupplicate feature

### New Features

 - <csr-id-0000088cab95089c8cfc124eb89f89bb9a1389ff/> create and delete mailboxes
 - <csr-id-000008751a5ae8b4df669d7b778ecedb882a4266/> add imap sub commands, first is list mailboxes

### Bug Fixes

 - <csr-id-0000089ab170b383496e0ea32e2779dd8ca3f897/> allow disabling the ratatui renderer
 - <csr-id-00000866226d9e10d3f29665b484103705ac15f5/> split out base configs from config
 - <csr-id-00000848a58024180b3ca4f7c19d42555ce60b55/> clippy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 20 calendar days.
 - 44 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.5.0 (6f9fb47)
    - Allow disabling the ratatui renderer (0000089)
    - Create and delete mailboxes (0000088)
    - Add imap sub commands, first is list mailboxes (0000087)
    - Split out base configs from config (0000086)
    - Update lock file (0000085)
    - Clippy (0000084)
    - Remove dupplicate feature (0000083)
</details>

## v1.4.4 (2025-06-08)

<csr-id-0000079a08f72310ae5e5a9fb2564d5da4d99864/>

### Chore

 - <csr-id-0000079a08f72310ae5e5a9fb2564d5da4d99864/> update lock file

### Bug Fixes

 - <csr-id-00000810f27f023da994b7d84c555955a8ff9046/> some new lints
 - <csr-id-0000080d75899efd4615fac3543658ee7468d665/> clippy rust 1.87

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.4.4 (812a963)
    - Some new lints (0000081)
    - Clippy rust 1.87 (0000080)
    - Update lock file (0000079)
</details>

## v1.4.3 (2025-03-05)

<csr-id-0000077ccb87ef520c2d9eb9dddc407a3ba4438a/>
<csr-id-000006918cf32438b7d0ee6e1e96121f20b53abd/>
<csr-id-00000681ad95657f45eed59efa3bd3294fe88c6c/>
<csr-id-00000674d74f7c1962a176a6f4035c7b4101335f/>

### Chore

 - <csr-id-0000077ccb87ef520c2d9eb9dddc407a3ba4438a/> update lock file
 - <csr-id-000006918cf32438b7d0ee6e1e96121f20b53abd/> update tempfile to 3.17.1
 - <csr-id-00000681ad95657f45eed59efa3bd3294fe88c6c/> slim release builds a bit
 - <csr-id-00000674d74f7c1962a176a6f4035c7b4101335f/> tomlfmt

### Bug Fixes

 - <csr-id-0000076e1d6190ed85b0fb6259f6433142ac9469/> update imap to 3.0.0-alpha.15
 - <csr-id-0000075e0b3a3cedcf85426ab1abd590a9bae213/> update size to 0.5.0
 - <csr-id-0000074f0a153eeb545ec12a86d6ac2c1da5d2b6/> update serde to 1.0.218
 - <csr-id-0000073a146a1a45e2bf4e947b1e1982355035c7/> update once_cell to 1.20.3
 - <csr-id-0000072a7606fca380e1aa04f8c341cc24c61b6f/> update clap to 4.5.31
 - <csr-id-000007180ef7e80b1c0f690404c3a374f1cd524b/> update chrono to 0.4.40
 - <csr-id-0000070b661e4786080ecf0a353c934708c9ab4b/> update anyhow to 1.0.97

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 48 days passed between releases.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.4.3 (1e36d3b)
    - Update lock file (0000077)
    - Update imap to 3.0.0-alpha.15 (0000076)
    - Update size to 0.5.0 (0000075)
    - Update serde to 1.0.218 (0000074)
    - Update once_cell to 1.20.3 (0000073)
    - Update clap to 4.5.31 (0000072)
    - Update chrono to 0.4.40 (0000071)
    - Update anyhow to 1.0.97 (0000070)
    - Update tempfile to 3.17.1 (0000069)
    - Slim release builds a bit (0000068)
    - Tomlfmt (0000067)
</details>

## v1.4.2 (2025-01-15)

<csr-id-0000064e2baeb8ed0020b2884fa7b71811adb3f2/>
<csr-id-00000596b042f8942f5ec57d21b9ccfe02f424dd/>
<csr-id-00000582ba77add2f190346a4491ec1ffe1297a6/>
<csr-id-00000573e3039b6f3cbc20948b42016b3032f9af/>
<csr-id-0000054b537566710c303f1b167327b9f6b67702/>
<csr-id-00000568f9b87b56eefd1fd812984d3239ab0c87/>
<csr-id-0000055b39a85bac66d46e599b302a9695669e58/>

### Chore

 - <csr-id-0000064e2baeb8ed0020b2884fa7b71811adb3f2/> update lock file
 - <csr-id-00000596b042f8942f5ec57d21b9ccfe02f424dd/> update tempfile to 3.15.0
 - <csr-id-00000582ba77add2f190346a4491ec1ffe1297a6/> add license

### Bug Fixes

 - <csr-id-0000063b9576e8c514e1941f2ba636d37769355c/> update serde to 1.0.217
 - <csr-id-0000062305b7ec11e70fc8a2c978d342bb6dbe4f/> update clap to 4.5.26
 - <csr-id-0000061cc4d385f1fee15b9c8617f8f85f6fe035/> update chrono to 0.4.39
 - <csr-id-00000601637abd9c5654de75dc159d8e7bd45e5b/> update anyhow to 1.0.95

### Other

 - <csr-id-00000573e3039b6f3cbc20948b42016b3032f9af/> add ci

### Style

 - <csr-id-0000054b537566710c303f1b167327b9f6b67702/> clippy

### Test

 - <csr-id-00000568f9b87b56eefd1fd812984d3239ab0c87/> more config tests
 - <csr-id-0000055b39a85bac66d46e599b302a9695669e58/> fix tests after anyhow

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 38 calendar days.
 - 38 days passed between releases.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.4.2 (d006534)
    - Release imap-tools v1.4.2 (64e2f2f)
    - Update lock file (0000064)
    - Update serde to 1.0.217 (0000063)
    - Update clap to 4.5.26 (0000062)
    - Update chrono to 0.4.39 (0000061)
    - Update anyhow to 1.0.95 (0000060)
    - Update tempfile to 3.15.0 (0000059)
    - Add license (0000058)
    - Add ci (0000057)
    - More config tests (0000056)
    - Fix tests after anyhow (0000055)
    - Clippy (0000054)
</details>

## v1.4.1 (2024-12-07)

<csr-id-00000526a5079c10eb082a33198b9507b1944762/>

### Chore

 - <csr-id-00000526a5079c10eb082a33198b9507b1944762/> update lock file

### Bug Fixes

 - <csr-id-00000510078bd05d01135b01406aa874e3b58fe6/> refactor renderers activation
 - <csr-id-000005065e7ac0817886802f09685f7d387717b3/> switch to using anyhow
 - <csr-id-0000049753eaf55bc4416915c56cda6339779267/> update clap to 4.5.23

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 3 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.4.1 (376261a)
    - Update lock file (0000052)
    - Refactor renderers activation (0000051)
    - Switch to using anyhow (0000050)
    - Update clap to 4.5.23 (0000049)
</details>

## v1.4.0 (2024-12-04)

### New Features

 - <csr-id-0000047c05f8db507ca61d8639bf8a9b5f6e3d8a/> convert find_dups to Renderer
 - <csr-id-00000462ab1ec92b8a6cea0fa69a5bb00815f8d0/> convert archive to Renderer
 - <csr-id-000004534c2937f6aa928df6f9a828347ad00edd/> convert clean to Renderer
 - <csr-id-000004426b28aae762e9c04e4ee7bc918cddb28e/> convert list to Renderer
 - <csr-id-0000042a18c49887b46338cb9ad8aa13c66c8eb6/> add a pretty terminal renderer based on ratatui

### Bug Fixes

 - <csr-id-00000439a769ae3acbc1d38d806c6a209327d134/> refactor errors a bit

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.4.0 (a7593bc)
    - Convert find_dups to Renderer (0000047)
    - Convert archive to Renderer (0000046)
    - Convert clean to Renderer (0000045)
    - Convert list to Renderer (0000044)
    - Refactor errors a bit (0000043)
    - Add a pretty terminal renderer based on ratatui (0000042)
</details>

## v1.3.2 (2024-12-01)

### Bug Fixes

 - <csr-id-0000040548864c38e7f0fe58774e6dfaa9c7c5b3/> update dependencies
 - <csr-id-0000040c8ebcaecadcdd9b48ed1d4004690653e2/> update dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 13 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.3.2 (565c7e0)
    - Update dependencies (0000040)
</details>

## v1.3.1 (2024-11-17)

<csr-id-0000039166f9224084d324170e72dead219bb382/>
<csr-id-0000033d0bcb5d10fa76d5639bef9f37108ced1b/>
<csr-id-00000384015332735dca79ad30079b0a4a5605ed/>

### Chore

 - <csr-id-0000039166f9224084d324170e72dead219bb382/> v1.3.1
 - <csr-id-0000033d0bcb5d10fa76d5639bef9f37108ced1b/> this is expected

### Bug Fixes

 - <csr-id-00000377ca784d3e8be66767e8747ac75d6f5195/> remove the lib, this is not a library
 - <csr-id-00000364a62be810db697f78de3ba84be71e6976/> bubble up our errors to main
 - <csr-id-00000352e920de24acb28ef12456f177b44bf265/> impl source for OurError
 - <csr-id-0000034a1618d365969f3f2454e8188176c76b5e/> refactor internal errors

### Test

 - <csr-id-00000384015332735dca79ad30079b0a4a5605ed/> add some tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 4 calendar days.
 - 6 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.3.1 (0000039)
    - Add some tests (0000038)
    - Remove the lib, this is not a library (0000037)
    - Bubble up our errors to main (0000036)
    - Impl source for OurError (0000035)
    - Refactor internal errors (0000034)
    - This is expected (0000033)
</details>

## v1.3.0 (2024-11-11)

<csr-id-00000328d25b5194a636f7257a5179a6d404b0c6/>

### Chore

 - <csr-id-00000328d25b5194a636f7257a5179a6d404b0c6/> v1.3.0

### New Features

 - <csr-id-00000318d6061a10b5445b0465cfe4154f159075/> make rustls/openssl features

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.3.0 (0000032)
    - Make rustls/openssl features (0000031)
</details>

## v1.2.0 (2024-11-09)

<csr-id-0000030e2e0f3d18ad449885d828cb32b455235f/>
<csr-id-00000292d6cf4a51f44df4a490f262ea0af35479/>

### Chore

 - <csr-id-0000030e2e0f3d18ad449885d828cb32b455235f/> v1.2.0
 - <csr-id-00000292d6cf4a51f44df4a490f262ea0af35479/> update lock file

### New Features

 - <csr-id-00000279c151fce8d34a9ca50727e92d5bf986e3/> use latest imap version+switch to rustls

### Bug Fixes

 - <csr-id-00000280fdf29dae30469d86266b28d63e5fb8ce/> stop messing with the terminal in find_dups

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.2.0 (0000030)
    - Update lock file (0000029)
    - Stop messing with the terminal in find_dups (0000028)
    - Use latest imap version+switch to rustls (0000027)
</details>

## v1.1.2 (2024-11-06)

<csr-id-0000026923ba236ad9c7396f24fe2655971a7187/>
<csr-id-000002580e3e43e86528d48230aada890c18a913/>
<csr-id-0000021d4d1c41beba5cb395a15067a7f9ac804e/>
<csr-id-000001999305c2a32214fb55cdd28402701bd768/>

### Chore

 - <csr-id-0000026923ba236ad9c7396f24fe2655971a7187/> v1.1.2
 - <csr-id-000002580e3e43e86528d48230aada890c18a913/> update lock file
 - <csr-id-0000021d4d1c41beba5cb395a15067a7f9ac804e/> shuffle some types/includes around

### Bug Fixes

 - <csr-id-00000240ff5d84e2e727682ca9caa422c6ffa0b7/> add a couple more aliases for find-dups
 - <csr-id-000002395b75258111096b523e4a32893677ad4e/> refactor a bit of code here too
 - <csr-id-000002283660546a9b52a7ce41dea28cfa1d0be3/> try and give a similar message when dry-run
 - <csr-id-0000020cd56633d3e2b30e0cd76bd50e3f49bfa6/> always use uid_* functions

### Refactor

 - <csr-id-000001999305c2a32214fb55cdd28402701bd768/> add an alias OurResult for Result<x, OurError>

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 1 day passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.1.2 (0000026)
    - Update lock file (0000025)
    - Add a couple more aliases for find-dups (0000024)
    - Refactor a bit of code here too (0000023)
    - Try and give a similar message when dry-run (0000022)
    - Shuffle some types/includes around (0000021)
    - Always use uid_* functions (0000020)
    - Add an alias OurResult for Result<x, OurError> (0000019)
</details>

## v1.1.1 (2024-11-05)

<csr-id-0000018e1249ac1ff1b4d76cb77a3967a96cece1/>
<csr-id-000001724dfb80f660c8c2fb277b8768c2ea4ac2/>
<csr-id-000001665c76fcb6cb652ee69f7dd1f17f33ace6/>
<csr-id-00000142ef7b70894811e79fdce929d15dad12ba/>

### Chore

 - <csr-id-0000018e1249ac1ff1b4d76cb77a3967a96cece1/> v1.1.1
 - <csr-id-000001724dfb80f660c8c2fb277b8768c2ea4ac2/> update lock file

### Documentation

 - <csr-id-0000013a1d35d3bc4b27524b798f3b7dd0038539/> fix short argument name

### Bug Fixes

 - <csr-id-0000015a9aa6ada766dc1f2cf441aac4e5174245/> replace unwrap's with ? or ok_or()? so we don't panic

### Other

 - <csr-id-000001665c76fcb6cb652ee69f7dd1f17f33ace6/> explain why those two unwrap are ok

### Refactor

 - <csr-id-00000142ef7b70894811e79fdce929d15dad12ba/> rename to OurError so it does not clash with others

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.1.1 (0000018)
    - Update lock file (0000017)
    - Explain why those two unwrap are ok (0000016)
    - Replace unwrap's with ? or ok_or()? so we don't panic (0000015)
    - Rename to OurError so it does not clash with others (0000014)
    - Fix short argument name (0000013)
</details>

## v1.1.0 (2024-11-04)

<csr-id-0000012f37424b28d1f828504f5245ac818466ff/>

### Chore

 - <csr-id-0000012f37424b28d1f828504f5245ac818466ff/> v1.1.0

### New Features

 - <csr-id-0000011fa73ff37474495eb0a722c37dfe506706/> add support for servers with no MOVE
 - <csr-id-00000080d6890b9211c5b7117a956a038b328b0e/> add capabilities check with caching

### Bug Fixes

 - <csr-id-00000101b856bc1847f96b463bf8c29e449e62f6/> quote mailboxes if they contain spaces or \ or "
 - <csr-id-0000009cb9cec77a8f231847b5a65db823f4cb3e/> check for UIDPLUS as we need it
 - <csr-id-0000007f6bc59b8e1829e7805dedb35af4c6931f/> display shorter archive mailbox
 - <csr-id-00000062f315793babcc2c92653c3045dffb02e7/> add missing documentation to the generic arguments
 - <csr-id-00000052418f6ccf9073489ad478621efc572228/> only do this computation when we actually need it

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 1 day passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - V1.1.0 (0000012)
    - Add support for servers with no MOVE (0000011)
    - Quote mailboxes if they contain spaces or \ or " (0000010)
    - Check for UIDPLUS as we need it (0000009)
    - Add capabilities check with caching (0000008)
    - Display shorter archive mailbox (0000007)
    - Add missing documentation to the generic arguments (0000006)
    - Only do this computation when we actually need it (0000005)
</details>

## v1.0.0 (2024-11-03)

### New Features

 - <csr-id-0000004d0e549bcc34a976e8788964b0424ff64e/> add archive
 - <csr-id-0000003a0ab3343eef31f38a21b492fc94346d19/> add clean
 - <csr-id-0000002ea109f59cd0b0dc9be3e7814968b8b201/> add find-dups
 - <csr-id-00000012d5ac978339ab8d92deaa314c4cab55af/> add base with list to test filters

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add archive (0000004)
    - Add clean (0000003)
    - Add find-dups (0000002)
    - Add base with list to test filters (0000001)
</details>

