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
};

export default datasetExport;
