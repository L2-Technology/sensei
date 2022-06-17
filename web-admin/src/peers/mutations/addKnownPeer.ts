import sensei from "../../utils/sensei";

const addKnownPeer = async (
  pubkey: string,
  label: string,
  zeroConf: boolean
) => {
  return await sensei.addKnownPeer(pubkey, label, zeroConf)
};

export default addKnownPeer;
