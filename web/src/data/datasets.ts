<<<<<<< Updated upstream
type Dataset = {
  assembly: string;
  species: string;
  status: string;
};

const datasetExport: { datasets: Dataset[] } = {
  datasets: [
    {
      assembly: "MpTak1_v7.1",
      species: "Marchantia polymorpha",
      status: "Available",
    },
  ],
||||||| Stash base
=======
const datasets = [
  { assembly: "IRGSP-1.0", species: "Oryza sativa", status: "Ready" },
  {
    assembly: "TAIR10",
    species: "Arabidopsis thaliana",
    status: "Ready",
  },
  {
    assembly: "Zm-B73-REFERENCE",
    species: "Zea mays",
    status: "Indexing",
  },
];

type Dataset = (typeof datasets)[number];

const datasetExport: { datasets: Dataset[] } = {
  datasets,
>>>>>>> Stashed changes
};

export default datasetExport;
