const datasets = [
  {
    assembly: "MpTak1_v7.1",
    species: "Marchantia polymorpha",
    status: "Available",
  },
];

type Dataset = (typeof datasets)[number];

const datasetExport: { datasets: Dataset[] } = {
  datasets,
};

export default datasetExport;
