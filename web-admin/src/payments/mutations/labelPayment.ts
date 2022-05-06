import sensei from "../../utils/sensei";

const labelPayment = async (label: string, paymentHash: string) => {
  return await sensei.labelPayment(label, paymentHash);
};

export default labelPayment;
