import sensei from "../../utils/sensei";

const getBalance = async () => {
  return await sensei.getBalance();
};

export default getBalance;
