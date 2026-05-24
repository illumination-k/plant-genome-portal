# AGENTS Guideline

This repository is pre-alpha and under active development. The API is not stable and may change without a major version bump, so backwards compatibility is not guaranteed at this stage.
So developers of this repository DO NOT need to worry about breaking changes or maintaining backwards compatibility. We prefer to iterate quickly and make breaking changes as needed, rather than trying to maintain backwards compatibility.

## Policy

Follow the YANGI, SOLID, DRY, and KISS principles in all code and documentation. Prioritize simplicity, readability, and maintainability over cleverness or optimization. Avoid premature optimization and over-engineering. Strive for clear and concise code that is easy to understand and modify.

## Development Process

Run `mise install` first to install the toolchain and project tools.

At the end of a session, run `mise run ci` and make sure it passes. Use the narrower tasks while iterating:

```bash
mise run fmt      # Format
mise run lint     # Lint and policy checks
mise run test     # Tests
mise run ci       # Full required verification
```

## Commands

Run `mise install` first to install all tools.

```bash
mise run ci    # Run all ci:* tasks
mise run fmt   # Run all fmt:* tasks
mise run lint  # Run all lint:* tasks
mise run test  # Run all test:* tasks
```

## Tools

All tools are managed by mise. Run `mise install` to install them.

| Tool           | Purpose                                 |
| -------------- | --------------------------------------- |
| uv             | Python package manager                  |
| dprint         | Code formatter                          |
| prek           | Pre-commit hook runner                  |
| shfmt          | Shell script formatter                  |
| actionlint     | GitHub Actions linter                   |
| zizmor         | GitHub Actions security linter          |
| shellcheck     | Shell script linter                     |
| ghalint        | GitHub Actions linter                   |
| pinact         | Pin GitHub Actions versions to SHAs     |
| rust           | Rust toolchain                          |
| cargo-binstall |                                         |
| cargo-nextest  | Fast Rust test runner                   |
| cargo-deny     | Dependency license and advisory checker |
| cargo-audit    | Security advisory checker for Rust      |
| cargo-mutants  |                                         |
| cargo-llvm-cov |                                         |
| node           | Node.js runtime                         |
| pnpm           | Node.js package manager                 |

## Purpose

植物のゲノム + マルチオミクスデータを統合する Web データベース / ポータル。
Rust backend + React SPA frontend の monorepo。

参考にする既存DB:
- **NCBI Datasets v2** — API デザイン(accession-first, TaxID-driven, 階層モデル)を踏襲
- **MarpolBase / TAIR / Phytozome** — 植物コミュニティDBの先例
- **ATTED-II** — 共発現解析の参照モデル(MR / HRR / LS)

最終的に統合したいデータ:
- Genome (FASTA + GFF アノテーション)
- Transcriptome / Expression
- Co-expression network (MR / HRR / LS)
- Variant / Resequencing
- Epigenome (ChIP-seq, ATAC-seq, methylation)

## MVP scope

**MVP は genome レイヤーのみ。** 他のオミクスは v2 以降。
初期データは **Marchantia polymorpha** (TaxID: 3197) 1種。
MVP 段階では annotation = assembly に固定 (annotation の別バージョン管理は v2 以降)。

### MVP に入れる
- 種 / アセンブリ一覧・詳細
- 遺伝子検索 (symbol / locus_tag)
- 遺伝子詳細 (座標 + 配列 + annotation)
- 領域内 feature 取得 (`Chr1:1000-2000` → 遺伝子リスト)
- refget 準拠の参照配列取得
- JBrowse 2 によるゲノムブラウザ表示

### MVP に入れない (v2 以降)
- 発現 / 共発現 / variant / epigenome
- pangenome / liftover
- annotation の別バージョン管理
- 認証 (MVP は read-only public)

## Architecture

### Stack 概要

| 層 | 採用 |
|---|---|
| Backend lang | Rust (axum + sqlx + tokio) |
| Bio I/O | [noodles](https://github.com/zaeleus/noodles) (FASTA/GFF/VCF/BAM) + bigtools (BigWig) |
| Metadata DB | PostgreSQL |
| 発現マトリクス (将来) | Parquet on disk + DuckDB クエリ (hybrid) |
| 共発現 (将来) | PostgreSQL (top-N pre-computed edges) |
| Sequence / signal files | ファイルストア (FS / S3 / MinIO) + index (`.fai` / `.tbi` / `.csi` / `.bbi`) |
| Frontend | React + Vite + TypeScript |
| Genome browser | JBrowse 2 (@jbrowse/react-linear-genome-view) |
| State/query | TanStack Query + TanStack Router |

### データ階層 (3 層)

オミクスデータは「重さ」と「クエリ形態」が全く違うので 3 層に分離:

1. **Metadata** (PostgreSQL) — 種・アセンブリ・サンプル・実験条件
2. **Annotation** (PostgreSQL / 将来 Parquet) — 遺伝子・ピーク・バリアント・発現マトリクス(集約済み)
3. **Signal/Sequence** (ファイルストア + index) — FASTA/BigWig/BAM/VCF 等の生データ

**原則**: 生 BigWig / FASTA / VCF は DB に入れない。`.fai`/`.tbi`/`.bbi` 経由で領域クエリ。

### API デザイン

- **シーケンス取得は GA4GH refget 準拠**
  (checksum ベース、不変な配列参照、外部ツール互換)
- **バリアント/アラインメントは将来 htsget 準拠を検討**
  (S3 への byte-range Range リクエストでサーバ無負荷配信)
- **その他オミクス (発現/共発現/ChIP/ATAC/methylation) は独自 REST**
  (GA4GH に該当仕様なし)
- URL は NCBI Datasets v2 スタイル:
  ```
  /v2/genome/accession/{accession}
  /v2/genome/taxon/{tax_id}
  /v2/gene/id/{gene_id}
  /v2/gene/search?symbol=...&tax_id=...
  /v2/genome/accession/{acc}/region/{chr}:{start}-{end}/features
  ```

### 座標系の規約

- **API 境界は 1-based closed** (GFF/VCF 慣習、NCBI 互換)
- **DB 内部は 0-based half-open** で統一 (範囲 index の自然な扱い)
- `genome-core` で `Position0` / `Position1` / `HalfOpenRegion` / `ClosedRegion` を型レベルで区別、暗黙変換禁止

### スキーマ要点

```sql
taxa             (tax_id PK, scientific_name, common_name, rank)
assemblies       (accession PK, tax_id FK, name, source, refget_checksum, ...)
sequences        (id PK, assembly_accession FK, name, length, refget_checksum, fasta_path)
genes            (id PK, assembly_accession FK, symbol, locus_tag, sequence_id, start, end, strand)
transcripts      (id PK, gene_id FK, ...)
exons            (id PK, transcript_id FK, start, end)
```

`assemblies.source` は `'ncbi' | 'marpolbase' | 'tair' | 'phytozome' | 'community' | 'local'`。
植物では NCBI 以外の一次ソースが多いため、最初から複数 source を受け入れる前提。

### Cargo workspace 構成 (target: 8 crate)

```
backend/crates/
  genome-core/         # 型のみ (no IO, no async)
                       # TaxId, Accession, GeneId, Position0/1, Region
  coexpression/        # MR / HRR / LS, Pearson/Spearman, top-N (pure lib, OSS化候補)
  storage/             # FS/S3 抽象, refget checksum, .fai/.tbi index 操作
  db/                  # sqlx + migrations + repository
  expression-store/    # Parquet/DuckDB (発現マトリクス専用)
  service/             # ユースケース層 (太ったらドメイン別に分割)
  api/      [bin]      # axum HTTP サーバ
  ingest/   [bin]      # データ取り込み CLI
```

**依存方向**:
```
                     genome-core
              /     /    |    \     \
        storage  db  expression-store  coexpression
              \     |     /                /
                   service ────────────────
                    /   \
                  api   ingest
```

**MVP 時点で実装するのは 6 crate** (`coexpression`, `expression-store` は将来):
`genome-core` / `storage` / `db` / `service` / `api` / `ingest`

### 実装順 (MVP)

1. `genome-core` (型決め — 後から変更が一番痛い)
2. `db` + migrations (schema 決め打ち)
3. `storage` (refget だけまず動く状態に)
4. `service` 薄く + `api` で `GET /v2/gene/id/{id}` レベル
5. `ingest` CLI で Marchantia GFF/FASTA 取り込み
6. frontend で 種/遺伝子検索 + JBrowse 表示

### Frontend 構成

monorepo だが frontend は workspace 化しない (シンプル維持):

```
frontend/
  src/
    pages/
      Home.tsx                # 種一覧
      TaxonDetail.tsx         # 種詳細 + アセンブリ一覧
      AssemblyDetail.tsx
      GeneSearch.tsx
      GeneDetail.tsx          # 座標 + 配列 + JBrowse 埋め込み
    components/
      GenomeBrowser.tsx       # JBrowse 2 wrapper
```

## Coding guidelines

- **解析ロジックと永続化/HTTP は分離する**。`coexpression` のような解析 crate は
  `sqlx` / `axum` / `tokio` に依存しない。pure lib として保つ
- **座標系は型で区別**。`u64` を生で持ち回さない。`genome-core` の型を経由する
- **GA4GH 既存標準が使える場所では使う**(refget, htsget)。植物特有/オミクス特有の
  部分だけ独自に設計する
- **アセンブリ accession は first-class identifier**。表示名 (`TAIR10` 等) は
  二次属性として持つが、内部参照は accession (`GCA_xxx` / `LOCAL_xxx`) で行う
- **MVP は genome に集中**。将来のオミクス層を意識した crate 境界は引いておくが、
  実装は genome に絞る

## Notes

- 初期データ取り込み時に確定する事項: Marchantia の使用 assembly 版 (Tak-1 v6.1 等)、
  ソース (NCBI / MarpolBase), accession 表記
- 設計議論の経緯は repo-admins 側の対話履歴を参照
