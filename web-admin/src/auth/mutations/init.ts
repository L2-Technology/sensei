import sensei from "../../utils/sensei";

const init = async (
  username: string,
  passphrase: string,
) => {
  return sensei.init({ username, passphrase });
};

export default init;
