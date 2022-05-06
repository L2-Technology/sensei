import sensei from "../../utils/sensei";

const getStatus = async () => {
  return await sensei.getStatus();
};

export default getStatus;
