import sensei from "../../utils/sensei";

const getNodes = async ({ page, searchTerm, take }) => {
  return await sensei.getNodes({ page, searchTerm, take });
};

export default getNodes;
