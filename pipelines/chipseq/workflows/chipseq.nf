include { SRA_FETCH                    } from '../modules/local/sra_fetch.nf'
include { FASTP                        } from '../modules/local/fastp.nf'
include { SAMTOOLS_FAIDX               } from '../modules/local/samtools_faidx.nf'
include { BOWTIE2_BUILD                } from '../modules/local/bowtie2_build.nf'
include { BOWTIE2_ALIGN                } from '../modules/local/bowtie2_align.nf'
include { SAMTOOLS_SORT                } from '../modules/local/samtools_sort.nf'
include { PICARD_MARKDUPLICATES        } from '../modules/local/picard_markduplicates.nf'
include { SAMTOOLS_FILTER              } from '../modules/local/samtools_filter.nf'
include { PICARD_COLLECTMULTIPLEMETRICS } from '../modules/local/picard_collectmultiplemetrics.nf'
include { DEEPTOOLS_BAMCOVERAGE        } from '../modules/local/deeptools_bamcoverage.nf'
include { MACS3_CALLPEAK               } from '../modules/local/macs3_callpeak.nf'
include { SAMTOOLS_NSORT               } from '../modules/local/samtools_nsort.nf'
include { BEDTOOLS_FRAGBEDGRAPH        } from '../modules/local/bedtools_fragbedgraph.nf'
include { SEACR_CALLPEAK               } from '../modules/local/seacr_callpeak.nf'
include { HOMER_FINDMOTIFS             } from '../modules/local/homer_findmotifs.nf'
include { MULTIQC                      } from '../modules/local/multiqc.nf'

workflow CHIPSEQ {

    // ------------------------------------------------------------------
    // Param validation
    // ------------------------------------------------------------------
    if (!params.input)        error "Missing required parameter --input (CSV samplesheet)"
    if (!params.genome_fasta) error "Missing required parameter --genome_fasta"

    ch_genome    = file(params.genome_fasta, checkIfExists: true)
    ch_no_file   = file("${projectDir}/assets/NO_FILE")

    // Closure: row → [meta, row]
    def parse_row = { row ->
        def has_sra = row.sra && row.sra.trim()
        def assay   = (row.assay ?: 'chipseq').trim().toLowerCase()
        if (!(assay in ['chipseq', 'cutrun'])) {
            error "Sample ${row.sample}: unknown assay '${assay}' (expected 'chipseq' or 'cutrun')"
        }
        def meta = [
            id        : row.sample,
            group     : (row.group ?: row.sample).trim(),
            replicate : (row.replicate ?: '1').trim(),
            control   : (row.control && row.control.trim()) ? row.control.trim() : null,
            assay     : assay,
            sra       : has_sra ? row.sra.trim() : null,
            single_end: has_sra ? null : !(row.fastq_2 && row.fastq_2.trim()),
        ]
        return [meta, row]
    }

    // ------------------------------------------------------------------
    // Parse samplesheet → branch SRA vs local FASTQ
    // ------------------------------------------------------------------
    ch_samples = Channel.fromPath(params.input, checkIfExists: true)
        .splitCsv(header: true, strip: true)
        .map(parse_row)

    // Separately compute the set of sample IDs referenced as controls.
    // Parse the CSV a second time so this and the sample branch each get a
    // fresh queue channel.
    ch_ctrl_ids = Channel.fromPath(params.input, checkIfExists: true)
        .splitCsv(header: true, strip: true)
        .map(parse_row)
        .map { meta, _row -> meta.control }
        .filter { it != null }
        .unique()
        .collect()
        .ifEmpty([])
        .map { ids -> ids as Set }

    ch_samples.branch { meta, _row ->
        sra:   meta.sra != null
        local: true
    }.set { ch_branched }

    // ---- SRA branch ---------------------------------------------------
    SRA_FETCH(ch_branched.sra.map { meta, _row -> [meta, meta.sra] })

    ch_sra_reads = SRA_FETCH.out.reads.map { meta, fqs ->
        def list = (fqs instanceof List) ? fqs : [fqs]
        def sorted = list.sort { it.name }
        def m = meta + [single_end: sorted.size() == 1]
        return [m, sorted.size() == 1 ? sorted[0] : sorted]
    }

    // ---- Local FASTQ branch ------------------------------------------
    ch_local_reads = ch_branched.local.map { meta, row ->
        def r1 = file(row.fastq_1, checkIfExists: true)
        def reads = (row.fastq_2 && row.fastq_2.trim())
            ? [r1, file(row.fastq_2, checkIfExists: true)]
            : r1
        return [meta, reads]
    }

    ch_reads = ch_local_reads.mix(ch_sra_reads)

    // ------------------------------------------------------------------
    // QC + trim
    // ------------------------------------------------------------------
    FASTP(ch_reads)

    // ------------------------------------------------------------------
    // Reference: faidx + Bowtie2 index
    // ------------------------------------------------------------------
    SAMTOOLS_FAIDX(ch_genome)

    if (params.bowtie2_index) {
        ch_bt2_index = Channel.value(file(params.bowtie2_index, checkIfExists: true))
    } else {
        BOWTIE2_BUILD(ch_genome)
        ch_bt2_index = BOWTIE2_BUILD.out.index
    }

    // ------------------------------------------------------------------
    // Align → sort → mark duplicates → filter
    // ------------------------------------------------------------------
    BOWTIE2_ALIGN(FASTP.out.reads, ch_bt2_index)
    SAMTOOLS_SORT(BOWTIE2_ALIGN.out.sam)
    PICARD_MARKDUPLICATES(SAMTOOLS_SORT.out.bam)
    SAMTOOLS_FILTER(PICARD_MARKDUPLICATES.out.bam)

    ch_filt_bam = SAMTOOLS_FILTER.out.bam

    PICARD_COLLECTMULTIPLEMETRICS(ch_filt_bam, SAMTOOLS_FAIDX.out.fasta)

    // ------------------------------------------------------------------
    // BigWig coverage tracks
    // ------------------------------------------------------------------
    if (!params.skip_bigwig) {
        DEEPTOOLS_BAMCOVERAGE(ch_filt_bam)
    }

    // ------------------------------------------------------------------
    // Peak calling
    //   - IP samples (control != null OR explicitly no-control IP) get peaks.
    //   - Pure control samples (where another row references them) get no peaks.
    //   - chipseq → MACS3; cutrun → SEACR (PE only).
    // ------------------------------------------------------------------
    ch_macs3_peaks = Channel.empty()
    ch_seacr_peaks = Channel.empty()

    if (!params.skip_peaks) {
        // Tag each filtered BAM as IP or control using the pre-computed ctrl_ids.
        ch_tagged = ch_filt_bam.combine(ch_ctrl_ids).map { meta, bam, bai, ctrl_set ->
            def is_control = ctrl_set.contains(meta.id)
            return [meta + [is_control: is_control], bam, bai]
        }

        ch_tagged.branch { meta, _bam, _bai ->
            control: meta.is_control
            ip:      !meta.is_control
        }.set { ch_split }

        // Map of control_id → (bam, bai), produced from the control branch.
        ch_control_map = ch_split.control.map { meta, bam, bai -> [meta.id, bam, bai] }

        // IP samples with a control → join on control id
        ch_ip_with_ctrl = ch_split.ip
            .filter { meta, _bam, _bai -> meta.control != null }
            .map { meta, bam, bai -> [meta.control, meta, bam, bai] }
            .combine(ch_control_map, by: 0)
            .map { _ctrl_id, meta, ip_bam, ip_bai, ctrl_bam, ctrl_bai ->
                [meta, ip_bam, ip_bai, ctrl_bam, ctrl_bai]
            }

        // IP samples without a control → pad with NO_FILE sentinels
        ch_ip_no_ctrl = ch_split.ip
            .filter { meta, _bam, _bai -> meta.control == null }
            .map { meta, bam, bai -> [meta, bam, bai, ch_no_file, ch_no_file] }

        ch_ip_for_peaks = ch_ip_with_ctrl.mix(ch_ip_no_ctrl)

        // Split by assay
        ch_ip_for_peaks.branch { meta, _ibam, _ibai, _cbam, _cbai ->
            chipseq: meta.assay == 'chipseq'
            cutrun:  meta.assay == 'cutrun'
        }.set { ch_by_assay }

        // ---- MACS3 for chipseq -----------------------------------------
        MACS3_CALLPEAK(ch_by_assay.chipseq)
        ch_macs3_peaks = MACS3_CALLPEAK.out.peaks

        // ---- SEACR for cutrun ------------------------------------------
        // SEACR requires PE data — fail loudly on SE rows tagged cutrun.
        // Convert each *unique* control BAM to a bedgraph only once, even if
        // multiple IPs reference it.
        ch_cutrun_ip = ch_split.ip
            .filter { meta, _bam, _bai -> meta.assay == 'cutrun' }
            .map { meta, bam, bai ->
                if (meta.single_end) {
                    error "Sample ${meta.id}: assay=cutrun requires paired-end reads (SEACR needs fragments)"
                }
                return [meta, bam, bai]
            }

        // Set of control IDs actually used by cutrun IPs — re-parse the CSV
        // so this is its own queue channel.
        ch_cutrun_ctrl_used_ids = Channel.fromPath(params.input, checkIfExists: true)
            .splitCsv(header: true, strip: true)
            .map(parse_row)
            .filter { meta, _r -> meta.assay == 'cutrun' && meta.control != null }
            .map { meta, _r -> meta.control }
            .unique()
            .collect()
            .ifEmpty([])
            .map { ids -> ids as Set }

        ch_cutrun_ctrl = ch_split.control
            .combine(ch_cutrun_ctrl_used_ids)
            .filter { meta, _bam, _bai, used_set -> used_set.contains(meta.id) }
            .map { meta, bam, bai, _set -> [meta, bam, bai] }

        SAMTOOLS_NSORT(ch_cutrun_ip.mix(ch_cutrun_ctrl))
        BEDTOOLS_FRAGBEDGRAPH(SAMTOOLS_NSORT.out.bam, SAMTOOLS_FAIDX.out.sizes)

        // Tag each emitted bedgraph as IP or control, then rejoin IPs to their
        // control bedgraph by control id.
        ch_bg_tagged = BEDTOOLS_FRAGBEDGRAPH.out.bedgraph
            .combine(ch_cutrun_ctrl_used_ids)
            .map { meta, bg, used_set ->
                [meta + [is_control: used_set.contains(meta.id)], bg]
            }

        ch_bg_tagged.branch { meta, _bg ->
            ctrl: meta.is_control
            ip:   true
        }.set { ch_bg_split }

        // Key IP bedgraphs by their control id (or '' for no-control)
        ch_ip_bg   = ch_bg_split.ip.map   { meta, bg -> [meta.control ?: '', meta, bg] }
        ch_ctrl_bg = ch_bg_split.ctrl.map { meta, bg -> [meta.id, bg] }

        ch_seacr_with_ctrl = ch_ip_bg
            .filter { ctrl_id, _meta, _bg -> ctrl_id != '' }
            .combine(ch_ctrl_bg, by: 0)
            .map { _ctrl_id, meta, ip_bg, ctrl_bg -> [meta, ip_bg, ctrl_bg] }

        ch_seacr_no_ctrl = ch_ip_bg
            .filter { ctrl_id, _meta, _bg -> ctrl_id == '' }
            .map { _ctrl_id, meta, ip_bg -> [meta, ip_bg, ch_no_file] }

        SEACR_CALLPEAK(ch_seacr_with_ctrl.mix(ch_seacr_no_ctrl))
        ch_seacr_peaks = SEACR_CALLPEAK.out.peaks
    }

    // ------------------------------------------------------------------
    // Motif analysis (HOMER findMotifsGenome.pl)
    //   - Runs on MACS3 narrowPeak/broadPeak and on SEACR peak BEDs.
    //   - Produces both known motif enrichment and de novo discovery.
    // ------------------------------------------------------------------
    if (!params.skip_peaks && !params.skip_motifs) {
        ch_macs3_bed = ch_macs3_peaks.map { meta, files ->
            def file_list = (files instanceof List) ? files : [files]
            def bed = file_list.find { it.name.endsWith('.narrowPeak') || it.name.endsWith('.broadPeak') }
            return bed ? [meta, bed] : null
        }.filter { it != null }

        ch_seacr_bed = ch_seacr_peaks.map { meta, files ->
            def file_list = (files instanceof List) ? files : [files]
            def bed = file_list.find { it.name.endsWith('.bed') }
            return bed ? [meta, bed] : null
        }.filter { it != null }

        HOMER_FINDMOTIFS(ch_macs3_bed.mix(ch_seacr_bed), ch_genome)
    }

    // ------------------------------------------------------------------
    // Aggregate QC with MultiQC
    // ------------------------------------------------------------------
    if (!params.skip_multiqc) {
        ch_multiqc_files = Channel.empty()
            .mix(FASTP.out.json)
            .mix(BOWTIE2_ALIGN.out.log)
            .mix(SAMTOOLS_SORT.out.flagstat)
            .mix(SAMTOOLS_SORT.out.stats)
            .mix(PICARD_MARKDUPLICATES.out.metrics)
            .mix(SAMTOOLS_FILTER.out.flagstat)
            .mix(PICARD_COLLECTMULTIPLEMETRICS.out.metrics.map { _meta, files -> files })
            .collect()

        ch_multiqc_config = params.multiqc_config
            ? Channel.value(file(params.multiqc_config, checkIfExists: true))
            : Channel.value(ch_no_file)

        MULTIQC(ch_multiqc_files, ch_multiqc_config)
    }
}
