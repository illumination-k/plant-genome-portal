include { SRA_FETCH       } from '../modules/local/sra_fetch.nf'
include { FASTP           } from '../modules/local/fastp.nf'
include { SALMON_INDEX    } from '../modules/local/salmon_index.nf'
include { SALMON_QUANT    } from '../modules/local/salmon_quant.nf'
include { HISAT2_BUILD    } from '../modules/local/hisat2_build.nf'
include { HISAT2_ALIGN    } from '../modules/local/hisat2_align.nf'
include { SAMTOOLS_SORT   } from '../modules/local/samtools_sort.nf'
include { QUALIMAP_RNASEQ } from '../modules/local/qualimap_rnaseq.nf'
include { MULTIQC         } from '../modules/local/multiqc.nf'

workflow TRANSCRIPTOME {

    // ------------------------------------------------------------------
    // Param validation
    // ------------------------------------------------------------------
    if (!params.input)             error "Missing required parameter --input (CSV samplesheet)"
    if (!params.genome_fasta)      error "Missing required parameter --genome_fasta"
    if (!params.gtf)               error "Missing required parameter --gtf"
    if (!params.transcripts_fasta) error "Missing required parameter --transcripts_fasta"

    ch_genome      = file(params.genome_fasta,      checkIfExists: true)
    ch_gtf         = file(params.gtf,               checkIfExists: true)
    ch_transcripts = file(params.transcripts_fasta, checkIfExists: true)

    // ------------------------------------------------------------------
    // Parse samplesheet → branch SRA vs local FASTQ
    // ------------------------------------------------------------------
    ch_samples = Channel.fromPath(params.input, checkIfExists: true)
        .splitCsv(header: true, strip: true)
        .map { row ->
            def has_sra = row.sra && row.sra.trim()
            def strand  = (row.strandedness ?: 'auto').trim().toLowerCase()
            def meta = [
                id          : row.sample,
                sra         : has_sra ? row.sra.trim() : null,
                strandedness: strand,
                // single_end is finalised after SRA fetch; for local FASTQ rows it's known here
                single_end  : has_sra ? null : !(row.fastq_2 && row.fastq_2.trim()),
            ]
            return [meta, row]
        }

    ch_samples.branch { meta, row ->
        sra:   meta.sra != null
        local: true
    }.set { ch_branched }

    // ---- SRA branch ---------------------------------------------------
    SRA_FETCH(ch_branched.sra.map { meta, row -> [meta, meta.sra] })

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
    // Salmon transcript-level quantification
    // ------------------------------------------------------------------
    if (params.salmon_index) {
        ch_salmon_index = Channel.value(file(params.salmon_index, checkIfExists: true))
    } else {
        SALMON_INDEX(ch_transcripts, ch_genome)
        ch_salmon_index = SALMON_INDEX.out.index
    }
    SALMON_QUANT(FASTP.out.reads, ch_salmon_index)

    // ------------------------------------------------------------------
    // HISAT2 alignment → sorted BAM → BAM QC
    // ------------------------------------------------------------------
    if (params.hisat2_index) {
        ch_hisat2_index = Channel.value(file(params.hisat2_index, checkIfExists: true))
    } else {
        HISAT2_BUILD(ch_genome, ch_gtf)
        ch_hisat2_index = HISAT2_BUILD.out.index
    }
    HISAT2_ALIGN(FASTP.out.reads, ch_hisat2_index)
    SAMTOOLS_SORT(HISAT2_ALIGN.out.sam)
    QUALIMAP_RNASEQ(SAMTOOLS_SORT.out.bam, ch_gtf)

    // ------------------------------------------------------------------
    // Aggregate QC with MultiQC
    // ------------------------------------------------------------------
    if (!params.skip_multiqc) {
        ch_multiqc_files = Channel.empty()
            .mix(FASTP.out.json)
            .mix(HISAT2_ALIGN.out.summary)
            .mix(SAMTOOLS_SORT.out.flagstat)
            .mix(SAMTOOLS_SORT.out.stats)
            .mix(QUALIMAP_RNASEQ.out.results.map { meta, dir -> dir })
            .mix(SALMON_QUANT.out.results.map    { meta, dir -> dir })
            .collect()

        ch_multiqc_config = params.multiqc_config
            ? Channel.value(file(params.multiqc_config, checkIfExists: true))
            : Channel.value(file("${projectDir}/NO_FILE"))

        MULTIQC(ch_multiqc_files, ch_multiqc_config)
    }
}
