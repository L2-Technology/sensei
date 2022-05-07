import sensei from "../../utils/sensei";

const getPeers = async () => {
  return await sensei.getPeers();
};

export default getPeers;
