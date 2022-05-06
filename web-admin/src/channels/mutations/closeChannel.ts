import sensei from "../../utils/sensei";

const closeChannel = async (channelId: string, force: boolean) => {
  return await sensei.closeChannel(channelId, force);
};

export default closeChannel;
