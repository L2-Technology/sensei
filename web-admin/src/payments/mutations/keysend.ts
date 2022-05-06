import sensei from "../../utils/sensei";

const keysend = async (destPubkey: string, amtMsat: number) => {
  return await sensei.keysend(destPubkey, amtMsat);
};

export default keysend;
