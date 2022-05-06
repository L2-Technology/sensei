import sensei from "../../utils/sensei";

const createNode = async (
  username: string,
  alias: string,
  passphrase: string,
  start: boolean
) => {
  return sensei.createNode({ username, alias, passphrase, start });
};

export default createNode;
