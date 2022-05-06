import sensei from "../../utils/sensei";

const deletePayment = async (paymentHash: string) => {
  return await sensei.deletePayment(paymentHash);
};

export default deletePayment;
