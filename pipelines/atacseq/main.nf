#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

include { ATACSEQ } from './workflows/atacseq.nf'

workflow {
    ATACSEQ()
}
