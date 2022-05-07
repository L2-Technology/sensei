import sensei from "../../utils/sensei";

const openChannel = async (
  nodeConnectionString: string,
  amtSatoshis: number,
  isPublic: boolean
) => {
  return await sensei.openChannel(nodeConnectionString, amtSatoshis, isPublic);
};

export default openChannel;
