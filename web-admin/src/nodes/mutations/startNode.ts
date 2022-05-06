import sensei from "../../utils/sensei";

const startNode = async (passphrase: string) => {
  return await sensei.startNode(passphrase);
};

export default startNode;
