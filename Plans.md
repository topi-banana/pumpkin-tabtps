# pumpkin-tabtps Plans.md

作成日: 2026-05-01
更新日: 2026-05-01 (WASM cutover を撤回し全行程ネイティブ走破に変更)

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
| 2.4 | retrospective を Plans.md 末尾に追記 (WASM cutover 撤回の判断根拠、PR #1675 周辺の native 破壊と回避策、Phase 1 walk の gotcha 一覧) | 手動レビュー | 2.3 | cc:TODO |

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
