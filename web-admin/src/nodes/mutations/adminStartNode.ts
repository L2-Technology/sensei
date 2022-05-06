import sensei from "../../utils/sensei";

const adminStartNode = async (pubkey: string, passphrase: string) => {
  return await sensei.adminStartNode(pubkey, passphrase);
};

export default adminStartNode;
