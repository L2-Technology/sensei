import sensei from "../../utils/sensei";

const createAccessToken = async (
  name: string,
  scope: string,
  expiresAt: number,
  singleUse: boolean
) => {
  return sensei.createAccessToken({ name, scope, expiresAt, singleUse });
};

export default createAccessToken;
