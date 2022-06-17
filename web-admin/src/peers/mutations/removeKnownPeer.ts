import sensei from "../../utils/sensei";

const removeKnownPeer = async (
  pubkey: string
) => {
  return await sensei.removeKnownPeer(pubkey)
};

export default removeKnownPeer;
