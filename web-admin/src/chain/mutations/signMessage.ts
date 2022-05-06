import sensei from "../../utils/sensei";

const signMessage = async (message: string) => {
  return await sensei.signMessage(message);
};

export default signMessage;
