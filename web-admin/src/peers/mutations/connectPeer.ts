import sensei from "../../utils/sensei";

const connectPeer = async (nodeConnectionString: string) => {
  return await sensei.connectPeer(nodeConnectionString);
};

export default connectPeer;
