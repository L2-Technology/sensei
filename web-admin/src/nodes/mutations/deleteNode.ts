import sensei from "../../utils/sensei";

const deleteNode = async (pubkey) => {
  return await sensei.deleteNode(pubkey);
};

export default deleteNode;
