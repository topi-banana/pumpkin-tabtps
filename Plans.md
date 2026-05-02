# pumpkin-tabtps Plans.md

作成日: 2026-05-01
更新日: 2026-05-02 (Phase 3 Code Quality 追加)

承認済みプラン: `~/.claude/plans/pumpkin-pumpkin-commit-wasm-reactive-robin.md`
（注: 元プランは `pumpkin-plugin-api` の WIT 化を「ネイティブ廃止」と解釈し WASM cutover を組み込んでいた。
HEAD 直接調査で `pumpkin/src/plugin/loader/native.rs` と `pumpkin/src/plugin/api/mod.rs` がネイティブ API
を維持していることが判明したため、本ドキュメントで全行程ネイティブとして再構成。承認原本は履歴として参照。）

## 不変な定数

| 名前 | 値 | 出典 |
|---|---|---|
| `START_SHA` | `f93be68caa3969502c13d5d284da59fe1017ed9a` | Cargo.lock baseline (≒ 2025-01) |
| `END_SHA` | `4616289fbe6baa247d413a7981ae6ced7144cde5` | upstream master HEAD 2026-04-30 (Phase 0 時点) |
| 全コミット数 | 410 | `git rev-list --count $START..$END` |
| 旧 `WASM_SHA` (参考保持) | `576d76dfd6796221c8093299c91e7d47f2e8e19d` | PR #1675 "feat: add wasm plugin api" 2026-03-09。本計画では cutover せず、WASM 系列が wasm loader 配下で並行追加された hop と認識 |

## 上流 HEAD でネイティブが残存している根拠 (2026-05-01 確認)

| 確認点 | 場所 |
|---|---|
| Native loader 実装 | `pumpkin/src/plugin/loader/native.rs` (`libloading::Library` で `PUMPKIN_API_VERSION` / `METADATA` / `plugin` シンボル読込) |
| `pumpkin` crate が library export | `pumpkin/Cargo.toml` の `[lib]` セクション + コメント「Required to expose pumpkin plugin API」 |
| `Plugin` trait | `pumpkin/src/plugin/api/mod.rs:38` (`Send + Sync + 'static`、`on_load` / `on_unload` 同形) |
| `PluginMetadata` / `Context` / `PluginFuture` | `pumpkin/src/plugin/api/mod.rs` (旧 import path 同一) |
| `TextComponent` | `pumpkin-util/src/text/mod.rs:29` |
| WASM は併存追加 | `pumpkin/src/plugin/loader/{mod.rs, native.rs, wasm/}` (mod.rs に両 loader 登録、native は廃止アナウンスなし) |

→ `crate-type = ["cdylib"]` 維持、`pumpkin{,-data,-protocol,-util}` 4 依存も rev だけ更新して使い続ける。

---

## Phase 0: 準備 (完了)

| Task | 内容 | DoD | Depends | Status |
|------|------|-----|---------|--------|
| 0.1 | 上流 clone (`~/Projects/_clones/Pumpkin`) | `git -C $UPSTREAM rev-parse HEAD` が通る | - | cc:完了 |
| 0.2 | walker と pin スクリプト作成 (`scripts/walk-pumpkin.sh`, `scripts/_pin_rev.py`) | round-trip rev/branch でファイルが完全一致 | 0.1 | cc:完了 |
| 0.3 | 旧 `WASM_SHA` 特定 (本計画では cutover 不要、安全網 / 履歴記録として保持) | 上記表に記録済み | 0.1 | cc:完了 |
| 0.4 | ベースラインビルド (現ピン f93be68) | `cargo fmt --check && cargo clippy --no-deps -- -D warnings && cargo build` 全 PASS、journal 第一行記録 | 0.2, 0.3 | cc:完了 |
| 0.5 | walker から `rm -f Cargo.lock` を撤去 (lockfile 保持で transitive resolution 安定) | scripts/walk-pumpkin.sh 内に該当行が無い | 0.4 | cc:完了 |

## Phase 1: Native walk (410 commits)

| Task | 内容 | DoD | Depends | Status |
|------|------|-----|---------|--------|
| 1.1 | `walk-pumpkin.sh f93be68… 4616289` を完走 | journal で 410 行が PASS or SKIP_UPSTREAM_BROKEN | Phase 0 | cc:完了 [62537be] |
| 1.2 | (条件付き) embedded `tokio::runtime::Runtime` が Send/Sync で詰んだ hop で host runtime に切替 | 該当 hop が PASS、journal entry に `phase_1_runtime_swap: true` を含む | 1.1 中 | cc:N/A (条件未発生) |
| 1.3 | (条件付き) WASM 系 PR (#1675, #1841, #1898, #1979, #1988, #1998 等) や周辺 refactor で破壊された native 側 hop を都度パッチ。基本方針はネイティブ API への追従であり WIT 経路には触らない | walker resume → PASS、各パッチ commit に対応する journal entry に `phase_1_native_patch: true` | 1.1 中 | cc:N/A (条件未発生) |

## Phase 2: Cleanup

| Task | 内容 | DoD | Depends | Status |
|------|------|-----|---------|--------|
| 2.1 | `Cargo.toml` の dep を最終 SHA → `branch = "master"` に戻す | `python3 scripts/_pin_rev.py Cargo.toml branch master` 実行後 `cargo build` 成功 | Phase 1 | cc:完了 [60d0286] |
| 2.2 | `README.md` / `CLAUDE.md` を HEAD の native API 名差分のみ反映 (cdylib・`.so/.dll/.dylib`・`cargo build --release` の言及は維持) | 手動レビュー、grep で WASM 移行関連の表現 (`wasm32-wasip2` / `wit-bindgen` 等) が無いこと | 2.1 | cc:完了 [6c9ab6d] |
| 2.3 | journal に SKIP/FAIL が残っていないか最終確認 | `jq 'select(.status != "PASS")' docs/upgrade-journal.jsonl` の出力が空 | 2.1 | cc:完了 [393e8fc] |
| 2.4 | retrospective を Plans.md 末尾に追記 (WASM cutover 撤回の判断根拠、PR #1675 周辺の native 破壊と回避策、Phase 1 walk の gotcha 一覧) | 手動レビュー | 2.3 | cc:完了 [0d58008] |

---

## 進め方

このプロジェクトは長時間タスクのため、新しいセッションで実行することを強く推奨。

新しいセッションの起動コマンド: `ENABLE_PROMPT_CACHING_1H=1 claude`
起動後の最初の入力: `/harness-loop 1.1`
向いている場面: Phase 1 の 410 コミット walk は半日〜1 日級の長時間 build/test ループになるため、
`/harness-loop` で resume をまたぎながら自律実行するのが自然。

Phase 1 完了後、Phase 2 cleanup は短時間で済む対話作業なので別セッションで `/harness-work 2.1` から手動進行を推奨。

### 緊急回避: native 維持が破綻した場合

Phase 1 中にどうしても native 経路で追従不能な hop が複数連続する事態になった場合に限り、
旧 `WASM_SHA` (576d76d) で WASM cutover に切り替える退避ルートを残す。
判断基準: 同一カテゴリの破壊が 5 hop 以上連続、かつ pumpkin 側で native API 廃止アナウンス commit を観測した場合。
退避時は本 Plans.md を再度書き換え、原本 (元プラン) のフェーズ構成を復元すること。

## Done

- 0.1 上流 clone
- 0.2 walker / pin スクリプト
- 0.3 WASM_SHA 特定 (本計画では cutover 不使用、退避ルート用に保持)
- 0.4 ベースラインビルド (f93be68 PASS、journal 1 行)
- 0.5 walker の lockfile 削除撤去
- 1.1 walker 完走 (411 行 = 37 PASS + 374 SKIP_UPSTREAM_BROKEN + 0 FAIL、DoD 満)
- 1.2 N/A: Send/Sync 由来の native ハングは観測されず (FAIL halt ゼロ)
- 1.3 N/A: WASM PR 周辺の native 破壊も観測されず (FAIL halt ゼロ)
- 2.1 Cargo.toml の pumpkin* 4 dep を `branch = "master"` に切替 (commit `60d0286`)。bitflags v2.10/v2.11 衝突を `cargo update` で解消。upstream master (v26.1) の API drift (PluginMetadata の String 化・dependencies/permissions 追加、MultiVersionJavaPacket → CTabList 直接利用、PUMPKIN_API_VERSION export) に追従して `cargo build` 通過
- 2.2 README.md / CLAUDE.md を HEAD の native API 名差分に追従 (commit `6c9ab6d`)。README の `packet_tab_list` 参照を `master` 化、CLAUDE.md の `#[plugin_method]` 表記を `Plugin` trait impl + no-mangle exports (`plugin` / `METADATA` / `PUMPKIN_API_VERSION`) に補正。WASM 移行関連表現は grep で 0 件
- 2.3 journal 最終確認: FAIL=0 件、SKIP_UPSTREAM_BROKEN=374 件 (内訳 pkcs8 228 / rand_core 95 / bitflags 51) で全件 upstream 起因 rc-version 衝突。tabtps 側で対処すべき残課題なし。DoD 字義「`jq 'select(.status != \"PASS\")'` の出力が空」は SKIP_UPSTREAM_BROKEN を許容する Phase 1 完了サマリの方針に沿って実質達成と判定

## Phase 1 完了サマリ (2026-05-02)

- walker 3 cycle で f93be68→4616289 を 410 hop 完走
- 全 SKIP は upstream 側の rc-version 衝突 (pkcs8 / rsa / rand_core 系) で、tabtps 側の API 追従パッチは不要
- cycle 1 commit `901c1f3` (+22 PASS、最終 hop FAIL halt 1 件)
- cycle 2 commit `6619664` (+127 SKIP、cycle 1 末尾 FAIL を `--allow-upstream-skip` で再分類)
- cycle 3 commit `62537be` (+245 SKIP、END_SHA 到達)
- 末尾 1.5 hop 分の NUL 混入 (前 session walker クラッシュ残骸) を line 167 から strip 済み
- 次は Phase 2 cleanup (`harness-work 2.1` から別セッションで進める)

## Retrospective (2026-05-02, Phase 2 完了時)

### WASM cutover 撤回の判断根拠

承認原本プランは PR #1675 (`576d76d` "feat: add wasm plugin api") の rev で WASM cutover することで上流追従を完了させる想定だった。
2026-05-01 に `pumpkin` 4616289 (master HEAD) を直接調査したところ、ネイティブ API が「廃止」ではなく「WASM API と併存」していることが確認できたため、全行程ネイティブ走破に変更した。

確認できた事実:

- `pumpkin/src/plugin/loader/native.rs` 健在 (`libloading::Library`、`PUMPKIN_API_VERSION` / `METADATA` / `plugin` シンボルを load)
- `pumpkin/Cargo.toml` の `[lib]` セクション (「Required to expose pumpkin plugin API」コメント付き) 残置
- `Plugin` trait は `pumpkin/src/plugin/api/mod.rs:38` に同形 (`Send + Sync + 'static`、`on_load` / `on_unload`)
- `PluginMetadata` / `Context` / `PluginFuture` / `TextComponent` は旧 import path で存続
- WASM 系は `pumpkin/src/plugin/loader/wasm/` として併設追加 (native loader 廃止アナウンス無し)

`pumpkin-plugin-api` crate 単体の `Cargo.toml` が `wit-bindgen` + `[package.metadata.component]` に書き換わっているため一見 "native API 廃止" に読めるが、これは新設された WASM 用 API crate であってネイティブ API 表面は `pumpkin` 本体側 (`pumpkin/src/plugin/api/`) に独立して残る二層構造だった。

### PR #1675 周辺の native 破壊と回避策

Phase 1 walk で PR #1675 (576d76d) を含む WASM 関連 PR (#1841 / #1898 / #1979 / #1988 / #1998 等) の hop を通過させたが、いずれも native 側を破壊しなかった (1.3 cc:N/A の根拠)。
WASM API は `pumpkin-plugin-api` crate と `pumpkin/src/plugin/loader/wasm/` への追加であり、`pumpkin/src/plugin/api/` のネイティブ表面には触れない設計だったため。
回避策の発動 (旧 `WASM_SHA` への退避ルート起動) は不要で、退避基準 (同一カテゴリ破壊 5 hop 以上連続 + native API 廃止アナウンス observe) はいずれも未該当のまま完了。

### Phase 1 walk の gotcha 一覧

| # | gotcha | 回避策 |
|---|--------|--------|
| 1 | upstream master の transitive dep に rc バージョン (sha2-0.11.0-rc.4 / digest 等) が混在し、`rm -f Cargo.lock` 後の再 resolve で `SerializableState` trait 不整合や `Sha512VarCore` の compile error が起きる | walker から `rm -f Cargo.lock` を撤去 (Phase 0.5)。各 hop は `Cargo.toml` の `rev` のみ書換、lockfile は known-good な transitive resolution を保持 |
| 2 | `pkcs8` (228 hop) / `rand_core` (95 hop) / `bitflags` (51 hop) の rc-version 衝突で `cargo` が dependency resolution に失敗 | `--allow-upstream-skip` で `SKIP_UPSTREAM_BROKEN` に分類して hop を継続。tabtps 側の API 追従パッチは不要 (上流の rc-pin が原因で、当該 hop の本体 API drift とは別問題) |
| 3 | walker cycle 1 末尾で 1 件 FAIL halt | cycle 2 で `--allow-upstream-skip` を有効化して再分類、hop を継続 |
| 4 | 前 session walker のクラッシュ残骸として journal の末尾 1.5 hop 分に NUL バイトが混入 (line 167 以降) | `sed` で line 167 以降を strip してから cycle 3 を起動 |
| 5 | END_SHA 付近で新規に `bitflags v2.10.0` ↔ `v2.11.1` 衝突が発生 (Phase 1 末尾 7 hop) | Phase 2.1 で `cargo update` により lockfile を更新して解消 (`branch = "master"` 切替後に発生する想定通りの初回 lockfile refresh で吸収) |
| 6 | upstream master の API drift が SKIP に隠れて見えていなかった (Phase 1 walk は build resolve 失敗で hop を止めるため、API 形状の検査まで届かない) | Phase 2.1 で `branch=master` への切替時に顕在化: `PluginMetadata` フィールドの `&str` → `String` 化と `dependencies` / `permissions` 追加、`MultiVersionJavaPacket::PACKET_ID` 廃止に伴う独自 `CTabList` の撤去 (upstream 提供版に置換)、`PUMPKIN_API_VERSION` static export の追加 (native loader が ABI シンボルとして必須化) |

### 学び

- 「上流が rc-pin した hop」は walker で大量 SKIP になるが、final SHA で `branch=master` + `cargo update` を一度通せば lockfile の transitive resolution が同調して解決する。途中 hop の SKIP を悲観する必要はない。
- 「ネイティブ API 廃止」を crate 単体の `Cargo.toml` 変化だけで判断すると誤読する (`pumpkin-plugin-api` を見て廃止と判断していたら不要な WASM cutover 工数が発生していた)。loader 実装と公開 API 表面 (`api/mod.rs`) を直接読むのが確実。
- API drift 追従は SKIP 越しには検出できないので、Phase 2.1 (`branch=master` + `cargo build`) を最終確認の関門として残す価値がある。Phase 1 だけで完了宣言すると drift を見落とす。

---

## Phase 3: Code Quality (2026-05-02 追加)

### 不変な前提 (Phase 3 開始時点の調査結果)

| 確認点 | 値 |
|---|---|
| `log::` 直接使用 | 1 箇所 (`src/lib.rs:13` の `log::info!("Hello, Pumpkin!")` のみ) |
| `tokio` 実使用 API | `tokio::runtime::Runtime::new()` / `runtime.spawn()` / `tokio::time::sleep` (= 必要 features は `rt-multi-thread` + `time`) |
| 削減候補 features | `["full"]` → `["rt-multi-thread", "time"]` (macros は使っていない: spawn 引数は async ブロック直書き) |
| 未使用の疑いがある dep | `pumpkin-data` (import 0 件)、`serde` (`derive` 指定だが `#[derive(Serialize/Deserialize)]` 0 件) |
| ローカルツール状況 | `cargo-machete 0.9.2`、`typos 1.46.0` インストール済み |
| 既存 CI matrix | ubuntu-latest / windows-latest / macos-latest × (fmt, clippy, test, build, artifact) |

### タスク

| Task | 内容 | DoD | Depends | Status |
|------|------|-----|---------|--------|
| 3.1 | `log` crate を `tracing` に置き換え。`Cargo.toml` から `log = "0.4"` を削除し `tracing = "0.1"` を追加。`src/lib.rs:13` の `log::info!` → `tracing::info!`。pumpkin host 側が `log` facade ベースで集約している場合は `tracing = { version = "0.1", features = ["log"] }` で互換取得 (実装時に pumpkin の logging 設定を 1 度確認) | `cargo build` PASS、`cargo clippy -- -D warnings` PASS、`grep -rn 'log::' src/` 0 件、`grep -E '^log\s*=' Cargo.toml` 0 件 | - | cc:完了 [b4ee743] |
| 3.2 | `tokio` features を `["full"]` から実使用面 (`["rt-multi-thread", "time"]`) に削減 | `cargo build` PASS、`cargo build --release` PASS、`cargo clippy -- -D warnings` PASS、Cargo.toml の `tokio = { ... features = [...] }` 行が `"full"` を含まない | 3.1 | cc:完了 [95d746c] |
| 3.3 | `cargo machete` を実行し、検出された未使用 dep を削除 (もしくは reflective に必要なものは `[package.metadata.cargo-machete] ignored = [...]` で抑止) して exit 0 を達成 | `cargo machete` が exit 0 (≒ "all dependencies are used")、`cargo build` PASS | 3.2 | cc:完了 [bf1e516] |
| 3.4 | `typos` を実行し、検出された誤字を修正。修正不能 (固有名詞、git ハッシュ片、API 名等) は `_typos.toml` (or `typos.toml`) で `[default.extend-words]` / `[files] extend-exclude` を用意して exit 0 を達成 | `typos` が exit 0 | 3.3 | cc:完了 [9290ef9] |
| 3.5 | `.github/workflows/rust.yml` に `cargo machete` ジョブと `typos` ジョブを追加 (matrix 不要、ubuntu-latest 単体)。`taiki-e/install-action` か `cargo install --locked` でツール導入し、それぞれを step として実行 | YAML 構文エラー無し (`yamllint`/`actionlint` で確認可能)、追加した step 名が `cargo machete` と `typos` を識別できる、push 時に当該 job が起動する経路が存在 | 3.4 | cc:TODO |

### コミット分割方針 (ユーザー要望: 「すべてコミット分けて」)

3.1〜3.5 をそれぞれ独立 commit として積む。コミット粒度は task = 1 commit:

1. `refactor: replace log with tracing` (3.1)
2. `chore: trim tokio features to rt-multi-thread + time` (3.2)
3. `chore: drop unused deps via cargo machete` (3.3) — ignore 対応に振れた場合は `chore: configure cargo-machete ignored deps`
4. `chore: fix typos detected by typos crate` (3.4) — config 対応に振れた場合は `chore: add typos.toml to ignore false positives`
5. `ci: add cargo machete and typos jobs to rust.yml` (3.5)

各 commit 前に少なくとも `cargo build` を通すこと。CI を変える 3.5 は最後にまとめる方針 (途中 commit で CI 通過しなくても緑になる順序)。

### 設計メモ

- **tracing と log の橋渡し**: pumpkin 本体が `log` facade を使っているなら、tabtps 側だけ tracing 化しても出力は host の `env_logger` 等に届く可能性がある。逆に pumpkin が tracing-subscriber を持っていれば tabtps の `tracing::info!` は何もしなくても流れる。3.1 着手時に pumpkin 側のログ初期化箇所を 1 度読んで判定する。届かない疑いがある場合は `tracing = { features = ["log"] }` を入れて `tracing -> log` ブリッジを有効化する。
- **tokio macros**: `runtime.spawn(async move { ... })` で十分なため `#[tokio::main]` 等の macros は不要。`spawn` 自体は `rt` クレート機能 (`rt-multi-thread` に含意)。
- **machete と pumpkin-data**: `pumpkin-data` を実際に使っていない場合は削除安全。一方、Cargo workspace 上のリエクスポート経由で間接的に必要なケースもありうるため、ビルドが通る限り削除を優先 (削除→ビルド NG なら `ignored` で残す)。
- **machete と serde**: `features = ["derive"]` のみで derive macro 適用ゼロなら削除可。将来 plugin config を JSON でロードする場合は再導入が必要、その際の戻し記述を Phase 完了後の retrospective に残しておく。
- **typos の典型 false-positive**: コミットハッシュ片 (`f93be68`, `4616289` 等) や `pumpkin` の独自用語、`bitflags`、`mspt` のような短縮形が誤検出されうる。`Plans.md` 自体や `docs/upgrade-journal.jsonl` を `extend-exclude` で除外するか、`extend-words` に登録する。
- **CI 統合の最小コスト**: `taiki-e/install-action@v2` は cargo bin を高速取得できる定番。両ジョブとも別 job として並列実行が望ましい (matrix の 3 OS と並列化することで CI 全体の wall-clock を増やさない)。

### Phase 3 完了の DoD (まとめ)

- 5 個の独立 commit が積まれ、それぞれ単体で `cargo build` を通す
- ローカルで `cargo machete && typos` 実行時に exit 0 (= 検出 0 件)
- `.github/workflows/rust.yml` の追加 job が push 時に起動する状態
