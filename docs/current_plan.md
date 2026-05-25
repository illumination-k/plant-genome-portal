# Current Plan

最終更新: 2026-05-25

## 目標

Genome MVP は完了扱いにする。次は MarpolBase Expression (MBEX) 相当のうち、まず **Expression の可視化** に絞る。

最初の目的は、gene detail から「この遺伝子がどのサンプル・組織・条件で発現しているか」を直感的に見られる状態にすること。co-expression、DEG、functional enrichment、set relation、orthology は後続フェーズに回す。

参考:

- Kawamura et al. 2022, "MarpolBase Expression: A Web-Based, Comprehensive Platform for Visualization and Analysis of Transcriptomes in the Liverwort Marchantia polymorpha"
- https://academic.oup.com/pcp/article/63/11/1745/6694961
- DOI: https://doi.org/10.1093/pcp/pcac129

## 今回やる範囲

### Data layer

- RNA-seq sample identity
  - SRA run / experiment / study
  - BioSample / BioProject
  - assembly accession
  - title / description
  - source paper / external links
  - library strategy / layout / platform
- Common visualization metadata trait
  - display label
  - primary group key
  - available facet keys
  - stable sort key
  - organism metadata profile name
- Organism-specific sample metadata
  - sample schema is not fixed globally
  - Marchantia can define fields such as organ, tissue, developmental stage, genotype, treatment, time point, sex, thallus region, gemma/cup context, light condition, stress condition
  - other species can define their own metadata fields without changing expression-core
  - raw metadata is preserved as typed organism metadata plus a string attribute map
- Gene-level expression matrix
  - TPM
  - raw count
  - optional: log2(TPM + 0.25)
- Expression summary
  - gene x sample values
  - sample group mean
  - sample group standard deviation
  - sample count per group

### API

- `GET /v2/expression/assemblies/{accession}/samples`
  - sample metadata list
  - filters are driven by the organism metadata profile
- `GET /v2/expression/genes/{gene_id}`
  - expression summary for one gene
  - grouped values for default grouping
- `GET /v2/expression/genes/{gene_id}/matrix`
  - raw per-sample expression values
  - query: `unit=tpm|count|log_tpm`
- `GET /v2/expression/metadata/facets`
  - UI filter options from the active organism metadata profile

### UI

- Gene detail に `Expression` tab を追加する
- Expression tab の初期表示
  - grouped TPM bar plot
  - group mean + standard deviation
  - sample count
  - sample table
- 表示切り替え
  - TPM / raw count / log TPM
  - group by fields exposed by the organism metadata profile
  - sort by genomic/default order / expression descending / group name
- CSV download
  - per-sample values
  - grouped summary
- Empty / loading / error state を genome UI と揃える

## やらない範囲

- Co-expression table / network
- PCC / MR / HRR matrix
- DEG viewer
- DESeq2 integration
- Functional enrichment
- Set relation / UpSet plot
- eFP / Chromatic Expression Image 風の臓器画像
- Orthology / phylogenetic tree
- private dataset upload
- real-time RNA-seq processing

## 実装順

### P0: 現状棚卸し

- `expression-core` の型を確認する
- `expression-store` の snapshot / repository 実装を確認する
- sample metadata を固定 schema にしないための trait 境界を決める
- 既存型で足りない identity / metadata profile / summary 型を洗い出す
- genome API への組み込み方を決める

Done 条件:

- Expression visualization に必要な domain model / storage model / API response が明確になっている
- core sample identity と organism-specific sample metadata の責務分離が明確になっている

### P1: 小さい fixture

- P0 用の小さい expression fixture を作る
  - genes: 3-5
  - samples: 6-12
  - groups: Marchantia metadata profile の organ / stage / treatment が分かる程度
  - units: TPM + raw count
- fixture で repository test を書く
- API smoke に使える deterministic data にする

Done 条件:

- CI で expression repository の最小読み書きが検証できる

### P2: Expression API

- API state に expression repository を追加する
- expression snapshot path を CLI option で渡せるようにする
- sample metadata endpoint を追加する
- gene expression summary endpoint を追加する
- gene expression matrix endpoint を追加する
- OpenAPI と TypeScript client を再生成する

Done 条件:

- fixture data で expression API が返る
- `pnpm --dir web run openapi:generate` 後に frontend client が型付きで使える

### P3: Gene detail Expression tab

- gene detail tabs に `Expression` を追加する
- grouped TPM bar plot を実装する
- sample table を実装する
- unit / group by / sort controls を実装する
- CSV download を実装する
- representative gene で表示確認する

Done 条件:

- `Mp1g00070` など代表 gene の Expression tab が表示できる
- grouped summary と per-sample values が確認できる
- `mise run ci` と `pnpm --dir web run build` が通る

### P4: 実データ import

- 対象 RNA-seq dataset を決める
- Marchantia 用 metadata profile と TSV schema を決める
- TPM / raw count matrix import を作る
- expression snapshot を生成する
- README / docs に再生成手順を書く

Done 条件:

- 実データ snapshot で Expression tab が表示できる
- source / sample metadata の由来が docs で追える

## 直近の作業

1. `expression-core` / `expression-store` の現状棚卸し
2. sample identity trait / metadata profile trait の設計
3. Marchantia metadata profile の設計
4. Expression visualization 用 response model の設計
5. 小さい fixture の追加
6. repository tests
7. API endpoints
8. frontend Expression tab

## 後続フェーズ

Expression visualization が固まった後に、次の順で広げる。

1. Co-expression MVP
2. Functional enrichment
3. DEG viewer
4. Heatmap / clustergram
5. eFP-like tissue overview
6. Orthology / phylogeny
