import sensei from "../../utils/sensei";

const getChannels = async ({ page, searchTerm, take }) => {
  return await sensei.getChannels({ page, searchTerm, take });
};

export default getChannels;
