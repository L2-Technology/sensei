import sensei from "../../utils/sensei";

const openChannel = async (
  nodeConnectionString: string,
  amtSatoshis: number,
  isPublic: boolean
) => {
  const connectionParts = nodeConnectionString.split("@");
  return await sensei.openChannel({
    counterpartyPubkey: connectionParts[0],
    counterpartyHostPort: connectionParts[1],
    amountSats: amtSatoshis,
    public: isPublic
  });
};

export default openChannel;
