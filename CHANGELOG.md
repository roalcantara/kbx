# Changelog

Every release of kbx, written by semantic-release from the commit messages (docs/adr/0007-releases.md).

## [0.6.0](https://github.com/roalcantara/kbx/compare/v0.5.0...v0.6.0) (2026-11-08)

### Features

* **app:** Add a navigator: a stack of places and the overlay ([59ff6af](https://github.com/roalcantara/kbx/commit/59ff6af8834fba75cdf8df6f7159ff51ec69e391))
* **app:** Add the catalog and its overrides (Plan 8b) ([d896f6e](https://github.com/roalcantara/kbx/commit/d896f6e3df3cf814591dca6198ab58037824e346))
* **app:** Draw the panel always dark with rounded corners ([42fcd3e](https://github.com/roalcantara/kbx/commit/42fcd3ef56b56814828450b749cc36134a52ea2d))
* **app:** Find the Skills command without a shell ([4469210](https://github.com/roalcantara/kbx/commit/44692107a71d49df21c53eefdb4a6964fcaf0864))
* **app:** Inspect and act on a refresh report's rows ([17d9b00](https://github.com/roalcantara/kbx/commit/17d9b00e18628729994df1fa8be3c116796ff596))
* **app:** List screen components, header, footer and row (Plan 8d) ([982e6f3](https://github.com/roalcantara/kbx/commit/982e6f3633d11310b4868b125fc94080944ac9a0))
* **app:** List the report's rows in a GPUI Kit list ([4503a5f](https://github.com/roalcantara/kbx/commit/4503a5fea75f4dcd827fd09360bc8774e70acf42))
* **app:** Move the keys into a split's pane with tab ([bfcad8c](https://github.com/roalcantara/kbx/commit/bfcad8ccf9b9ef948a0c092a229ca2fd2f5691de))
* **app:** Navigate the refresh report like the list ([475e275](https://github.com/roalcantara/kbx/commit/475e2755f8c53f519a7cc2fba6f22fa31fa09571))
* **app:** Open a report row as a place that scrolls by keyboard ([31489e0](https://github.com/roalcantara/kbx/commit/31489e0ff4d72ab7ebba9d1a9ec60d457e652f81))
* **app:** Open files with an editor template ([0e06d6b](https://github.com/roalcantara/kbx/commit/0e06d6bc838d98a8ef7b6ecab459381097c39af9))
* **app:** Read an editor template from config.toml ([c71fb88](https://github.com/roalcantara/kbx/commit/c71fb886494e304e37ca316caca1f0d4a4e3596d))
* **bench:** Add probe window-id for window captures ([c58ab91](https://github.com/roalcantara/kbx/commit/c58ab9115ddca7728fec0d759ba0f9b8af9dac12))
* **catalog:** Add the report's actions, keys and words ([06593bb](https://github.com/roalcantara/kbx/commit/06593bb8498d46a863fb8ca91705fbcbc1e5c022))
* **catalog:** Add the Skills type and the scoped refresh actions ([1b96cd2](https://github.com/roalcantara/kbx/commit/1b96cd23f499a480c90358d124b1a18f935a0e00))
* **cli:** Refresh one source with kb --sync --source ([a41d098](https://github.com/roalcantara/kbx/commit/a41d098d10198a44773e5f9a6e974ce7561d179d))
* **design:** Adopt the Neon Dusk colours ([35e28c1](https://github.com/roalcantara/kbx/commit/35e28c12c592aab878ac9da1aec23d2a3aa79ecf)), closes [#38265C](https://github.com/roalcantara/kbx/issues/38265C) [#382654](https://github.com/roalcantara/kbx/issues/382654)
* **design:** Bundle Inter and JetBrains Mono ([0888aeb](https://github.com/roalcantara/kbx/commit/0888aebc2f098ce5028d4148c97980ae25e0f680))
* **design:** Capture every screen on the showcase corpus ([969d420](https://github.com/roalcantara/kbx/commit/969d4208f92c761444254d28ca0252966025cd30))
* **design:** Generate GPUI Kit's theme from the design tokens ([b1f7bdf](https://github.com/roalcantara/kbx/commit/b1f7bdf647bf42d51214815ad87fcea903b06afc))
* **design:** Put every spacing on the ramp with one content edge ([7b4e321](https://github.com/roalcantara/kbx/commit/7b4e3215c68cbf72d9f52f1707bf82424ee40656))
* **design:** Put the design tokens in DESIGN.md's front matter ([7f99875](https://github.com/roalcantara/kbx/commit/7f998753d69914a6e038ee425174840fb9bf3b81))
* **design:** Validate the design tokens in DESIGN.md ([634ed1b](https://github.com/roalcantara/kbx/commit/634ed1bbbe23c2cf11fa407bb67b99197db31d1f))
* **domain:** Locate a failed file at its line and column ([5835888](https://github.com/roalcantara/kbx/commit/5835888a0c1ab5263773976303bc028bcb969790))
* **domain:** Map a Skills record to a skills entry ([378a2e6](https://github.com/roalcantara/kbx/commit/378a2e6545b619cac4d8cdb14a5ee0e3f9e9c4a0))
* **domain:** Name the SKILL.md a warning is about ([95fbaf0](https://github.com/roalcantara/kbx/commit/95fbaf0e26ffdcccb9e0ae8ab386fe5a11006524))
* **domain:** Name the sources a sync can refresh ([d738580](https://github.com/roalcantara/kbx/commit/d738580593c85b60b15d7159967ef7fcdc723724))
* **domain:** Parse the Skills command's JSON records ([e53144b](https://github.com/roalcantara/kbx/commit/e53144b1c6581fffe62f1a72776d29049555b030))
* **domain:** Record each index row's source and migrate old indexes ([a54d215](https://github.com/roalcantara/kbx/commit/a54d215362e730e4483309ef826bd1676acd54e6))
* **domain:** Run the Skills command and kill it on cancel ([511ebea](https://github.com/roalcantara/kbx/commit/511ebea1dcaae50fabae7b9e77d89a5df6e5d526))
* **domain:** Sync the selected sources, each in its own transaction ([0248e92](https://github.com/roalcantara/kbx/commit/0248e92fa7c3efd7bae497a2bf8bdf9799250f44))
* **engine:** Count matches, narrow to four types ([3b82a0f](https://github.com/roalcantara/kbx/commit/3b82a0f954b186d00f05f68149b567bbad14be03))
* **theme:** Build GPUI Kit's theme from a palette in Rust ([0900b7a](https://github.com/roalcantara/kbx/commit/0900b7a7bfa19ca8bbde9c78d2ccd7bb5c4099f3))
* **theme:** Check a palette against kb's legibility rules ([a9a4a2b](https://github.com/roalcantara/kbx/commit/a9a4a2b59d89222c7b4430321023532ca873eada))
* **theme:** Draw with the active theme, kb-native by default ([d21c542](https://github.com/roalcantara/kbx/commit/d21c5423728a2805fe9b364558a1fd9832f78fca))
* **theme:** Map base24 and base16 schemes onto kb's colour roles ([d81fa0c](https://github.com/roalcantara/kbx/commit/d81fa0c5774f52b7842ae829178b4ef6556d9a08))
* **theme:** Port the design kit's colour arithmetic to Rust ([c611084](https://github.com/roalcantara/kbx/commit/c611084321aeecbd65f873599b38a4891ab39ca2))
* **theme:** Ship eight themes and seed them into the themes folder ([6e42489](https://github.com/roalcantara/kbx/commit/6e42489017ea730e8e60b1e558f6ca58a9bb5f86))
* **todos:** Add task to improve refresh results ([bfb46ae](https://github.com/roalcantara/kbx/commit/bfb46ae7fa8645b682ae2ba7cea6ab2b4f2eaa78))
* **ui:** Add the empty view ([3e3a607](https://github.com/roalcantara/kbx/commit/3e3a60785a1c4952a897844f3dea067ecd5c8bd7))
* **ui:** Draw a report row's pane ([713c6e5](https://github.com/roalcantara/kbx/commit/713c6e58ee16e0918a78f877505538508b2c5379))
* **ui:** Draw one cap per key and keep rows and footer still ([162f8e1](https://github.com/roalcantara/kbx/commit/162f8e144f4126084c0fe5cac96bf5c48e05c754))
* **ui:** Draw the action panel and the filter as the approved pages ([41355c3](https://github.com/roalcantara/kbx/commit/41355c300dd4b37a3435121f457e044d511d58ae))
* **ui:** Draw the refresh screens as the approved page ([d5c08b7](https://github.com/roalcantara/kbx/commit/d5c08b7538889ab1e7fd985e7ba073481ab6aa8b))
* **ui:** Drive Links and Filter keys and sections from catalog ([c13a09b](https://github.com/roalcantara/kbx/commit/c13a09b504be6a041fa7e685fe858fb971bd0bf5))
* **ui:** Map rebuild to Refreshing and omit Details ([5b42955](https://github.com/roalcantara/kbx/commit/5b42955e57a1bdc9b7ba385de4d267cf0cd3af3f))
* **ui:** Name where back goes with one crumb on every place ([fd7488d](https://github.com/roalcantara/kbx/commit/fd7488db57d9050381dd6ae8b493f3802cc9c86b))
* **ui:** Open Refresh More… from More… in the action panel ([cd15edc](https://github.com/roalcantara/kbx/commit/cd15edc813d71456c886e093cb91081478d89764))
* **ui:** Pick a theme live from More… → Theme… ([a54a65c](https://github.com/roalcantara/kbx/commit/a54a65c3b5f33b5a24b06e5a236edbd62bacd8eb))
* **ui:** Refresh a source alone and report each source apart ([76ff869](https://github.com/roalcantara/kbx/commit/76ff869414fcf317aea217e8a4b703dca55a1a7b))
* **ui:** Run matching chord row on ⌘↵ and ⇧⌘↵ in action panel ([91469c0](https://github.com/roalcantara/kbx/commit/91469c0ad6cd0e1e9391c03e2b133a044f25c3d5))
* **ui:** Scroll selected panel row into view with ScrollHandle ([3e4243d](https://github.com/roalcantara/kbx/commit/3e4243d702e26927dc514226f2992d49f0739a1d))
* **ui:** Show entries in the new template, split and detail ([d21f618](https://github.com/roalcantara/kbx/commit/d21f618c40eaf82b029e81b35244ced76bacec63))
* **ui:** Show the empty view for no match and no index ([72b7c6c](https://github.com/roalcantara/kbx/commit/72b7c6ca2e7a350f1a27382b1260c417d159bb70))
* **ui:** Take the failed report chip from the catalog ([37a15e2](https://github.com/roalcantara/kbx/commit/37a15e2e24d1ae18f66dcb3f486c10d93a36e872))

### Bug Fixes

* **app:** Act on keys only for the top place or the open overlay ([02d4a86](https://github.com/roalcantara/kbx/commit/02d4a869dafc358488f9ae5c5c9fa0addb8055a9))
* **app:** Change overlays in one place, with their own footer keys ([039ae7b](https://github.com/roalcantara/kbx/commit/039ae7b0ef6f3f8ee5b843f0d36a27152f63886a))
* **app:** Follow every selection change from one place ([9c56ac7](https://github.com/roalcantara/kbx/commit/9c56ac793b53ebbda80f32533baf44f45edea537))
* **app:** Keep report row actions on the report and to spec ([62fc4c4](https://github.com/roalcantara/kbx/commit/62fc4c48bd3b34f81777188f5fafacbad99476d8))
* **app:** Keep the navigator's invariants in one place ([7ab6c0c](https://github.com/roalcantara/kbx/commit/7ab6c0cbe45107d8e800835e715172050d7af3b6))
* **app:** Open Configure from an unreadable sources directory ([3721990](https://github.com/roalcantara/kbx/commit/3721990461942b6709119eb7ff30237e787d8139))
* **app:** Select through one function that keeps the row in view ([d03c1fb](https://github.com/roalcantara/kbx/commit/d03c1fb5867952677c375e1897c1d989df8a6f90))
* **app:** Take every notice's words from the catalog ([7c26d2e](https://github.com/roalcantara/kbx/commit/7c26d2e08959969ec50e7c1112bb2f144e9a52b7))
* **deps:** Update rustls to 0.23.46 for RUSTSEC-2026-0346/0347 ([088898b](https://github.com/roalcantara/kbx/commit/088898b49d6c36be1b86610d670007709733d3f6))
* **deps:** Update rustls-webpki to 0.103.16 for RUSTSEC-2026-0382 ([29d6905](https://github.com/roalcantara/kbx/commit/29d6905f215763aea04e7e46960823268c6bb651))
* **design:** Capture every screen from a checked reset ([262e7d2](https://github.com/roalcantara/kbx/commit/262e7d228a3cc2324d38d88e5f54a97d5f03744b))
* **design:** Keep kb's own colours; Neon Dusk becomes a theme ([aec1884](https://github.com/roalcantara/kbx/commit/aec1884e9f21b7e070975739db4c4c34216c6ae4))
* **design:** Keep the tokens the design tools read ([e5d5728](https://github.com/roalcantara/kbx/commit/e5d5728b48ff60027b6422e6a9568c825a409ac9))
* **design:** Set keycap glyphs at 10 pt ([6ad97c9](https://github.com/roalcantara/kbx/commit/6ad97c9ca7fbf7b02012957789f0b85a1b23b5a6))
* **design:** Shoot More… and park the pointer before each shot ([778c7dd](https://github.com/roalcantara/kbx/commit/778c7ddfb00f3204ac49780e07b530a6a7a80626))
* **hk:** Pass mise's GitHub token to pinact ([7c91fe5](https://github.com/roalcantara/kbx/commit/7c91fe532536b8ac4f14bd3827af23faef82e0bd))
* **hk:** Scan hk's files in one gitleaks run ([acb0c54](https://github.com/roalcantara/kbx/commit/acb0c54c72e0b07081edc16e142dc54eb6e68828))
* **hk:** Use 'mise exec' for consistency ([75169b4](https://github.com/roalcantara/kbx/commit/75169b415c16e903d69d768e9c25a8b91859c737))
* **keys:** Route navigation keys through one function ([a7ed906](https://github.com/roalcantara/kbx/commit/a7ed9066db0da1ddf9ccd42f3bc17646ea983151))
* **skills:** Read the Skills command's output from files, not pipes ([4c7a595](https://github.com/roalcantara/kbx/commit/4c7a59508cf29bc1cc24cfb3afcd37025f260c06))
* **ui:** Adjust keycap dimensions and add width normalization ([9895641](https://github.com/roalcantara/kbx/commit/9895641218d16d15cef65d5c1c3c185eb283cbb5))
* **ui:** Cap entry header tags and let row shrink ([9fe3fbc](https://github.com/roalcantara/kbx/commit/9fe3fbcf18300380d8d887a99f52ae5e589a5f24))
* **ui:** Close 8f review findings on copy, docs and a lost test ([9a8f185](https://github.com/roalcantara/kbx/commit/9a8f185ab93fc1ce94fd099ab06ee425d708b935))
* **ui:** Fall back to hqdefault when maxres thumbnail fails ([89d9cdd](https://github.com/roalcantara/kbx/commit/89d9cdd601e8ce1646ec507910bef844975eaa1d))
* **ui:** Give the action panel and filter their keyboard and keys ([76fa25c](https://github.com/roalcantara/kbx/commit/76fa25c9dc9f1a8e5c996b768b99238cc0e5e1ea))
* **ui:** Ignore empty-screen key ladder while search is pending ([140c571](https://github.com/roalcantara/kbx/commit/140c571eb468fcd6249e1001cd269e1eaba8fb4b))
* **ui:** Keep a toggled filter row selected; follow 8e in the guides ([7b8fcc3](https://github.com/roalcantara/kbx/commit/7b8fcc3759ed4d17172cbd676a428d46ee9c8937))
* **ui:** Keep each row`s description on one line ([1757954](https://github.com/roalcantara/kbx/commit/1757954bf3f08135b9332894cad5076a61ca798d))
* **ui:** Land the pill's and the footer's keys on the content edge ([6708003](https://github.com/roalcantara/kbx/commit/6708003e64f7f140a5c45d4e1ac16c8b56f7fff6))
* **ui:** Say No entries for an empty file in the report ([cb923dc](https://github.com/roalcantara/kbx/commit/cb923dc67e57a077784aeb1460c4edad705136e9))
* **ui:** Say No entries with this filter for an empty query ([19cd355](https://github.com/roalcantara/kbx/commit/19cd3553ac194d4e0e43d369a090b604fe8ff46a))
* **ui:** Scroll the panel to its selected row's own child ([9d0ad14](https://github.com/roalcantara/kbx/commit/9d0ad1481cfd22f4fadb73436e43148bb47fb426))
* **ui:** Use tabular digits for counts ([fc9bc2a](https://github.com/roalcantara/kbx/commit/fc9bc2afd4e2361c70f6e1f1355482991c73b07f))

# Changelog

Every release of kb, written by semantic-release from the commit messages (docs/adr/0007-releases.md).

## [0.5.0](https://github.com/roalcantara/kbx/compare/v0.4.0...v0.5.0) (2026-10-12)

### Features

* **app:** Scroll the detail screen's notes from the keyboard ([639c3ee](https://github.com/roalcantara/kbx/commit/639c3ee08472c95a7d6d566aea9b679804952bc6))
* **app:** Tint tag images inside their pill ([bad8591](https://github.com/roalcantara/kbx/commit/bad8591bd1e996c532b2444ff62b132f13f65f79))
* **domain:** Read file:// notes from their files ([62d7b69](https://github.com/roalcantara/kbx/commit/62d7b69ba673102ce4f45cc3f7457b21300fdef9))
* **tools:** Add the tag tools: suggest, apply and check ([7a3f30f](https://github.com/roalcantara/kbx/commit/7a3f30fcabb9634a66d82739f341ed275ac5167f))
* **tools:** Shorten, cluster, split and drop tags ([6b978f5](https://github.com/roalcantara/kbx/commit/6b978f5766712c28d238e6d7d98f58c0b10c068f))

### Bug Fixes

* **app:** Calm the footer, rows, popovers and tag images ([dbf8cf0](https://github.com/roalcantara/kbx/commit/dbf8cf05398a52f9147b3eb608a19e239afd558a))
* **app:** Draw the icons the default bundle left out ([b3cae0f](https://github.com/roalcantara/kbx/commit/b3cae0f391efab2e6ff3a2396aa789c72d461a47))
* **app:** Focus each popup's search field when it opens ([e2e3e96](https://github.com/roalcantara/kbx/commit/e2e3e96410a6dd9c6a5aa2e76c44eca6f9fd9e38))
* **app:** Give code in the notes its own contrast ([5b013da](https://github.com/roalcantara/kbx/commit/5b013dae9bf05bf1bcf5382b2b6167223acdaac2))
* **app:** Match the list, footer and popovers to the prototype ([bf9ce8a](https://github.com/roalcantara/kbx/commit/bf9ce8adc07d33345a16ba3f38826d4b6593043a))
* **app:** Open the filter and actions on a click ([0f4d7cd](https://github.com/roalcantara/kbx/commit/0f4d7cd2eea4f838f144c14e54eb0907097bfd80))
* **tools:** Keep tag guesses from feeding each other ([e7c561d](https://github.com/roalcantara/kbx/commit/e7c561d0ca4540e50709f27aac9bb862063f2198))
* **tools:** Merge a renamed singular into its plural ([d38d664](https://github.com/roalcantara/kbx/commit/d38d66419a724d84cbddfb9d35835c5d8fea7ab2))

## [0.4.0](https://github.com/roalcantara/kbx/compare/v0.3.1...v0.4.0) (2026-10-11)

### Features

* **app:** Anchor the actions to the footer, with keycaps and pages ([e7b65b3](https://github.com/roalcantara/kbx/commit/e7b65b3ba1f970242b05dc023c28a2cb86801902))
* **app:** Draw rows as Raycast's, with favicons and tag images ([6167d0b](https://github.com/roalcantara/kbx/commit/6167d0ba6327722005f0be22d9261027fcb81fe5))
* **app:** Draw the header as Raycast's ([ce8207e](https://github.com/roalcantara/kbx/commit/ce8207e3ab4fa3ab953792168651d7e5980075fd))
* **app:** Float the footer as Raycast's pills ([9ce42e7](https://github.com/roalcantara/kbx/commit/9ce42e71ba87e49f033bc76dc7b0333ed6f148b5))
* **app:** Keep the filter open under its button while picking ([639c023](https://github.com/roalcantara/kbx/commit/639c023ac54f9856ed6e26f6b9c2efe8c29ec2f4))
* **app:** Show the metadata card and Raycast's split ([f737e7e](https://github.com/roalcantara/kbx/commit/f737e7e76c410d4b540e02dc6fb322947e5c83c5))
* **app:** Tint types and tags with DESIGN.md's seven accents ([fdf3b3c](https://github.com/roalcantara/kbx/commit/fdf3b3c265f39a8e7b239ddfabad54bb1a770541))
* **app:** Toggle the split pane with → and ← as Raycast ([37ab079](https://github.com/roalcantara/kbx/commit/37ab0797724ef2777cc70994746de4a283ecde4f))
* **app:** Toggle the window between compact and expanded with ⌘⇧F ([809a72c](https://github.com/roalcantara/kbx/commit/809a72c16c39ff765c3fae72af0e3eb9637b6455))
* **app:** Use Raycast's Title Case labels ([858cd98](https://github.com/roalcantara/kbx/commit/858cd984f630b02561eeec9a29d9c315a912204d))

### Bug Fixes

* **app:** Load remote images through a curl HTTP client ([f399dc5](https://github.com/roalcantara/kbx/commit/f399dc58aa23cf1e89f7b84c1eb4e14db9ba82a5))

## [0.3.1](https://github.com/roalcantara/kbx/compare/v0.2.1...v0.3.1) (2026-10-10)

Prepared by semantic-release as 0.3.0, published by hand as 0.3.1: GitHub reserves the tag `v0.3.0` (an immutable
release once used it), so 0.3.0 was never published.

### Features

* **app:** Apply DESIGN.md's palette and base typography ([68de3a8](https://github.com/roalcantara/kbx/commit/68de3a8a5b49f848f6590a4a6d08ced935dc9e56))
* **design:** Add shared Raycast layout foundation ([940b26c](https://github.com/roalcantara/kbx/commit/940b26c8b32630acbb34848799d3158cfe40c53e))
* **tools:** Add candidate palettes for kb ([beb7e78](https://github.com/roalcantara/kbx/commit/beb7e78cceb48b75760a587bc3865b24bf853ae6))
* **tools:** Measure Raycast's palette and sizes ([85d15a6](https://github.com/roalcantara/kbx/commit/85d15a6d4bb8c2b1f412773e1c43226d54810337))
* **tools:** Render the palette comparison page ([00099d0](https://github.com/roalcantara/kbx/commit/00099d025b61edee8b708e5a8c29f58b33eebca4))
* **tools:** Validate the candidate palettes ([a790a07](https://github.com/roalcantara/kbx/commit/a790a071b7763dbbe760a46df7fb973eae35d78d))

### Bug Fixes

* **tools:** Share colour rules and render every palette ([f38f547](https://github.com/roalcantara/kbx/commit/f38f5475f861d3a4871597c8acc99b0dfc223ff5))

## [0.2.1](https://github.com/roalcantara/kbx/compare/v0.1.0...v0.2.1) (2026-10-10)

Prepared by semantic-release as 0.2.0, published by hand as 0.2.1: GitHub reserves the tag `v0.2.0` (an immutable
release once used it), so 0.2.0 was never published.

### Features

* **tools:** Add the graph task for agents' code graphs ([db1623d](https://github.com/roalcantara/kbx/commit/db1623d32049209c551b8b0faf47302723a93181))
* **tools:** Refresh the code graphs after commits and checkouts ([29dd2da](https://github.com/roalcantara/kbx/commit/29dd2dab08b30bd5be68e65e951f9b05621cbaa5))

### Bug Fixes

* **tools:** Harden the graph task and its background refresh ([afe84b1](https://github.com/roalcantara/kbx/commit/afe84b16b9bd00c28adbbd5b11416c095f3c4b2b))
* **tools:** Report graph failures and close a refresh race ([0f24fc2](https://github.com/roalcantara/kbx/commit/0f24fc2209167ec7b2ac216586533a054074c399))
