import sensei from "../../utils/sensei";

const adminStopNode = async (pubkey: string) => {
  return await sensei.adminStopNode(pubkey);
};

export default adminStopNode;
