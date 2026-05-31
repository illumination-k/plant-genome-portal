# General Plan

最終更新: 2026-05-26

## 目的

Plant Genome Portal は、既存の MarpolBase と MarpolBase Expression (MBEX) を統合し、さらにヒメツリガネゴケやツノゴケを含むコケ植物の機能ゲノミクス基盤へ拡張する。

短期的には **Marchantia polymorpha の genome / nomenclature / bulk RNA-seq / co-expression / pathway / analysis tools を一つの gene-centered portal に統合する**。中期的には liverwort, moss, hornwort を横断する **bryophyte functional genomics portal** として、コケ植物研究の de facto database を狙う。

このプロジェクトは、Ensembl Plants、Phytozome、NCBI Datasets のような汎植物・汎ゲノムDBと正面から競合しない。勝ち筋は、コケ植物モデル系に深く特化し、実験生物学者が遺伝子機能を調べるための genome、expression、co-expression、orthology、pathway、single-cell 情報を一貫したUI/APIで提供することにある。

## Positioning

### 既存資産

- MarpolBase
  - Marchantia genome assembly / annotation / nomenclature / analysis tools の中心リソース。
  - 今後の Plant Genome Portal では、MarpolBase の後継・統合版として扱う。
- MBEX
  - Marchantia bulk RNA-seq、TPM / raw count、co-expression を提供する expression database。
  - 今後は gene detail、pathway view、co-expression view、download/API に統合する。

### 外部DBとの役割分担

- NCBI Datasets
  - accession-first、TaxID-driven、data package / API の設計を参考にする。
  - 原データと公式アクセッションの参照元として使う。
- Ensembl Plants / Phytozome
  - 多種ゲノム閲覧、gene tree、orthology、comparative genomics に強い。
  - Plant Genome Portal は全植物網羅ではなく、コケ植物モデル系の深い機能ゲノミクスに集中する。
- BryoGenomes
  - 多数の bryophyte genomes の集約に強い。
  - Plant Genome Portal はモデル種の annotation、expression、co-expression、cell atlas、実験利用性を深掘りする。
- ATTED-II
  - plant co-expression の代表的先例。
  - MR / HRR / LS などの指標設計を参考にしつつ、Marchantia / moss / hornwort の横断解析に展開する。
- scPlantDB などの plant scRNA-seq databases
  - single-cell atlas の先例。
  - Plant Genome Portal は bryophyte-specific cell atlas と gene-centered integration で差別化する。

## Target Scope

### Core species

最初に扱うべき中核種:

- **Marchantia polymorpha**
  - 既存資産: MarpolBase / MBEX。
  - 最初の統合対象。
  - genome、nomenclature、bulk expression、co-expression、pathway、analysis tools の完成度を上げる。
- **Physcomitrium patens** / ヒメツリガネゴケ
  - moss model。
  - 公開ゲノム・発現データ・実験コミュニティがあり、Marchantia との比較対象として重要。
- **Representative hornwort**
  - Anthoceros など、利用可能な高品質 genome / annotation / transcriptome を優先する。
  - liverwort / moss / hornwort の3系統を並べることで、陸上植物進化・evo-devo の文脈を作る。

### Expansion species

コケ植物全体の de facto portal を狙う場合、全ゲノムを同じ深さで扱う必要はない。初期は depth tiers を分ける。

- Tier 1: genome + annotation + expression + co-expression + curated metadata
- Tier 2: genome + annotation + orthology + basic gene pages
- Tier 3: external links / accession registry / minimal metadata

## Product Concept

### Gene-centered portal

主要な入口は gene detail page とする。ユーザーは一つの遺伝子から以下へ移動できる。

- gene model / transcripts / exons / CDS
- sequence retrieval via refget
- functional annotation: GO, Pfam, InterPro, KEGG, KOG, NCBIfam
- nomenclature / symbol / aliases / literature links
- genome browser around the locus
- bulk RNA-seq expression by sample, tissue, treatment, genotype
- epigenome tracks: ChIP-seq, CUT&RUN, ATAC-seq, methylation, peaks, motifs
- co-expression neighbors and ranked edges
- KEGG pathway context and pathway-level expression
- orthologs and gene family members across bryophytes and selected outgroups
- BLAST / sequence search results
- future: single-cell expression by tissue, cluster, cell type, developmental stage

### API-first database

Web UI は主要な利用面だが、論文・再利用性の観点では API-first を明確にする。

- OpenAPI schema を公開する。
- TypeScript client は自動生成する。
- sequence API は GA4GH refget を維持する。
- 将来、alignment / variant / signal data では htsget や byte-range file serving を検討する。
- download endpoint では matrix、metadata、snapshot、annotation を再利用可能な形式で提供する。

### AI-first access

API-first の上に、AI-first / agent-ready なアクセス層を置く。ここでの AI-first は、Web UI をAIで置き換えるという意味ではなく、LLMや研究支援エージェントが安全に、根拠付きで、再現可能にデータへアクセスできるようにすることを指す。

- Gene、sample、dataset、pathway、orthology、epigenome track などを stable ID で参照できるようにする。
- API responses に source accession、dataset version、pipeline version、citation、license、last updated を含める。
- LLMが要約しやすい compact response と、再解析に使える full response を分ける。
- gene-centered context endpoint を用意し、1遺伝子について annotation、expression、co-expression、pathway、orthology、epigenome evidence をまとめて返せるようにする。
- AIが返す答えに根拠を付けられるよう、各フィールドの provenance を保持する。
- hallucination を避けるため、未収録データとゼロ値を明確に区別する。
- LLMは sample metadata、dataset summaries、gene context summaries、curation suggestions の生成に使う。
- LLM-generated metadata は早期に利用可能にするが、`generated_by_llm` / `curated` / `reviewed` などのタグで状態を明示する。
- 将来的には natural language query を直接DBに投げるのではなく、MCP tools や typed API calls に変換して実行する。

### LLM-assisted metadata curation

公開RNA-seq、ChIP-seq、ATAC-seq、methylation、scRNA-seq は、SRA / BioSample / GEO / ArrayExpress / 論文本文 / supplement にメタデータが分散し、表記ゆれも多い。Plant Genome Portal では、LLMを用いて初期メタデータ候補を生成し、community-based human curation で品質を担保する。

対象:

- sample title / description の正規化
- tissue / organ / developmental stage / genotype / treatment / time point
- assay type / library strategy / platform / layout
- ChIP-seq target / antibody / control sample
- ATAC-seq condition
- methylation context and treatment
- scRNA-seq tissue / cell type / cluster label / marker evidence
- paper / accession / supplementary table からの dataset summary

設計方針:

- LLM output は利便性のため早期に public metadata として表示できる。ただし `generated_by_llm` タグを必ず付ける。
- 人間が確認した field には `curated` または `reviewed` タグを付ける。
- curator が修正した field は `curated` とし、LLM由来であることも履歴として保持する。
- 各 metadata field は source evidence、confidence、LLM model/version、prompt/template version、generated_at、review status、curation tags を持つ。
- curator は approve / edit / reject / request discussion を行える。
- review history と contributor attribution を保持する。
- community curator には role-based permission を設定する。
- conflicting curation は issue / discussion として残し、最終判断者を明示する。
- official release には `generated_by_llm` のままの値も含められるが、タグとconfidenceを落とさない。
- high-confidence LLM metadata は検索・facet・summary に使えるようにする。
- low-confidence または disputed metadata は検索重みを下げるか、UIで注意表示する。
- APIでは raw metadata、LLM-generated metadata、human-curated metadata を区別して返せるようにする。
- UIでは sample table、facet、download、MCP response に `generated_by_llm` / `curated` タグを表示する。
- 論文では LLM支援を自動annotationではなく、human-in-the-loop curation workflow として説明する。

### Authentication and API keys

初期の public read-only portal では、基本データ閲覧は認証なしで提供する。一方で、重い処理、ジョブ実行、private / embargoed dataset、AI agent traffic、rate-limit 管理には認証とAPI keyを導入する。

- Public anonymous access
  - gene pages
  - genome metadata
  - basic annotation
  - small expression queries
  - public downloads
- API key access
  - higher rate limits
  - BLAST / enrichment / co-expression / large matrix jobs
  - batch API
  - MCP server access
  - usage tracking for computational users
- Authenticated user access
  - private datasets before publication
  - draft imports
  - community metadata curation
  - collaborator-only previews
  - saved gene sets / analysis sessions
  - admin curation tools

設計方針:

- 公開DBとしての friction は増やさない。
- API key は最初は rate limit と job ownership のために使う。
- private dataset support は後続フェーズに回すが、metadata model は public/private を区別できるようにする。
- user identity、API key、quota、job ownership、audit log は portal 本体の研究データモデルから分離する。

### MCP server

Model Context Protocol (MCP) server を公式な AI agent interface として用意する。MCP は OpenAPI の代替ではなく、OpenAPI / internal service layer の上に載る agent-friendly wrapper とする。

最初に提供する tools:

- `search_genes`
- `get_gene`
- `get_gene_context`
- `get_expression`
- `get_coexpression_neighbors`
- `get_pathway`
- `get_orthologs`
- `get_region_features`
- `get_epigenome_tracks`
- `get_peaks_near_gene`
- `run_blastn`
- `list_datasets`
- `get_dataset_provenance`

設計方針:

- MCP responses は小さく、引用・ID・取得日時・dataset version を含める。
- 大きな matrix や track data は直接返さず、download URL または job result reference を返す。
- destructive / private actions は明示的な user auth と permission check を必須にする。
- AI agent が論文・解析ノート・実験計画を作るときに、どのデータに基づく記述か追跡できるようにする。

### Reproducible data builds

database paper として成立させるには、データ投入手順を再現可能にする必要がある。

- raw data accession list を固定する。
- pipeline version と container image を記録する。
- QC基準を明示する。
- rejected samples と理由を保存する。
- generated matrix / annotation / co-expression index に version を付ける。
- public release ごとに DOI または archival snapshot を用意する。

## Data Layers

### Genome layer

対象:

- taxon
- species / strain / accession
- assembly
- annotation version
- sequences
- genes / transcripts / exons / CDS
- functional annotations
- nomenclature

設計方針:

- API境界は accession-first / TaxID-driven を維持する。
- sequence identity は refget checksum を持つ。
- assembly と annotation は将来的に別versionとして扱う。
- FASTA / GFF / GTF / indexed browser files はDBに埋め込まず、ファイルとして保持する。

### Bulk expression layer

対象:

- SRA run / experiment / study
- BioSample / BioProject
- sample metadata
- tissue / organ / developmental stage / genotype / treatment / time point
- TPM
- raw count
- normalized count where needed
- sample group summary
- gene x sample matrix

設計方針:

- sample identity はSRA/BioSample/BioProjectを主キー情報として保持する。
- species-specific metadata は固定schemaに押し込まず、profile + attributes として扱う。
- TPM / raw count は保存unit、log2(TPM + offset) は表示変換として扱う。
- UIのfacetは metadata profile から生成する。
- sample metadata は raw imported values、LLM-generated values、human-curated values を分けて保持する。
- sample curation tags は `generated_by_llm` / `curated` / `reviewed` / `disputed` / `deprecated` などを扱えるようにする。
- facet や search では `generated_by_llm` の値も使えるが、UI/API/download でタグを明示する。

### Co-expression layer

対象:

- Pearson / Spearman correlation
- MR: Mutual Rank
- HRR: Highest Reciprocal Rank
- LS: Logit Score
- top-N co-expression neighbors
- network view
- gene set / pathway-level co-expression

設計方針:

- dense matrix と query index は分ける。
- 論文では ATTED-II 系の指標との互換性・差分を説明する。
- Marchantia で先に完成させ、moss / hornwort に横展開する。

### Epigenome layer

対象:

- ChIP-seq
- CUT&RUN / CUT&Tag
- ATAC-seq
- DNA methylation
- histone marks
- transcription factor binding
- chromatin accessibility
- peak calls
- motif enrichment and de novo motifs
- bigWig signal tracks
- BAM/CRAM alignment tracks where appropriate
- gene-proximal regulatory regions

設計方針:

- raw alignment / signal files はDBに入れず、BAM/CRAM、bigWig、BED/narrowPeak/broadPeak などの indexed files として保持する。
- metadata、sample、assay、target、antibody、condition、replicate、QC metrics、peak set はDB/API側で管理する。
- gene detail では promoter / gene body / nearby regulatory regions の signal と peaks を見られるようにする。
- genome browser では assay / target / condition で track を選択できるようにする。
- ChIP-seq / CUT&RUN は既存の `pipelines/chipseq` を portal import に接続する。
- ATAC-seq と methylation は別pipelineとして追加するか、既存pipelineを拡張する。
- methylation は cytosine context (CG / CHG / CHH)、region-level summary、gene body / promoter summary を扱えるようにする。
- 将来的には expression、co-expression、single-cell と接続し、regulatory evidence として gene detail に統合する。

### Orthology and gene family layer

対象:

- pairwise orthologs
- many-to-many orthogroups
- gene family
- selected outgroups: Arabidopsis, rice, Selaginella, algae where useful
- synteny or collinearity if available

設計方針:

- bryophyte portal としては必須の中核レイヤー。
- 初期は OrthoFinder / SonicParanoid / Ensembl / Phytozome 由来など、再現可能な方法を一つ決める。
- gene detail から ortholog table を引ける状態を最初の完成形にする。

### Pathway and functional enrichment layer

対象:

- KEGG KO / pathway / module / reaction
- Plant Reactome or Gramene links where useful
- GO enrichment
- gene set enrichment
- pathway expression heatmap

設計方針:

- KEGG annotation は gene detail と pathway detail の両方から辿れるようにする。
- enrichment は co-expression / DEG / selected gene set の下流解析として提供する。

### Single-cell layer

対象:

- scRNA-seq datasets
- cell barcode metadata
- sample / tissue / developmental stage / genotype / treatment
- cluster / cell type / marker genes
- UMAP / PCA coordinates
- raw counts / normalized counts
- pseudobulk expression
- gene expression by cluster / cell type

設計方針:

- bulk expression と同じテーブルに押し込まない。
- metadata はDB、large matrix は H5AD / Zarr / Parquet などに分離する。
- 最初の user-facing API は以下に絞る。
  - dataset list
  - cell type / cluster list
  - marker genes
  - gene expression by cluster
  - UMAP coordinates for selected dataset
  - pseudobulk by sample or cell type
- single-cell は将来の独立論文候補として扱う。

## Architecture Direction

### Current

- in-memory genome snapshot
- FASTA on disk
- optional expression snapshot
- Rust API
- React SPA
- OpenAPI generated client
- JBrowse 2 integration
- Nextflow transcriptome pipeline
- Nextflow ChIP-seq / CUT&RUN pipeline

### Medium-term

- authentication and API key infrastructure
- rate limits and job ownership
- PostgreSQL for metadata and annotation indexes
- Parquet / Arrow / Zarr for large expression matrices
- file-backed sequence / signal data
- object storage compatible layout for public release
- versioned dataset registry
- background workers for BLAST, co-expression index build, import jobs
- epigenome track registry and peak query API
- MCP server backed by the typed service layer

### Long-term

- collaborator login and private dataset preview
- multi-assembly and multi-annotation support
- orthology and gene family index
- epigenome signal and regulatory annotation layer
- scRNA-seq atlas storage
- downloadable public snapshots
- stable public API versioning
- stable AI-first / MCP tool versioning
- DOI-backed data releases

## Roadmap

### Cross-cutting platform layer

Goal:

- Public users, computational users, and AI agents can access the same curated data safely through UI, OpenAPI, downloads, and MCP.

Deliverables:

- anonymous public read access
- API key issuance and validation
- quota and rate limit policy
- job ownership for BLAST / enrichment / co-expression / large matrix requests
- provenance-rich API responses
- `get_gene_context` endpoint
- MCP server with read-only research tools
- LLM-assisted metadata suggestion pipeline
- community curation UI and review workflow
- private dataset and collaborator access design

Paper value:

- Positions the portal as an AI-ready and community-curated research database, not only a human-facing website.

### Phase 1: MarpolBase + MBEX integration

Goal:

- Marchantia genome and expression resources are integrated into one portal.

Deliverables:

- gene detail with genome model, annotation, sequence, expression, KEGG, browser
- sample list and facet API
- TPM / raw count / log-transformed visualization
- expression CSV download
- pathway expression heatmap
- BLASTN job API and UI
- public dataset metadata and download links
- OpenAPI documentation with stable identifiers and provenance fields
- initial API key support for job-style endpoints

Paper value:

- Official integrated successor to MarpolBase and MBEX.
- Reproducible, API-first, and AI-ready implementation.

### Phase 2: Co-expression and gene set analysis

Goal:

- MBEX-style co-expression becomes a first-class portal feature.

Deliverables:

- co-expression index build pipeline
- gene co-expression endpoint
- co-expression table and network UI
- MR / HRR / LS display
- gene set input
- functional enrichment
- pathway-level interpretation

Paper value:

- Moves the portal from expression visualization to functional discovery.

### Phase 3: Bryophyte multi-species foundation

Goal:

- Marchantia, Physcomitrium, and a representative hornwort share the same portal model.

Deliverables:

- species / assembly / annotation registry
- multi-species dataset table
- gene pages for moss and hornwort
- JBrowse config per assembly
- shared functional annotation pipeline
- basic orthology table
- cross-species gene search

Paper value:

- Establishes a bryophyte functional genomics portal rather than a Marchantia-only database.

### Phase 4: Epigenome and regulatory genomics

Goal:

- ChIP-seq, CUT&RUN, ATAC-seq, and methylation datasets become visible from gene pages and genome browser tracks.

Deliverables:

- epigenome dataset registry
- assay / target / condition / replicate metadata model
- bigWig and peak track serving
- peak query API by gene or genomic region
- gene-proximal regulatory summary
- motif result import and display
- ChIP-seq / CUT&RUN pipeline outputs connected to portal import
- ATAC-seq and methylation pipeline design

Paper value:

- Expands the portal from transcriptome-centric functional genomics to regulatory genomics.

### Phase 5: Orthology and comparative views

Goal:

- Users can move from one gene to homologs, orthologs, and gene families across bryophytes.

Deliverables:

- orthogroup pipeline
- ortholog API
- gene family page
- phylogenetic tree or external tree links
- cross-species expression comparison where data supports it

Paper value:

- Enables evo-devo and land plant evolution use cases.

### Phase 6: Single-cell atlas integration

Goal:

- scRNA-seq datasets become discoverable from gene, tissue, and cell-type views.

Deliverables:

- single-cell dataset registry
- H5AD/Zarr-backed matrix access
- UMAP view
- cluster / cell type marker table
- gene expression by cluster
- pseudobulk expression
- links between bulk tissue expression and single-cell cell types

Paper value:

- Strong candidate for a second database paper focused on bryophyte cell atlas resources.

## Publication Strategy

### Paper 1

Recommended scope:

- MarpolBase + MBEX integration.
- Marchantia genome / expression / co-expression / pathway / API.
- Foundation for multi-species bryophyte expansion.

Possible message:

> An API-first and AI-ready integrated genome and transcriptome portal for Marchantia polymorpha, unifying MarpolBase and MBEX and providing a foundation for bryophyte comparative functional genomics.

Minimum requirements:

- public portal
- stable Marchantia dataset release
- downloadable genome/expression/co-expression resources
- reproducible import pipeline
- OpenAPI documentation
- API key support for job-style endpoints
- MCP server or documented MCP-ready prototype
- LLM-assisted metadata curation workflow with human review
- biological use cases
- comparison table against MarpolBase, MBEX, Ensembl Plants, Phytozome, BryoGenomes, ATTED-II

### Paper 2

Recommended scope:

- Bryophyte multi-species portal.
- Marchantia + moss + hornwort.
- epigenome tracks, orthology, gene family, comparative expression.

Possible message:

> A bryophyte functional genomics portal for comparative and regulatory analysis across liverworts, mosses, and hornworts.

### Paper 3

Recommended scope:

- scRNA-seq / cell atlas.
- cell types, markers, pseudobulk, evolutionary comparison.

Possible message:

> A bryophyte single-cell atlas integrated with genome and transcriptome resources.

## Success Criteria

### Scientific

- Users can start from a gene and understand its annotation, expression, pathway context, co-expression partners, and orthologs.
- Users can inspect regulatory evidence around a gene, including chromatin accessibility, histone marks, TF binding, methylation, nearby peaks, and motifs.
- Marchantia data are more integrated than MarpolBase and MBEX as separate systems.
- Moss and hornwort are present enough to support cross-bryophyte comparison.
- Use cases demonstrate biological discovery or clear re-discovery of known biology.

### Technical

- Public API is documented by OpenAPI.
- MCP tools expose common gene, expression, pathway, orthology, epigenome, and dataset provenance queries.
- API key and rate limit infrastructure protects heavy endpoints without blocking public browsing.
- Data releases are versioned and reproducible.
- LLM-generated sample metadata can become useful quickly while remaining clearly tagged and traceable.
- Large matrices are not forced into request-time computation.
- Genome browser, expression visualization, epigenome tracks, BLAST, and downloads work from the same dataset registry.
- The system can add a new species without rewriting core UI/API contracts.

### Community

- The portal becomes the default entry point for Marchantia functional genomics.
- It provides a credible path to become the default entry point for bryophyte model species.
- It supports both wet-lab users through UI and computational users through API/downloads.
- It supports AI-assisted literature review, gene prioritization, experiment planning, and reproducible query workflows through MCP/API access.
- It lets trusted community curators improve sample metadata while preserving provenance, review status, and contributor attribution.

## Near-term Priorities

1. Finish the Marchantia expression API contract.
2. Load real MBEX-equivalent expression data into the new model.
3. Add sample/facet UI, unit switching, sorting, and CSV downloads.
4. Add co-expression query/API/UI from the existing co-expression crate.
5. Define epigenome dataset metadata and connect `pipelines/chipseq` outputs to portal import.
6. Add provenance fields needed for AI-first gene context responses.
7. Define API key scope for job-style endpoints and future MCP access.
8. Draft the first read-only MCP tool set around gene context and dataset provenance.
9. Define LLM-assisted sample metadata schema with `generated_by_llm` and `curated` tags.
10. Design community curation roles, review UI, and contributor attribution.
11. Define species / assembly / annotation versioning before adding moss and hornwort.
12. Choose the first moss and hornwort assemblies and document their source data.
13. Define the orthology pipeline before claiming cross-species comparative genomics.
14. Keep scRNA-seq as a planned separate layer, not an extension of bulk expression tables.
