# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v1.10.1 (2026-04-28)

### Chore

 - <csr-id-fb9fc3d376a7e1e11ade63c03423f5ad0b8cea1f/> lock file maintenance
 - <csr-id-00002950dcd0acc581ea14c062d4ad70155da3fa/> set rust-version
 - <csr-id-a80a7d850a6be0cb0076a4bf0b9294ecdfa0e7a5/> lock file maintenance

### Bug Fixes

 - <csr-id-0000303094f96ec23e4d9fe8a24199a014aed073/> remove SingleOrArray and use a simple custom deserializer

### Other

 - <csr-id-00003040de6f80eae48d6710618a5691c0638c84/> use correct token
 - <csr-id-00003020854675634e299ec6061c08140402aadc/> unpin
 - <csr-id-00003010eaf229c64347102583a7f98b25916b86/> publishing should not be interrupted
 - <csr-id-0000298064a48e33c7b1e81c4d066a7735ad7d29/> a lot better
 - <csr-id-00002970e7d6586979c2d1c0ddb3be8a216b20a1/> better message
 - <csr-id-00002960201e8c2e7a73b1aa91b1f63543f9ea6d/> add publish via OIDC

### Test

 - <csr-id-000029904dc9616d0e56fbc4763673b0a5c6262f/> don't run when not in debug

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release.
 - 9 days passed between releases.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Use correct token (0000304)
    - Remove SingleOrArray and use a simple custom deserializer (0000303)
    - Unpin (0000302)
    - Publishing should not be interrupted (0000301)
    - Lock file maintenance (fb9fc3d)
    - Don't run when not in debug (0000299)
    - A lot better (0000298)
    - Better message (0000297)
    - Add publish via OIDC (0000296)
    - Set rust-version (0000295)
    - Lock file maintenance (a80a7d8)
</details>

## v1.10.0 (2026-04-19)

<csr-id-000029104b52f05e8a57825ae8d9b0ac3c8f48d9/>
<csr-id-4fd43e298495fe6b67886d501288123ed5711bbb/>
<csr-id-00002920863dadf7db2bceb09d9ff144325bce24/>
<csr-id-00002860ce5a52e2b6b1355025e775a1ba60afbd/>

### Chore

 - <csr-id-000029104b52f05e8a57825ae8d9b0ac3c8f48d9/> refactor tls cfg* tests a bit
 - <csr-id-4fd43e298495fe6b67886d501288123ed5711bbb/> lock file maintenance

### New Features

 - <csr-id-00002890af72c73f631283fcd7ffe56856f2ea74/> migrate from sync imap to async-imap on tokio
 - <csr-id-000028504d8b9d4b3a1e724abfd437c78909c557/> add a json output

### Bug Fixes

 - <csr-id-0000290041d17e5c064c3430ed49c54728487453/> change default feature to rustls
 - <csr-id-75fdd9f99cfd239c6ef38f183bc436828dc99268/> pin rust crate serde_json to =1.0.149
   | datasource | package    | from    | to      |
   | ---------- | ---------- | ------- | ------- |
   | crate      | serde_json | 1.0.149 | 1.0.149 |
 - <csr-id-00002870ff9889aca40d8bc215356ee12a994887/> refactor formatting
 - <csr-id-00002840599043b54927a0dc23bb05f1ec6d9ab0/> note that all those are 'static
 - <csr-id-0000283000179d2d2eda7947cc0b9bff2bf937de/> put the headers length in a constant
 - <csr-id-f6dadf4064b5dda1514316d2d2cb5a3fba824c46/> update rust crate clap to v4.6.1
   | datasource | package | from  | to    |
   | ---------- | ------- | ----- | ----- |
   | crate      | clap    | 4.6.0 | 4.6.1 |
 - <csr-id-0000281064f7f15f5c2b45593d9d493a434a0cdb/> force compile time check of the renderers rows length
 - <csr-id-000028008ca6723490d637ebacc5c4f2e0882cc1/> delegate formatting to std::fmt::from_fn

### Other

 - <csr-id-00002920863dadf7db2bceb09d9ff144325bce24/> better clippy

### Test

 - <csr-id-00002860ce5a52e2b6b1355025e775a1ba60afbd/> add unit tests for testable renderers

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release.
 - 11 days passed between releases.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.10.0 (4388c52)
    - Better clippy (0000292)
    - Refactor tls cfg* tests a bit (0000291)
    - Change default feature to rustls (0000290)
    - Migrate from sync imap to async-imap on tokio (0000289)
    - Pin rust crate serde_json to =1.0.149 (75fdd9f)
    - Refactor formatting (0000287)
    - Add unit tests for testable renderers (0000286)
    - Add a json output (0000285)
    - Note that all those are 'static (0000284)
    - Put the headers length in a constant (0000283)
    - Update rust crate clap to v4.6.1 (f6dadf4)
    - Force compile time check of the renderers rows length (0000281)
    - Delegate formatting to std::fmt::from_fn (0000280)
    - Lock file maintenance (4fd43e2)
</details>

## v1.9.0 (2026-04-08)

<csr-id-754207379260f6573a5ede2bf500ec449df49009/>
<csr-id-000026700e96439e9554d21e9375e870d525e096/>
<csr-id-00925453af7c272eb4f97a738726164388d3cc44/>
<csr-id-afe568bd6b2b2035a6d09bb4cedca367771b3f72/>
<csr-id-00002410a3a85fc9f1a8017fb2d36d30087f7b18/>
<csr-id-17d9f3f37abc358bb5391477d9a764916caa3131/>
<csr-id-75d322174b3ef280aae4d389740e32f7b62356a9/>
<csr-id-0000243076f04f38a14a08d36ba6f004bc3d1455/>
<csr-id-000024208857f63a01d7d5172a606ecc880e53f4/>
<csr-id-000025403a5aba4ba9b5f03384862f612a051c08/>
<csr-id-000025201be47a1a3187c2588d012e0b90150308/>
<csr-id-00002510880259df2c9c9517c352b8e7a1d007db/>
<csr-id-000025003c6999b1080e275c79968b8062c41fa4/>
<csr-id-0000273074ec8dd76d13cf1a21f095e02e435cfb/>
<csr-id-00002570e69f8ea44aaf68c0dce498cf698c558d/>
<csr-id-00002400bd15ee73ff03764d6a789642398a2a7c/>
<csr-id-000027203ead927da209cf04936ffbb1920b409e/>
<csr-id-00002710a38f03251bac1f6fb5fa4465f688262b/>
<csr-id-0000270020cc5fc3c1b971b7454868ea1d57f2c8/>
<csr-id-0000266020b576ca0576f8522d9ce3fb5321c8c5/>
<csr-id-00002650fae1bc9238e84bf452890ff723c51731/>
<csr-id-000026408c77280101badcd63edf47f56b42d53e/>
<csr-id-000026309dd443f0fc9a24ec95b24d417d98028c/>
<csr-id-0000262061f5a8a9efa13a76fd952f05222e2db3/>
<csr-id-00002610c1bef6aad96fde1370b2346b83fd7bf6/>
<csr-id-00002600355e9a8565be885dca14de7de8e74c3e/>
<csr-id-00002590fd0291a6a8db16fd638b6e9bf5447ba2/>
<csr-id-000025802f2bcdafc85be44cc9b04633659dba4e/>
<csr-id-000025609ccc5d2638b525be2a82769282105800/>
<csr-id-00002550eb965294cc61cb873c820620e2acfd79/>

### Chore

 - <csr-id-754207379260f6573a5ede2bf500ec449df49009/> lock file maintenance
 - <csr-id-000026700e96439e9554d21e9375e870d525e096/> utiliser cfg_attr pour n'avoir qu'une seule option
 - <csr-id-00925453af7c272eb4f97a738726164388d3cc44/> update rust crate insta to v1.47.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.47.1 | 1.47.2 |
 - <csr-id-afe568bd6b2b2035a6d09bb4cedca367771b3f72/> lock file maintenance
 - <csr-id-00002410a3a85fc9f1a8017fb2d36d30087f7b18/> typos
 - <csr-id-17d9f3f37abc358bb5391477d9a764916caa3131/> update rust crate insta to v1.47.1
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.47.0 | 1.47.1 |
 - <csr-id-75d322174b3ef280aae4d389740e32f7b62356a9/> update rust crate insta to v1.47.0
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.46.3 | 1.47.0 |

### New Features

 - <csr-id-0000269056541cd270ca5f033b5de7f6463c3e8e/> ajouter un renderer csv

### Bug Fixes

 - <csr-id-00002770c34f0d7dca5d4193706f67cb4587f4b9/> add sorting name and size by descending order
 - <csr-id-0000276058bf87fcba7ce2538bc2785ce401acec/> use clap's value_enum/ValueEnum to get help from enums
   Convert Mode to an enum for this to work.
 - <csr-id-00002750de65e7886b135390b714e9a24c26fc54/> add csv renderer to args
 - <csr-id-000026808c25a7f0a7b2daf9bb6c9c2c1b8ba570/> ajouter le renderer aux arguments
 - <csr-id-0000249061b6eb5a3058600ef986d2112f628257/> propagate unexpected IMAP errors in create/delete commands
   Previously, any IMAP error other than the expected NO response was
   printed to stdout and silently returned Ok(()), making failures
   invisible to callers and scripts. Unexpected errors now bail with
   a proper error.
 - <csr-id-00002480108f74cc2c914ed1c0e0533f1ac7b437/> unfold RFC 2822 header continuations before parsing Message-ID
   Folded headers (CRLF followed by whitespace) were not being unfolded,
   causing duplicates with multi-line Message-ID headers to go undetected.
 - <csr-id-000024701e705f893ce9e66cb661e5e065ba7c65/> use actual port in connection error message
   The error was hardcoding port 143 even when a different port was
   configured via config file or --port CLI argument.
 - <csr-id-00002460de32f264fb59ab707b5fe13543bfdee2/> use resolved dry_run from config in destructive commands
   When dry-run=true is set in the config file but not on the CLI,
   the renderer title showed "DRY-RUN" but the actual IMAP operations
   (SELECT/STORE/CLOSE) would still execute. Now the inner methods
   receive dry_run from config.base.dry_run, which merges both sources.

### Other

 - <csr-id-0000243076f04f38a14a08d36ba6f004bc3d1455/> fix
 - <csr-id-000024208857f63a01d7d5172a606ecc880e53f4/> fmt → nightly

### Performance

 - <csr-id-00002530e34ff6968c2ce611cb744b130060016f/> remove redundant terminal.clear() before ratatui re-init
   The clear() call immediately before try_init_with_options() was a
   no-op since reinitializing ratatui sets up a fresh terminal state.

### Refactor

 - <csr-id-000025403a5aba4ba9b5f03384862f612a051c08/> remove empty else branch workaround in base_config
   The if-else-if-else with an empty last branch existed only to satisfy
   clippy::else_if_without_else. Rewriting with clone_from + get_or_insert_with
   expresses the intent more clearly.
 - <csr-id-000025201be47a1a3187c2588d012e0b90150308/> name magic numbers in clean command
   300 (minimum message count) and 1_000_000 (minimum total size in
   bytes) are now named constants, making their intent clear.
 - <csr-id-00002510880259df2c9c9517c352b8e7a1d007db/> extract delete_uids method on Imap to remove duplication
   The select + uid_store(\Deleted) + close sequence was duplicated in
   clean and find_dups. It now lives as Imap::delete_uids.
 - <csr-id-000025003c6999b1080e275c79968b8062c41fa4/> deduplicate test_base() helper into test_helpers
   The identical test_base() function was copied verbatim in 7 test
   modules. It now lives in src/test_helpers.rs and is imported where
   needed.

### Style

 - <csr-id-0000273074ec8dd76d13cf1a21f095e02e435cfb/> fmt
 - <csr-id-00002570e69f8ea44aaf68c0dce498cf698c558d/> cleanup
 - <csr-id-00002400bd15ee73ff03764d6a789642398a2a7c/> lints

### Test

 - <csr-id-000027203ead927da209cf04936ffbb1920b409e/> really fix
 - <csr-id-00002710a38f03251bac1f6fb5fa4465f688262b/> fixup time sensitive tests
 - <csr-id-0000270020cc5fc3c1b971b7454868ea1d57f2c8/> utiliser le renderer csv pour les tests
 - <csr-id-0000266020b576ca0576f8522d9ce3fb5321c8c5/> merge ok/no and expect_command*
 - <csr-id-00002650fae1bc9238e84bf452890ff723c51731/> add commands
 - <csr-id-000026408c77280101badcd63edf47f56b42d53e/> add tests for non-dry-run archive with MOVE and COPY+DELETE fallback
 - <csr-id-000026309dd443f0fc9a24ec95b24d417d98028c/> add tests for empty search result and multi-rule iteration in clean
 - <csr-id-0000262061f5a8a9efa13a76fd952f05222e2db3/> add tests for folded headers, short Message-ID, no duplicates, and 3-way dedup
 - <csr-id-00002610c1bef6aad96fde1370b2346b83fd7bf6/> add tests for Sort::from_str and DiskUsage include/exclude_re filtering
 - <csr-id-00002600355e9a8565be885dca14de7de8e74c3e/> add tests for make_filter_re with only literals or only regex
 - <csr-id-00002590fd0291a6a8db16fd638b6e9bf5447ba2/> add tests for --port and -m/--mode CLI flags
 - <csr-id-000025802f2bcdafc85be44cc9b04633659dba4e/> add tests for Mode FromStr, Serialize, Deserialize, and Default
 - <csr-id-000025609ccc5d2638b525be2a82769282105800/> cleanup
 - <csr-id-00002550eb965294cc61cb873c820620e2acfd79/> add missing tests for list, exclude_re, and destructive paths
   - commands/list.rs: extract run() helper and add list_renders_mailboxes test
   - imap/list.rs: add list_exclude_re_filters_mailboxes test
   - clean.rs: add cleanup_destructive_large_old_mailbox test (non-dry-run)
   - find_dups.rs: add process_destructive_deletes_duplicates test (non-dry-run)
   - update insta snapshots for shifted line numbers from earlier refactors

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 41 commits contributed to the release.
 - 40 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.9.0 (20ed598)
    - Add sorting name and size by descending order (0000277)
    - Use clap's value_enum/ValueEnum to get help from enums (0000276)
    - Add csv renderer to args (0000275)
    - Lock file maintenance (7542073)
    - Fmt (0000273)
    - Really fix (0000272)
    - Fixup time sensitive tests (0000271)
    - Utiliser le renderer csv pour les tests (0000270)
    - Ajouter un renderer csv (0000269)
    - Ajouter le renderer aux arguments (0000268)
    - Utiliser cfg_attr pour n'avoir qu'une seule option (0000267)
    - Merge ok/no and expect_command* (0000266)
    - Add commands (0000265)
    - Add tests for non-dry-run archive with MOVE and COPY+DELETE fallback (0000264)
    - Add tests for empty search result and multi-rule iteration in clean (0000263)
    - Add tests for folded headers, short Message-ID, no duplicates, and 3-way dedup (0000262)
    - Add tests for Sort::from_str and DiskUsage include/exclude_re filtering (0000261)
    - Add tests for make_filter_re with only literals or only regex (0000260)
    - Add tests for --port and -m/--mode CLI flags (0000259)
    - Add tests for Mode FromStr, Serialize, Deserialize, and Default (0000258)
    - Cleanup (0000257)
    - Cleanup (0000256)
    - Add missing tests for list, exclude_re, and destructive paths (0000255)
    - Remove empty else branch workaround in base_config (0000254)
    - Remove redundant terminal.clear() before ratatui re-init (0000253)
    - Name magic numbers in clean command (0000252)
    - Extract delete_uids method on Imap to remove duplication (0000251)
    - Deduplicate test_base() helper into test_helpers (0000250)
    - Propagate unexpected IMAP errors in create/delete commands (0000249)
    - Unfold RFC 2822 header continuations before parsing Message-ID (0000248)
    - Use actual port in connection error message (0000247)
    - Use resolved dry_run from config in destructive commands (0000246)
    - Update rust crate insta to v1.47.2 (0092545)
    - Lock file maintenance (afe568b)
    - Fix (0000243)
    - Fmt → nightly (0000242)
    - Typos (0000241)
    - Lints (0000240)
    - Update rust crate insta to v1.47.1 (17d9f3f)
    - Update rust crate insta to v1.47.0 (75d3221)
</details>

## v1.8.3 (2026-03-25)

<csr-id-5ec2513a91c5964a1e61a088676bafb7d7b09386/>
<csr-id-08f403cb52e2f47fc51104c71bd38de733ee96e3/>

### Chore

 - <csr-id-5ec2513a91c5964a1e61a088676bafb7d7b09386/> lock file maintenance
 - <csr-id-08f403cb52e2f47fc51104c71bd38de733ee96e3/> lock file maintenance

### Bug Fixes

 - <csr-id-00002360ba0713fc288176861ddba155f42c632f/> return a more meaningfull error in the exhaustive case
 - <csr-id-000023505ea491e64e135fdbc8260cb654522699/> define a constant for minimum message id length
 - <csr-id-00002340ed950a6500a4ef10bffdf1db91950545/> always keep the first message
 - <csr-id-000023300f86464b084361900ece130c76b083be/> bail if no internal_date instead of archiving to jan 1970
 - <csr-id-00002320978c1cd0a520f874578c175d83a09f1e/> don't advertise tls if not compiled in
 - <csr-id-0000231030cd91a8547b958c3c501698d068f683/> refactor to avoid duplicate code
 - <csr-id-000023007303dbdd05d0297625c3573a864c6594/> allow specifying the port number
 - <csr-id-37bb74be771543b2da51780aed279c8adcb54840/> update rust crate tracing-subscriber to v0.3.23
   | datasource | package            | from   | to     |
   | ---------- | ------------------ | ------ | ------ |
   | crate      | tracing-subscriber | 0.3.22 | 0.3.23 |
 - <csr-id-fd7793d1faa0a07c84b4a358b6261bb0248a96e2/> update rust crate clap to v4.6.0
   | datasource | package | from   | to    |
   | ---------- | ------- | ------ | ----- |
   | crate      | clap    | 4.5.61 | 4.6.0 |
 - <csr-id-5ad3ceebf323f4f6f1e712baed8006243b18a509/> update rust crate clap to v4.5.61
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.60 | 4.5.61 |

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release.
 - 13 days passed between releases.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.8.3 (43980a4)
    - Return a more meaningfull error in the exhaustive case (0000236)
    - Define a constant for minimum message id length (0000235)
    - Always keep the first message (0000234)
    - Bail if no internal_date instead of archiving to jan 1970 (0000233)
    - Don't advertise tls if not compiled in (0000232)
    - Refactor to avoid duplicate code (0000231)
    - Allow specifying the port number (0000230)
    - Lock file maintenance (5ec2513)
    - Lock file maintenance (08f403c)
    - Update rust crate tracing-subscriber to v0.3.23 (37bb74b)
    - Update rust crate clap to v4.6.0 (fd7793d)
    - Update rust crate clap to v4.5.61 (5ad3cee)
</details>

## v1.8.2 (2026-03-11)

<csr-id-6cd247154bb9af506101b12f7c41e4165c908646/>
<csr-id-a0f5cb71df8ae2028e41e786c82574e72fc2a860/>
<csr-id-92c9a4b85fb6d7315a688662f433ca73842c561e/>
<csr-id-00002160ca4d84a323511a864797a3d75e49dea6/>
<csr-id-7372968dbf42f8384bb52c7188e34d52f4be3461/>
<csr-id-f766398b0568492f0cf2ec31f758c72abcc8b66f/>
<csr-id-669b32174bf6a4e7e8a2d78339af1f46a8776970/>
<csr-id-0000214074a9643c092afe589811cfb449beb9b5/>
<csr-id-00002230801c1907bbb2bd8ffeff176b96aec18e/>

### Chore

 - <csr-id-6cd247154bb9af506101b12f7c41e4165c908646/> update rust crate tempfile to v3.27.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.26.0 | 3.27.0 |
 - <csr-id-a0f5cb71df8ae2028e41e786c82574e72fc2a860/> lock file maintenance
 - <csr-id-92c9a4b85fb6d7315a688662f433ca73842c561e/> lock file maintenance
 - <csr-id-00002160ca4d84a323511a864797a3d75e49dea6/> don't use turbofish on assignments
 - <csr-id-7372968dbf42f8384bb52c7188e34d52f4be3461/> update rust crate tempfile to v3.26.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.25.0 | 3.26.0 |
 - <csr-id-f766398b0568492f0cf2ec31f758c72abcc8b66f/> lock file maintenance
 - <csr-id-669b32174bf6a4e7e8a2d78339af1f46a8776970/> lock file maintenance

### Bug Fixes

 - <csr-id-00002220c2ef62b8f604cef1ba98f4f27b3f2945/> don't panic needlessly, an imap error will be raised if it happens
 - <csr-id-00002210aad6f21242f5f3a12a545cdefdb156b0/> don't truncate non utf-8 password
 - <csr-id-000022001faa05e3756a9f83eec26b8e319a9da1/> small typos
 - <csr-id-fde4b3abf251261e517937b58672e26b78562ab8/> update rust crate chrono to v0.4.44
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | chrono  | 0.4.43 | 0.4.44 |
 - <csr-id-1d21d24e94ca3805d8f335df325eadef2a8f78c8/> update rust crate clap to v4.5.60
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.59 | 4.5.60 |
 - <csr-id-e37213389427824a1a99c1233df40af7da139b36/> update rust crate clap to v4.5.59
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.58 | 4.5.59 |
 - <csr-id-6737a9c57c232ecaf2ebd144ee4d3e1cfaa9cd7f/> update rust crate indicatif to v0.18.4
   | datasource | package   | from   | to     |
   | ---------- | --------- | ------ | ------ |
   | crate      | indicatif | 0.18.3 | 0.18.4 |
 - <csr-id-2b80aba9403ea701b8cda470b1fb006035311a66/> update rust crate clap to v4.5.58
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.57 | 4.5.58 |

### Style

 - <csr-id-0000214074a9643c092afe589811cfb449beb9b5/> fmt

### Test

 - <csr-id-00002230801c1907bbb2bd8ffeff176b96aec18e/> add tests for commands

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release.
 - 28 days passed between releases.
 - 17 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.8.2 (cf14130)
    - Add tests for commands (0000223)
    - Don't panic needlessly, an imap error will be raised if it happens (0000222)
    - Don't truncate non utf-8 password (0000221)
    - Small typos (0000220)
    - Update rust crate tempfile to v3.27.0 (6cd2471)
    - Lock file maintenance (a0f5cb7)
    - Lock file maintenance (92c9a4b)
    - Don't use turbofish on assignments (0000216)
    - Update rust crate tempfile to v3.26.0 (7372968)
    - Fmt (0000214)
    - Update rust crate chrono to v0.4.44 (fde4b3a)
    - Lock file maintenance (f766398)
    - Update rust crate clap to v4.5.60 (1d21d24)
    - Update rust crate clap to v4.5.59 (e372133)
    - Lock file maintenance (669b321)
    - Update rust crate indicatif to v0.18.4 (6737a9c)
    - Update rust crate clap to v4.5.58 (2b80aba)
</details>

## v1.8.1 (2026-02-11)

<csr-id-025d5088a490cc9ae9cf7efb3c650ec4e1d4772c/>
<csr-id-cdef790c16cd063356dd38bf077fc7a2b7eb6e90/>
<csr-id-f7d1266e3fac234eef0a629608cbdb32a93cb487/>
<csr-id-a407af20d3cee41e9ad12a5f9fd98cb66434178c/>
<csr-id-d69389789e7aa40b9cde8cc4472e8d14c75a7244/>
<csr-id-afcdcfbcbb6ef27ca3cabb3fde57721978e8ccd4/>
<csr-id-54204abcd19c14bdd8209f563b363ceadef61b69/>
<csr-id-25db08f1d861ad2733417539f3491bc4565aad58/>
<csr-id-aa0f49343c9819d99af9f4c5e30d27dfed19a2fd/>
<csr-id-0000189076babb842e0fceb6e2d652d303aaccf9/>

### Chore

 - <csr-id-025d5088a490cc9ae9cf7efb3c650ec4e1d4772c/> update rust crate tempfile to v3.25.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.24.0 | 3.25.0 |
 - <csr-id-cdef790c16cd063356dd38bf077fc7a2b7eb6e90/> lock file maintenance
 - <csr-id-f7d1266e3fac234eef0a629608cbdb32a93cb487/> update rust crate insta to v1.46.3
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.46.2 | 1.46.3 |
 - <csr-id-a407af20d3cee41e9ad12a5f9fd98cb66434178c/> lock file maintenance
 - <csr-id-d69389789e7aa40b9cde8cc4472e8d14c75a7244/> update rust crate insta to v1.46.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.46.1 | 1.46.2 |
 - <csr-id-afcdcfbcbb6ef27ca3cabb3fde57721978e8ccd4/> lock file maintenance
 - <csr-id-54204abcd19c14bdd8209f563b363ceadef61b69/> lock file maintenance
 - <csr-id-25db08f1d861ad2733417539f3491bc4565aad58/> update rust crate insta to v1.46.1
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.46.0 | 1.46.1 |
 - <csr-id-aa0f49343c9819d99af9f4c5e30d27dfed19a2fd/> lock file maintenance

### Bug Fixes

 - <csr-id-3889e63425ec793d1dda3adfa46b4d5d4318a387/> update rust crate regex to v1.12.3
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | regex   | 1.12.2 | 1.12.3 |
 - <csr-id-9afcc7b40f0734a6aa273702b11074961d58042a/> update rust crate clap to v4.5.57
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.56 | 4.5.57 |
 - <csr-id-1cb079faa0550c802392823e66201edc2e6bf8a7/> update rust crate exn to v0.3.0
   | datasource | package | from  | to    |
   | ---------- | ------- | ----- | ----- |
   | crate      | exn     | 0.2.1 | 0.3.0 |
 - <csr-id-a5d21f25811d9baa8f7c83182e44a5d39f200f42/> update rust crate clap to v4.5.56
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.55 | 4.5.56 |
 - <csr-id-e243172cfc559ef988a4f2980b375dd697fb1dd1/> update rust crate clap to v4.5.55
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.54 | 4.5.55 |
 - <csr-id-671ed29add33e120aec35ef5e10db62f3d3695f9/> update rust crate chrono to v0.4.43
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | chrono  | 0.4.42 | 0.4.43 |
 - <csr-id-a026b620e035b944647fd05a62144a0fba6d3abc/> pin dependencies
   | datasource | package     | from  | to    |
   | ---------- | ----------- | ----- | ----- |
   | crate      | derive_more | 2.1.1 | 2.1.1 |
   | crate      | exn         | 0.2.1 | 0.2.1 |

### Test

 - <csr-id-0000189076babb842e0fceb6e2d652d303aaccf9/> fix tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 34 calendar days.
 - 34 days passed between releases.
 - 17 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.8.1 (891031d)
    - Update rust crate tempfile to v3.25.0 (025d508)
    - Lock file maintenance (cdef790)
    - Update rust crate regex to v1.12.3 (3889e63)
    - Update rust crate clap to v4.5.57 (9afcc7b)
    - Update rust crate insta to v1.46.3 (f7d1266)
    - Lock file maintenance (a407af2)
    - Update rust crate exn to v0.3.0 (1cb079f)
    - Update rust crate insta to v1.46.2 (d693897)
    - Update rust crate clap to v4.5.56 (a5d21f2)
    - Update rust crate clap to v4.5.55 (e243172)
    - Lock file maintenance (afcdcfb)
    - Lock file maintenance (54204ab)
    - Update rust crate insta to v1.46.1 (25db08f)
    - Update rust crate chrono to v0.4.43 (671ed29)
    - Lock file maintenance (aa0f493)
    - Pin dependencies (a026b62)
    - Fix tests (0000189)
</details>

## v1.8.0 (2026-01-07)

<csr-id-5fbcadda9869538e56aa1c2ace0013f091c7866d/>
<csr-id-98488715fa04344a1f1fea433b984dd129b48a87/>
<csr-id-000018605e94ee60885a4f1b0c64333736e6fe47/>

### Chore

 - <csr-id-5fbcadda9869538e56aa1c2ace0013f091c7866d/> lock file maintenance
 - <csr-id-98488715fa04344a1f1fea433b984dd129b48a87/> update rust crate insta to v1.46.0
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.45.1 | 1.46.0 |

### New Features

 - <csr-id-00001870d16b9cbb2a84f45d9d5205057045c67f/> move from eyre to exn for better errors

### Style

 - <csr-id-000018605e94ee60885a4f1b0c64333736e6fe47/> rename lint

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
    - Release imap-tools v1.8.0 (2094c07)
    - Move from eyre to exn for better errors (0000187)
    - Rename lint (0000186)
    - Lock file maintenance (5fbcadd)
    - Update rust crate insta to v1.46.0 (9848871)
</details>

## v1.7.4 (2026-01-03)

<csr-id-592e571cb6ef432ab17998dcc7adac6117191d81/>
<csr-id-a6635f40cdd01ccf7b68bf4344a9967ee7c059ad/>
<csr-id-a576bbaaeb17ea69bda5b080b7401ff176e7369d/>
<csr-id-65cf3dde105b783c1cbf392647d32f9aea8b6793/>
<csr-id-540f7474029a9c9b09e0d5de7602b0773913d383/>
<csr-id-993c83eb6c284d5528c78bcadcc0c5e4aa089c35/>
<csr-id-7a005d4cb69eaef31543efa74ab8afb6ad560f01/>
<csr-id-97a26adb5223c5b571b52e6ef2dff2f06e119a5d/>
<csr-id-db011e80c8ca787c95b20d5fb6c265cc1a765d74/>
<csr-id-5c545371fa5310bee12c55ea67fc1d464d6ae90f/>
<csr-id-ed6e1259eee99f75c9dadc7d1d0c3b79ddb0de23/>
<csr-id-9d065bc3e7ebf5e25fe50a3bdea87db693c75f48/>
<csr-id-60ce5af7c76593f4f7a81f1d8c1060aadaef3a15/>
<csr-id-cd62621313b5f0d2d22caed9dacbb73a47f8f2f5/>
<csr-id-8cd1bb84bab91784bd1df38aa59c9457e63e9ff0/>
<csr-id-344a77d99bb670fd7d1b307bd93c112e6354ce11/>
<csr-id-2fa69f3b2b74fde8a8bd8bd9662bc13d629a67a0/>

### Chore

 - <csr-id-592e571cb6ef432ab17998dcc7adac6117191d81/> update rust crate insta to v1.45.1
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.44.3 | 1.45.1 |
 - <csr-id-a6635f40cdd01ccf7b68bf4344a9967ee7c059ad/> update rust crate tempfile to v3.24.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.23.0 | 3.24.0 |
 - <csr-id-a576bbaaeb17ea69bda5b080b7401ff176e7369d/> lock file maintenance
 - <csr-id-65cf3dde105b783c1cbf392647d32f9aea8b6793/> lock file maintenance
 - <csr-id-540f7474029a9c9b09e0d5de7602b0773913d383/> lock file maintenance
 - <csr-id-993c83eb6c284d5528c78bcadcc0c5e4aa089c35/> lock file maintenance
 - <csr-id-7a005d4cb69eaef31543efa74ab8afb6ad560f01/> update rust crate insta to v1.44.3
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.44.2 | 1.44.3 |
 - <csr-id-97a26adb5223c5b571b52e6ef2dff2f06e119a5d/> update rust crate insta to v1.44.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.44.1 | 1.44.2 |
 - <csr-id-db011e80c8ca787c95b20d5fb6c265cc1a765d74/> lock file maintenance
 - <csr-id-5c545371fa5310bee12c55ea67fc1d464d6ae90f/> update rust crate insta to v1.44.1
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.43.2 | 1.44.1 |
 - <csr-id-ed6e1259eee99f75c9dadc7d1d0c3b79ddb0de23/> lock file maintenance
 - <csr-id-9d065bc3e7ebf5e25fe50a3bdea87db693c75f48/> lock file maintenance
 - <csr-id-60ce5af7c76593f4f7a81f1d8c1060aadaef3a15/> lock file maintenance
 - <csr-id-cd62621313b5f0d2d22caed9dacbb73a47f8f2f5/> lock file maintenance
 - <csr-id-8cd1bb84bab91784bd1df38aa59c9457e63e9ff0/> lock file maintenance
 - <csr-id-344a77d99bb670fd7d1b307bd93c112e6354ce11/> lock file maintenance
 - <csr-id-2fa69f3b2b74fde8a8bd8bd9662bc13d629a67a0/> lock file maintenance

### Bug Fixes

 - <csr-id-96a1c92bf8e33c1ecc046d408f896fa3a6ac9ba9/> update rust crate clap to v4.5.54
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.53 | 4.5.54 |
 - <csr-id-2f447df3fc9e9cc517a9b0d66da20c6802335640/> update rust crate ratatui to v0.30.0
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | ratatui | 0.29.0 | 0.30.0 |
 - <csr-id-54cce0674e4483623ebc1f04de982da190216608/> update rust crate tracing to v0.1.44
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | tracing | 0.1.43 | 0.1.44 |
 - <csr-id-ba049979c7d7f77d3ee0dffffa906f62b573beca/> update rust crate shell-words to v1.1.1
   | datasource | package     | from  | to    |
   | ---------- | ----------- | ----- | ----- |
   | crate      | shell-words | 1.1.0 | 1.1.1 |
 - <csr-id-b99dac99a118a14c5d4fd8ce47b48eb96be6d6d9/> update tokio-tracing monorepo
   | datasource | package            | from   | to     |
   | ---------- | ------------------ | ------ | ------ |
   | crate      | tracing            | 0.1.41 | 0.1.43 |
   | crate      | tracing-subscriber | 0.3.20 | 0.3.22 |
 - <csr-id-1e312aaa308f50add0472c2fc890f68d081c6d28/> update rust crate clap to v4.5.53
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.52 | 4.5.53 |
 - <csr-id-f14d60276a6046eb39f50c12ed7c72c52a744c10/> update rust crate clap to v4.5.52
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.51 | 4.5.52 |
 - <csr-id-60b608baee5de33ca9255ded6010d613e585d6ab/> update rust crate indicatif to v0.18.3
   | datasource | package   | from   | to     |
   | ---------- | --------- | ------ | ------ |
   | crate      | indicatif | 0.18.2 | 0.18.3 |
 - <csr-id-4e71a91bcc5bd27c738a5c1d384e301789c4337d/> update rust crate clap to v4.5.51
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.50 | 4.5.51 |
 - <csr-id-b57e19394af005567adc550026c653b6b0975e4f/> update rust crate indicatif to v0.18.2
   | datasource | package   | from   | to     |
   | ---------- | --------- | ------ | ------ |
   | crate      | indicatif | 0.18.1 | 0.18.2 |
 - <csr-id-6eddff7a42baaa9029e3de02cfabccea71042a16/> update rust crate indicatif to v0.18.1
   | datasource | package   | from   | to     |
   | ---------- | --------- | ------ | ------ |
   | crate      | indicatif | 0.18.0 | 0.18.1 |
 - <csr-id-9d0bba833262d2f7e70b3914430f6ca8d2b95b87/> update rust crate clap to v4.5.50
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.49 | 4.5.50 |
 - <csr-id-63099a3ce035a5da2e23c2922156df8bbbe31929/> update rust crate clap to v4.5.49
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.48 | 4.5.49 |
 - <csr-id-f44306213295ea1c728dc5f2ff25374b2b1e0d0e/> update rust crate regex to v1.12.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | regex   | 1.11.3 | 1.12.2 |

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 32 commits contributed to the release.
 - 31 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.7.4 (c8ee31e)
    - Update rust crate clap to v4.5.54 (96a1c92)
    - Update rust crate ratatui to v0.30.0 (2f447df)
    - Update rust crate insta to v1.45.1 (592e571)
    - Update rust crate tempfile to v3.24.0 (a6635f4)
    - Lock file maintenance (a576bba)
    - Lock file maintenance (65cf3dd)
    - Update rust crate tracing to v0.1.44 (54cce06)
    - Update rust crate shell-words to v1.1.1 (ba04997)
    - Lock file maintenance (540f747)
    - Lock file maintenance (993c83e)
    - Update tokio-tracing monorepo (b99dac9)
    - Update rust crate insta to v1.44.3 (7a005d4)
    - Update rust crate insta to v1.44.2 (97a26ad)
    - Lock file maintenance (db011e8)
    - Update rust crate insta to v1.44.1 (5c54537)
    - Update rust crate clap to v4.5.53 (1e312aa)
    - Update rust crate clap to v4.5.52 (f14d602)
    - Lock file maintenance (ed6e125)
    - Lock file maintenance (9d065bc)
    - Update rust crate indicatif to v0.18.3 (60b608b)
    - Lock file maintenance (60ce5af)
    - Lock file maintenance (cd62621)
    - Update rust crate clap to v4.5.51 (4e71a91)
    - Update rust crate indicatif to v0.18.2 (b57e193)
    - Lock file maintenance (8cd1bb8)
    - Update rust crate indicatif to v0.18.1 (6eddff7)
    - Update rust crate clap to v4.5.50 (9d0bba8)
    - Lock file maintenance (344a77d)
    - Update rust crate clap to v4.5.49 (63099a3)
    - Update rust crate regex to v1.12.2 (f443062)
    - Lock file maintenance (2fa69f3)
</details>

## v1.7.3 (2025-10-09)

<csr-id-cc78cb2238449a23d80e556ffcec3d9d96ac0b81/>
<csr-id-84c22964727ed4dd818b3804ea67c847fc907a06/>

### Chore

 - <csr-id-cc78cb2238449a23d80e556ffcec3d9d96ac0b81/> lock file maintenance
 - <csr-id-84c22964727ed4dd818b3804ea67c847fc907a06/> lock file maintenance

### Bug Fixes

 - <csr-id-0000150073bee007fc38ef2f69d63946633af27c/> create the archive mbx when it is a simple NoSelect folder

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 10 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.7.3 (839532d)
    - Create the archive mbx when it is a simple NoSelect folder (0000150)
    - Lock file maintenance (cc78cb2)
    - Lock file maintenance (84c2296)
</details>

## v1.7.2 (2025-09-28)

<csr-id-81e9ab2f0a2617982ccffc958503c276e4f1b1e4/>
<csr-id-928b2c36ae221c16b16076f4d15e2815b454efad/>

### Chore

 - <csr-id-81e9ab2f0a2617982ccffc958503c276e4f1b1e4/> update rust crate tempfile to v3.23.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.22.0 | 3.23.0 |
 - <csr-id-928b2c36ae221c16b16076f4d15e2815b454efad/> lock file maintenance

### Bug Fixes

 - <csr-id-a65afbd9d841fdc2b0533ed1830552c018d10e88/> update rust crate serde to v1.0.228
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.227 | 1.0.228 |
 - <csr-id-2338c618c28d8b8bd4c1f100606b3d28c8c8ca14/> update rust crate imap-proto to v0.16.6
   | datasource | package    | from   | to     |
   | ---------- | ---------- | ------ | ------ |
   | crate      | imap-proto | 0.16.5 | 0.16.6 |
 - <csr-id-0af5f73a48f842e003c03e7a1fe47a173954a6cb/> update rust crate serde to v1.0.227
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.226 | 1.0.227 |
 - <csr-id-8ebd5e15de5236b7be1b02ecb441fbb1d190a136/> update rust crate regex to v1.11.3
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | regex   | 1.11.2 | 1.11.3 |
 - <csr-id-e1c7c47f460d9e1b16f63e60d36ea7568ee01b2b/> update rust crate serde to v1.0.226
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.225 | 1.0.226 |

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 7 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.7.2 (8813108)
    - Update rust crate serde to v1.0.228 (a65afbd)
    - Update rust crate imap-proto to v0.16.6 (2338c61)
    - Update rust crate serde to v1.0.227 (0af5f73)
    - Update rust crate regex to v1.11.3 (8ebd5e1)
    - Update rust crate tempfile to v3.23.0 (81e9ab2)
    - Lock file maintenance (928b2c3)
    - Update rust crate serde to v1.0.226 (e1c7c47)
</details>

## v1.7.1 (2025-09-20)

<csr-id-16631a116dc25494df73dbb808e59c38740f3b13/>

### Chore

 - <csr-id-16631a116dc25494df73dbb808e59c38740f3b13/> lock file maintenance

### Bug Fixes

 - <csr-id-124ce70ce3e40a4af4d0143162a99c8c18aa4584/> update rust crate clap to v4.5.48
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.47 | 4.5.48 |
 - <csr-id-da659088291c12bf7c7b176cfddf11d5c6698c74/> update rust crate serde to v1.0.225
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.224 | 1.0.225 |
 - <csr-id-5a37c4f93c422dc34f7796d9e2b9cba0930426ee/> update rust crate serde to v1.0.224
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.223 | 1.0.224 |
 - <csr-id-c704eaba3871caaf7a2b42c180b653653aa8ffc7/> update rust crate serde to v1.0.223
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.221 | 1.0.223 |
 - <csr-id-94581536ffc675037a2044d0c4d70e150a7dd2ae/> update rust crate serde to v1.0.221
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.220 | 1.0.221 |
 - <csr-id-7a0702aa56456f0fb119222327e55bd047fdb31d/> update rust crate serde to v1.0.220
   | datasource | package | from    | to      |
   | ---------- | ------- | ------- | ------- |
   | crate      | serde   | 1.0.219 | 1.0.220 |

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 7 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.7.1 (58e75ee)
    - Update rust crate clap to v4.5.48 (124ce70)
    - Update rust crate serde to v1.0.225 (da65908)
    - Update rust crate serde to v1.0.224 (5a37c4f)
    - Lock file maintenance (16631a1)
    - Update rust crate serde to v1.0.223 (c704eab)
    - Update rust crate serde to v1.0.221 (9458153)
    - Update rust crate serde to v1.0.220 (7a0702a)
</details>

## v1.7.0 (2025-09-12)

<csr-id-60464dd1d7a5919c9f75b3e2740b292d421c4e1a/>

### Chore

 - <csr-id-60464dd1d7a5919c9f75b3e2740b292d421c4e1a/> update rust crate tempfile to v3.22.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.21.0 | 3.22.0 |

### New Features

 - <csr-id-000013001d64b23c7deb513b9df8b3b48baa1e17/> allow changing the tls mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.7.0 (e2569aa)
    - Allow changing the tls mode (0000130)
    - Update rust crate tempfile to v3.22.0 (60464dd)
</details>

## v1.6.5 (2025-09-09)

<csr-id-43a8096756660aa001193477b579250b6c6c2fdc/>
<csr-id-066d7a780284a2d0ebd18fbc1343a9adfb8cd7a1/>

### Chore

 - <csr-id-43a8096756660aa001193477b579250b6c6c2fdc/> lock file maintenance
 - <csr-id-066d7a780284a2d0ebd18fbc1343a9adfb8cd7a1/> update rust crate insta to v1.43.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | insta   | 1.43.1 | 1.43.2 |

### Bug Fixes

 - <csr-id-bc3398124b79069761022a4a5cc5406b3245e470/> update rust crate chrono to v0.4.42
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | chrono  | 0.4.41 | 0.4.42 |

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 5 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.5 (0780ed5)
    - Update rust crate chrono to v0.4.42 (bc33981)
    - Lock file maintenance (43a8096)
    - Update rust crate insta to v1.43.2 (066d7a7)
</details>

## v1.6.4 (2025-09-03)

<csr-id-9ff6ff5365313777a61bee7bde5653524e5f60bc/>
<csr-id-ecc3cb5213043626609e447f978a2b542f7aab16/>
<csr-id-00001140da2c96458e8fc615ff8305bbb11dd82e/>
<csr-id-00001150c4edbb89701be58395bccf8296d3986c/>

### Chore

 - <csr-id-9ff6ff5365313777a61bee7bde5653524e5f60bc/> lock file maintenance
 - <csr-id-ecc3cb5213043626609e447f978a2b542f7aab16/> update rust crate tempfile to v3.21.0
   | datasource | package  | from   | to     |
   | ---------- | -------- | ------ | ------ |
   | crate      | tempfile | 3.20.0 | 3.21.0 |
 - <csr-id-00001140da2c96458e8fc615ff8305bbb11dd82e/> add regex logging when tracing

### Bug Fixes

 - <csr-id-21e4be3cc4dbe1e1b2b4330771e42bc8a338bed2/> update rust crate clap to v4.5.47
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.46 | 4.5.47 |
 - <csr-id-93b2b9673b33c3f37b60b129c78b342aabae1667/> update rust crate tracing-subscriber to v0.3.20
   | datasource | package            | from   | to     |
   | ---------- | ------------------ | ------ | ------ |
   | crate      | tracing-subscriber | 0.3.19 | 0.3.20 |
 - <csr-id-5b371f28d58d8abd45235e7c8641c0a278b9a1be/> update rust crate regex to v1.11.2
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | regex   | 1.11.1 | 1.11.2 |
 - <csr-id-fabc3d92074eb0d1e530afcd7a6dd4fc0aed1bac/> update rust crate clap to v4.5.46
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | clap    | 4.5.45 | 4.5.46 |
 - <csr-id-651d05e45fc27335431af016387c6165b8b583f6/> pin dependencies
   | datasource | package | from   | to     |
   | ---------- | ------- | ------ | ------ |
   | crate      | eyre    | 0.6.12 | 0.6.12 |
   | crate      | insta   | 1.43.1 | 1.43.1 |
 - <csr-id-00001130921ac6cb739cf83568f466baebe5c969/> the imap crate is not optional

### Test

 - <csr-id-00001150c4edbb89701be58395bccf8296d3986c/> fix tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 17 calendar days.
 - 18 days passed between releases.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.4 (88c1c77)
    - Update rust crate clap to v4.5.47 (21e4be3)
    - Lock file maintenance (9ff6ff5)
    - Update rust crate tempfile to v3.21.0 (ecc3cb5)
    - Update rust crate tracing-subscriber to v0.3.20 (93b2b96)
    - Update rust crate regex to v1.11.2 (5b371f2)
    - Update rust crate clap to v4.5.46 (fabc3d9)
    - Pin dependencies (651d05e)
    - Add renovate.json (7b9458f)
    - Fix tests (0000115)
    - Add regex logging when tracing (0000114)
    - The imap crate is not optional (0000113)
</details>

## v1.6.3 (2025-08-15)

<csr-id-00001110d007118fe17b4e31859842d52ab3393d/>

### Chore

 - <csr-id-00001110d007118fe17b4e31859842d52ab3393d/> change how we depend on ratatui

### Bug Fixes

 - <csr-id-0000110050b1fa34a9cc9eac7313d0daed9b6724/> remove profile.release, it should be up to the user
 - <csr-id-0000109054556ff12d31e0f2a325ea0e2eaf8731/> move from anyhow to eyre
 - <csr-id-00001080001740f8d3a5c569b001a1b85361dad9/> simplify the tracing/not tracing for main
 - <csr-id-00001070c404e36f4c579d6dd26ed4dde9106656/> update clap to 4.5.45

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.3 (daccb93)
    - Change how we depend on ratatui (0000111)
    - Remove profile.release, it should be up to the user (0000110)
    - Move from anyhow to eyre (0000109)
    - Simplify the tracing/not tracing for main (0000108)
    - Update clap to 4.5.45 (0000107)
</details>

## v1.6.2 (2025-08-15)

<csr-id-00001050acaba101d9e86a2ee7de8fb57881f157/>

### Chore

 - <csr-id-00001050acaba101d9e86a2ee7de8fb57881f157/> update lock file

### Bug Fixes

 - <csr-id-00001040450b3c9792009753f4984cdec4d4fc22/> fix build with tracing

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 16 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.2 (b3d0e47)
    - Update lock file (0000105)
    - Fix build with tracing (0000104)
</details>

## v1.6.1 (2025-07-30)

### Bug Fixes

 - <csr-id-00001010c98ea54c87f41d70fdc103be2e9e596b/> add a progress bar

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.1 (92ce253)
    - Fix(deps) update clap to 4.5.42 (+pin deps) (0000102)
    - Add a progress bar (0000101)
</details>

## v1.6.0 (2025-07-29)

<csr-id-00000990adacbe0f266bd62eaa94ffc497dda1fc/>

### Chore

 - <csr-id-00000990adacbe0f266bd62eaa94ffc497dda1fc/> update lock file

### New Features

 - <csr-id-00000980b2cc94f338803989527664c195c3bad1/> add a `du(1)` like tool

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 5 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.6.0 (3aee95b)
    - Update lock file (0000099)
    - Add a `du(1)` like tool (0000098)
</details>

## v1.5.1 (2025-07-24)

<csr-id-00000933ceddd6506b398e4e3aa9e5b9fe85f466/>
<csr-id-0000092f419a98d38c917ad480390da365aa4bad/>
<csr-id-000009146c5e1f2fe4f7997e7adc855c1af6ec78/>

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

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release imap-tools v1.5.1 (5bbfe01)
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

