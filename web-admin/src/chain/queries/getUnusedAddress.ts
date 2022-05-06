import sensei from "../../utils/sensei";

const getUnusedAddress = async () => {
  return await sensei.getUnusedAddress();
};

export default getUnusedAddress;
