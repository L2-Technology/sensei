import sensei from "../../utils/sensei";

const getPeers = async ({ page, take, searchTerm }) => {
  return await sensei.getKnownPeers({page, take, searchTerm})
};

export default getPeers;
