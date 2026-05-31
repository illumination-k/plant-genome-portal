#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

include { ORTHOLOGY } from './workflows/orthology.nf'

workflow {
    ORTHOLOGY()
}
