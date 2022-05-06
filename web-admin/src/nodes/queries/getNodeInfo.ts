import sensei from "../../utils/sensei";

const getNodeInfo = async () => {
  return await sensei.getInfo();
};

export default getNodeInfo;
